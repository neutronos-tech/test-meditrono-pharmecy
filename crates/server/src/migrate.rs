//! Applies `migrations/postgres` at start-up, each file once, in name order.
//! Applied files are recorded in `pharmacy_schema_migrations`. Production
//! applied the first file by hand before this runner existed, so an existing
//! `pharmacies` table marks that file as already applied.

use sqlx_core::{query::query, raw_sql::raw_sql, row::Row};
use sqlx_postgres::PgPool;

const MIGRATIONS: &[(&str, &str)] = &[
    (
        "20261009_pharmacy_auth_and_sync",
        include_str!("../../../migrations/postgres/20261009_pharmacy_auth_and_sync.sql"),
    ),
    (
        "20261015_devices_plans_licences",
        include_str!("../../../migrations/postgres/20261015_devices_plans_licences.sql"),
    ),
];

const BASELINE: &str = "20261009_pharmacy_auth_and_sync";

pub async fn run(pool: &PgPool) -> Result<(), sqlx_core::error::Error> {
    // One server at a time: a second instance waits here until the first finishes.
    let mut lock = pool.acquire().await?;
    query("SELECT pg_advisory_lock(7242001)")
        .execute(&mut *lock)
        .await?;
    let result = apply(&mut lock).await;
    if result.is_err() {
        // A failed file leaves its transaction open; close it so the unlock
        // below runs and the original error is the one reported.
        let _ = raw_sql("ROLLBACK").execute(&mut *lock).await;
    }
    query("SELECT pg_advisory_unlock(7242001)")
        .execute(&mut *lock)
        .await?;
    result
}

async fn apply(conn: &mut sqlx_postgres::PgConnection) -> Result<(), sqlx_core::error::Error> {
    query(
        "CREATE TABLE IF NOT EXISTS pharmacy_schema_migrations (\
            name TEXT PRIMARY KEY, applied_at TIMESTAMPTZ NOT NULL DEFAULT now())",
    )
    .execute(&mut *conn)
    .await?;
    let has_baseline_tables: bool =
        query("SELECT to_regclass('public.pharmacies') IS NOT NULL AS present")
            .fetch_one(&mut *conn)
            .await?
            .try_get("present")?;
    if has_baseline_tables {
        query("INSERT INTO pharmacy_schema_migrations(name) VALUES($1) ON CONFLICT DO NOTHING")
            .bind(BASELINE)
            .execute(&mut *conn)
            .await?;
    }
    for (name, sql) in MIGRATIONS {
        let applied: bool =
            query("SELECT EXISTS(SELECT 1 FROM pharmacy_schema_migrations WHERE name=$1) AS done")
                .bind(name)
                .fetch_one(&mut *conn)
                .await?
                .try_get("done")?;
        if applied {
            continue;
        }
        tracing::info!(migration = name, "applying database migration");
        // Each file wraps itself in BEGIN/COMMIT.
        raw_sql(sql).execute(&mut *conn).await?;
        query("INSERT INTO pharmacy_schema_migrations(name) VALUES($1)")
            .bind(name)
            .execute(&mut *conn)
            .await?;
    }
    Ok(())
}
