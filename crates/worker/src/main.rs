use std::process::ExitCode;
use std::time::Duration as StdDuration;

use anyhow::{anyhow, Result};
use chrono::{Duration as ChronoDuration, Utc};
use sqlx::postgres::PgListener;
use sqlx::PgPool;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;
use zeroclaw_core::models::{MediaJob, MediaJobKind};
use zeroclaw_core::repositories::media;
use zeroclaw_core::{db, Config, ServiceRole};

const MEDIA_JOBS_CHANNEL: &str = "media_jobs";
const IDLE_POLL_INTERVAL: StdDuration = StdDuration::from_secs(10);
const MAX_JOBS_PER_WAKE: usize = 25;

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("worker failed: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<()> {
    init_tracing()?;

    let config = Config::from_env(ServiceRole::Worker)?;
    let pool = db::create_pool(&config).await?;
    db::health_check(&pool).await?;

    let worker_id = format!("worker-{}", Uuid::new_v4());
    tracing::info!(
        service = config.service_name(),
        role = ?config.role(),
        worker_id = %worker_id,
        public_base_url = %config.public_base_url(),
        "worker starting media job loop"
    );

    let listener = create_listener(config.database_url()).await?;
    let worker = Worker::new(pool, worker_id, listener);
    worker.run().await
}

async fn create_listener(database_url: &str) -> Result<PgListener> {
    let mut listener = PgListener::connect(database_url).await?;
    listener.listen(MEDIA_JOBS_CHANNEL).await?;
    Ok(listener)
}

struct Worker {
    pool: PgPool,
    worker_id: String,
    listener: PgListener,
}

impl Worker {
    fn new(pool: PgPool, worker_id: String, listener: PgListener) -> Self {
        Self {
            pool,
            worker_id,
            listener,
        }
    }

    async fn run(mut self) -> Result<()> {
        let shutdown = tokio::signal::ctrl_c();
        tokio::pin!(shutdown);

        loop {
            self.drain_available_jobs().await?;

            tokio::select! {
                result = self.listener.recv() => {
                    let notification = result?;
                    tracing::debug!(
                        channel = notification.channel(),
                        payload = notification.payload(),
                        "received media job notification"
                    );
                }
                _ = tokio::time::sleep(IDLE_POLL_INTERVAL) => {
                    tracing::debug!("polling media job queue");
                }
                _ = &mut shutdown => {
                    tracing::info!("worker shutdown signal received");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn drain_available_jobs(&self) -> Result<()> {
        for _ in 0..MAX_JOBS_PER_WAKE {
            let Some(job) = media::claim_next_job(&self.pool, &self.worker_id).await? else {
                return Ok(());
            };

            self.process_job(job).await?;
        }

        Ok(())
    }

    async fn process_job(&self, job: MediaJob) -> Result<()> {
        tracing::info!(
            job_id = %job.id(),
            asset_id = %job.asset_id(),
            kind = ?job.kind(),
            attempt = job.attempts(),
            max_attempts = job.max_attempts(),
            "claimed media job"
        );

        match dispatch_job(&job).await {
            Ok(()) => {
                media::mark_job_succeeded(&self.pool, job.id()).await?;
                tracing::info!(job_id = %job.id(), "media job succeeded");
            }
            Err(error) => {
                let message = error.to_string();
                if job.attempts() >= job.max_attempts() {
                    media::mark_job_failed(&self.pool, job.id(), &message).await?;
                    tracing::error!(
                        job_id = %job.id(),
                        error = %message,
                        "media job exhausted retry attempts"
                    );
                } else {
                    let run_after = Utc::now() + retry_backoff(job.attempts());
                    media::retry_job(&self.pool, job.id(), run_after, &message).await?;
                    tracing::warn!(
                        job_id = %job.id(),
                        error = %message,
                        run_after = %run_after,
                        "media job scheduled for retry"
                    );
                }
            }
        }

        Ok(())
    }
}

async fn dispatch_job(job: &MediaJob) -> Result<()> {
    match job.kind() {
        MediaJobKind::ImageProcessing => handle_image_processing(job).await,
        MediaJobKind::VideoProcessing => handle_video_processing(job).await,
    }
}

async fn handle_image_processing(job: &MediaJob) -> Result<()> {
    validate_processing_payload(job)
}

async fn handle_video_processing(job: &MediaJob) -> Result<()> {
    validate_processing_payload(job)
}

fn validate_processing_payload(job: &MediaJob) -> Result<()> {
    let payload = job
        .payload()
        .as_object()
        .ok_or_else(|| anyhow!("media job payload must be a JSON object"))?;

    for field in ["asset_id", "owner_id", "kind", "original_key"] {
        let value = payload
            .get(field)
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty());

        if value.is_none() {
            return Err(anyhow!("media job payload is missing {field}"));
        }
    }

    Ok(())
}

fn retry_backoff(attempt: i32) -> ChronoDuration {
    let exponent = attempt.saturating_sub(1).clamp(0, 5);
    let seconds = 30_i64.saturating_mul(1_i64 << exponent);
    ChronoDuration::seconds(seconds)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_backoff_grows_from_claim_attempt() {
        assert_eq!(retry_backoff(1), ChronoDuration::seconds(30));
        assert_eq!(retry_backoff(2), ChronoDuration::seconds(60));
        assert_eq!(retry_backoff(6), ChronoDuration::seconds(960));
        assert_eq!(retry_backoff(10), ChronoDuration::seconds(960));
    }
}
