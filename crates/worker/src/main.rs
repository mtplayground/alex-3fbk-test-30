use std::process::ExitCode;

use anyhow::{anyhow, Result};
use tracing_subscriber::EnvFilter;
use zeroclaw_core::{Config, ServiceRole};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("worker failed to start: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    init_tracing()?;

    let config = Config::from_env(ServiceRole::Worker)?;

    tracing::info!(
        service = config.service_name(),
        role = ?config.role(),
        public_base_url = %config.public_base_url(),
        "worker crate initialized"
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
