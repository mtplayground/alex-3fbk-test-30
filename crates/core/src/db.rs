use std::time::Duration;

use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

use crate::Config;

pub static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

const DEFAULT_MAX_CONNECTIONS: u32 = 5;
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

pub async fn create_pool(config: &Config) -> sqlx::Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(DEFAULT_MAX_CONNECTIONS)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .connect(config.database_url())
        .await
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    MIGRATOR.run(pool).await
}

pub async fn health_check(pool: &PgPool) -> sqlx::Result<()> {
    let row = sqlx::query("SELECT 1 AS health_check")
        .fetch_one(pool)
        .await?;
    let value: i32 = row.try_get("health_check")?;

    if value == 1 {
        return Ok(());
    }

    Err(sqlx::Error::RowNotFound)
}
