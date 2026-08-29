#![forbid(unsafe_code)]

use std::{future::Future, time::Duration};

use axum::{Router, http::HeaderName, http::StatusCode, routing::get};
use sqlx::{PgPool, Postgres, Transaction};
use tokio::net::TcpListener;
use tower_http::{
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
};

const REQUEST_BODY_LIMIT_BYTES: usize = 1024 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Minimal HTTP surface used only to verify the Stage-1 Axum/Tower assumptions.
pub fn verification_router() -> Router {
    let request_id = HeaderName::from_static("x-request-id");

    Router::new()
        .route("/health", get(|| async { "ok" }))
        .layer(PropagateRequestIdLayer::new(request_id.clone()))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::GATEWAY_TIMEOUT,
            REQUEST_TIMEOUT,
        ))
        .layer(RequestBodyLimitLayer::new(REQUEST_BODY_LIMIT_BYTES))
        .layer(SetRequestIdLayer::new(request_id, MakeRequestUuid))
}

/// Runs the verification HTTP surface with explicit graceful-shutdown wiring.
/// Production binaries may add richer cancellation coordination later, but the
/// Stage-1 spike must prove the chosen Axum/Tokio lifecycle supports it cleanly.
pub async fn serve_verification(
    listener: TcpListener,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> std::io::Result<()> {
    axum::serve(listener, verification_router())
        .with_graceful_shutdown(shutdown)
        .await
}

/// Transaction wrapper used by the verification spike to prove that tenant
/// context is transaction-local and enforced by PostgreSQL RLS.
pub struct TenantTransaction<'a> {
    transaction: Transaction<'a, Postgres>,
}

impl<'a> TenantTransaction<'a> {
    pub async fn begin(pool: &'a PgPool, tenant_id: &str) -> Result<Self, sqlx::Error> {
        let mut transaction = pool.begin().await?;

        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *transaction)
            .await?;

        Ok(Self { transaction })
    }

    pub async fn visible_tenants(&mut self) -> Result<Vec<String>, sqlx::Error> {
        sqlx::query_scalar::<_, String>("SELECT tenant_id FROM awbms_vg04_probe ORDER BY tenant_id")
            .fetch_all(&mut *self.transaction)
            .await
    }

    pub async fn insert_probe(
        &mut self,
        tenant_id: &str,
        payload: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO awbms_vg04_probe (tenant_id, payload) VALUES ($1, $2)")
            .bind(tenant_id)
            .bind(payload)
            .execute(&mut *self.transaction)
            .await?;

        Ok(())
    }

    pub async fn rollback(self) -> Result<(), sqlx::Error> {
        self.transaction.rollback().await
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use axum::{body::Body, http::Request};
    use tokio::{net::TcpListener, sync::oneshot, time::timeout};
    use tower::ServiceExt;

    use super::{serve_verification, verification_router};

    #[tokio::test]
    async fn request_id_is_added_and_propagated() {
        let response = verification_router()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .expect("verification request must be constructible"),
            )
            .await
            .expect("verification router must answer");

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        assert!(response.headers().contains_key("x-request-id"));
    }

    #[tokio::test]
    async fn graceful_shutdown_completes_without_aborting_the_server_task() {
        let listener = TcpListener::bind(("127.0.0.1", 0))
            .await
            .expect("verification listener must bind");
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let server = tokio::spawn(serve_verification(listener, async move {
            let _ = shutdown_rx.await;
        }));

        shutdown_tx
            .send(())
            .expect("verification shutdown signal must be delivered");

        timeout(Duration::from_secs(1), server)
            .await
            .expect("server must complete within the shutdown deadline")
            .expect("server task must not panic")
            .expect("graceful shutdown must complete successfully");
    }
}
