use std::process::ExitCode;

use tracing_subscriber::EnvFilter;
use zeroclaw_core::storage::ObjectStorage;
use zeroclaw_core::{db, redis::RedisClient, Config, ServiceRole};

mod auth;
mod comments;
mod email;
mod error;
pub mod extractors;
mod health;
mod http;
mod media;
mod posts;
mod profile;
mod social;
mod state;

use crate::error::AppError;
use crate::state::AppState;

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("api failed to start: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), AppError> {
    init_tracing()?;

    let config = Config::from_env(ServiceRole::Api)?;
    let bind_address = config.bind_address();

    let pool = db::create_pool(&config).await?;
    db::run_migrations(&pool).await?;

    let redis_client = RedisClient::new(&config)?;
    let redis_manager = redis_client.connection_manager().await?;

    let state = AppState::new(
        pool,
        redis_manager,
        redis_client.namespace().clone(),
        config.jwt().clone(),
        config.smtp().clone(),
        config.public_base_url().to_owned(),
        ObjectStorage::new(config.s3()),
    );
    social::spawn_count_reconciliation(state.clone());
    let router = http::router(state);
    let listener = tokio::net::TcpListener::bind(&bind_address).await?;

    tracing::info!(
        service = config.service_name(),
        bind_address = %bind_address,
        public_base_url = %config.public_base_url(),
        "api listening"
    );

    axum::serve(listener, router).await?;

    Ok(())
}

fn init_tracing() -> Result<(), AppError> {
    let filter = match EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(_) => EnvFilter::new("info"),
    };

    tracing_subscriber::fmt()
        .json()
        .with_env_filter(filter)
        .try_init()
        .map_err(|error| AppError::Tracing(error.to_string()))
}
