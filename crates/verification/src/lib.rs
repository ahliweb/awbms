//! Stage-1 verification spike for the AWBMS Axum/Tower/Tokio/SQLx assumptions.
//!
//! This crate exists only to turn architecture assumptions into executable
//! checks (HTTP middleware wiring, graceful shutdown, and PostgreSQL
//! row-level-security tenant isolation). It is not a production service and is
//! never published (`publish = false`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

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
///
/// The router serves `GET /health` and applies, from outermost to innermost:
/// request-id generation and propagation (`x-request-id`), a 1 MiB request body
/// limit, and a 5 second request timeout that answers `504 Gateway Timeout`.
///
/// # Examples
///
/// ```
/// let router = awbms_verification::verification_router();
/// # let _ = router;
/// ```
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
///
/// The server stops accepting new connections once `shutdown` completes and
/// then drains in-flight requests before returning.
///
/// # Errors
///
/// Returns the underlying [`std::io::Error`] if the server fails while
/// accepting or serving connections.
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
///
/// The wrapped transaction is rolled back when dropped without an explicit
/// commit, so the spike never persists probe rows.
pub struct TenantTransaction<'a> {
    transaction: Transaction<'a, Postgres>,
}

impl<'a> TenantTransaction<'a> {
    /// Begins a transaction and sets `app.current_tenant_id` for it only
    /// (`set_config(..., is_local => true)`), so the tenant context cannot leak
    /// to other transactions on the same pooled connection.
    ///
    /// # Errors
    ///
    /// Returns [`sqlx::Error`] if a connection cannot be acquired from `pool`,
    /// or if starting the transaction or setting the tenant context fails.
    pub async fn begin(pool: &'a PgPool, tenant_id: &str) -> Result<Self, sqlx::Error> {
        let mut transaction = pool.begin().await?;

        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *transaction)
            .await?;

        Ok(Self { transaction })
    }

    /// Returns the sorted `tenant_id` of every probe row that PostgreSQL RLS
    /// makes visible to the current tenant context.
    ///
    /// # Errors
    ///
    /// Returns [`sqlx::Error`] if the query fails, for example when the probe
    /// table is missing or the role lacks `SELECT` privilege.
    pub async fn visible_tenants(&mut self) -> Result<Vec<String>, sqlx::Error> {
        sqlx::query_scalar::<_, String>("SELECT tenant_id FROM awbms_vg04_probe ORDER BY tenant_id")
            .fetch_all(&mut *self.transaction)
            .await
    }

    /// Inserts a probe row for `tenant_id` with `payload`.
    ///
    /// Whether the insert is allowed is decided by the RLS policy of the
    /// database, not by this method; an insert for a tenant other than the
    /// current context is expected to be rejected.
    ///
    /// # Errors
    ///
    /// Returns [`sqlx::Error`] if the insert fails, including when the RLS
    /// policy rejects the row.
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

    /// Rolls back the transaction, discarding any rows inserted through it.
    ///
    /// # Errors
    ///
    /// Returns [`sqlx::Error`] if the rollback cannot be issued to the
    /// database.
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
