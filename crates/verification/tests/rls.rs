use std::{env, error::Error};

use awbms_verification::TenantTransaction;
use sqlx::{postgres::PgPoolOptions, PgPool};

#[tokio::test]
async fn postgres_force_rls_is_real_and_tenant_context_does_not_leak() -> Result<(), Box<dyn Error>> {
    let database_url = match env::var("AWBMS_TEST_DATABASE_URL") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("AWBMS_TEST_DATABASE_URL is not set; skipping PostgreSQL verification");
            return Ok(());
        }
    };

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&database_url)
        .await?;

    assert_app_role_is_least_privilege(&pool).await?;

    let visible_without_context: Vec<String> = sqlx::query_scalar(
        "SELECT tenant_id FROM awbms_vg04_probe ORDER BY tenant_id",
    )
    .fetch_all(&pool)
    .await?;
    assert!(
        visible_without_context.is_empty(),
        "missing tenant context must fail closed"
    );

    let mut tenant_a = TenantTransaction::begin(&pool, "tenant-a").await?;
    assert_eq!(tenant_a.visible_tenants().await?, vec!["tenant-a"]);

    let cross_tenant_error = tenant_a
        .insert_probe("tenant-b", "must-be-denied")
        .await
        .expect_err("RLS WITH CHECK must reject a cross-tenant insert");

    let sql_state = cross_tenant_error
        .as_database_error()
        .and_then(|error| error.code())
        .map(|code| code.to_string());
    assert_eq!(sql_state.as_deref(), Some("42501"));

    tenant_a.rollback().await?;

    let visible_after_rollback: Vec<String> = sqlx::query_scalar(
        "SELECT tenant_id FROM awbms_vg04_probe ORDER BY tenant_id",
    )
    .fetch_all(&pool)
    .await?;
    assert!(
        visible_after_rollback.is_empty(),
        "SET LOCAL tenant context must not leak back into the pool"
    );

    let mut tenant_b = TenantTransaction::begin(&pool, "tenant-b").await?;
    assert_eq!(tenant_b.visible_tenants().await?, vec!["tenant-b"]);
    tenant_b.rollback().await?;

    Ok(())
}

async fn assert_app_role_is_least_privilege(pool: &PgPool) -> Result<(), sqlx::Error> {
    let (is_superuser, bypasses_rls): (bool, bool) = sqlx::query_as(
        "SELECT rolsuper, rolbypassrls FROM pg_roles WHERE rolname = current_user",
    )
    .fetch_one(pool)
    .await?;

    assert!(!is_superuser, "application role must not be superuser");
    assert!(!bypasses_rls, "application role must not BYPASSRLS");

    let is_table_owner: bool = sqlx::query_scalar(
        "SELECT pg_get_userbyid(relowner) = current_user FROM pg_class WHERE relname = 'awbms_vg04_probe' AND relkind = 'r'",
    )
    .fetch_one(pool)
    .await?;

    assert!(!is_table_owner, "application role must not own business tables");

    Ok(())
}
