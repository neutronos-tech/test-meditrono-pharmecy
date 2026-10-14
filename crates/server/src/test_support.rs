//! Real-Postgres test databases. Every test gets a fresh database built from
//! the production pharmacy migration plus a stand-in for meditrino's catalog
//! tables. `DATABASE_URL` must point at a server whose role may CREATE DATABASE.
//! A test that fails leaves its database behind for inspection, as sqlx does.

use crate::app::AppState;
use sqlx_core::raw_sql::raw_sql;
use sqlx_postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::str::FromStr;
use uuid::Uuid;

const CATALOG_STANDIN: &str = include_str!("../tests/fixtures/medicine_catalog.sql");
const PHARMACY_MIGRATION: &str =
    include_str!("../../../migrations/postgres/20261009_pharmacy_auth_and_sync.sql");
const LICENCE_MIGRATION: &str =
    include_str!("../../../migrations/postgres/20261015_devices_plans_licences.sql");

pub struct TestDb {
    pub app: AppState,
    name: String,
    admin: PgPool,
}

pub async fn fresh_db() -> TestDb {
    let url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must point at a Postgres server that may create databases");
    let base = PgConnectOptions::from_str(&url).expect("DATABASE_URL must be a valid URL");
    let admin = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(base.clone().database("postgres"))
        .await
        .expect("connect to the postgres maintenance database");
    let name = format!("pharmacy_test_{}", Uuid::now_v7().simple());
    raw_sql(&format!(r#"CREATE DATABASE "{name}""#))
        .execute(&admin)
        .await
        .expect("create test database");
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect_with(base.database(&name))
        .await
        .expect("connect to the test database");
    raw_sql(CATALOG_STANDIN)
        .execute(&pool)
        .await
        .expect("apply catalog stand-in");
    raw_sql(PHARMACY_MIGRATION)
        .execute(&pool)
        .await
        .expect("apply pharmacy migration");
    raw_sql(LICENCE_MIGRATION)
        .execute(&pool)
        .await
        .expect("apply licence migration");
    let mut app = AppState::from_pool(pool);
    app.licence_key = Some(std::sync::Arc::new(crate::licence::LicenceKey::from_seed(
        1, [9; 32],
    )));
    TestDb { app, name, admin }
}

impl TestDb {
    /// Drops the database. Called only when a test has passed.
    pub async fn drop_db(self) {
        self.app.pool.close().await;
        raw_sql(&format!(r#"DROP DATABASE "{}" WITH (FORCE)"#, self.name))
            .execute(&self.admin)
            .await
            .expect("drop test database");
    }
}
