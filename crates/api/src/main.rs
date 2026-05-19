use std::process::ExitCode;

use anyhow::{anyhow, Result};
use tracing_subscriber::EnvFilter;
use zeroclaw_core::{AppConfig, ServiceRole};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("api failed to start: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    init_tracing()?;

    let config = AppConfig::from_env(ServiceRole::Api)?;

    tracing::info!(
        service = config.service_name(),
        role = ?config.role(),
        bind_address = %config.bind_address(),
        "api crate initialized"
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
