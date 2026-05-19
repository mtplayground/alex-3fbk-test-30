use std::process::ExitCode;

use anyhow::{anyhow, Result};
use tracing_subscriber::EnvFilter;
use zeroclaw_core::{db, Config, ServiceRole};

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("worker failed to start: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<()> {
    init_tracing()?;

    let config = Config::from_env(ServiceRole::Worker)?;
    let pool = db::create_pool(&config).await?;

    db::health_check(&pool).await?;

    tracing::info!(
        service = config.service_name(),
        role = ?config.role(),
        public_base_url = %config.public_base_url(),
        "worker database pool initialized"
    );

    Ok(())
}

fn init_tracing() -> Result<()> {
    let filter = match EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(_) => EnvFilter::new("info"),
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .try_init()
        .map_err(|error| anyhow!("failed to initialize tracing: {error}"))
}
