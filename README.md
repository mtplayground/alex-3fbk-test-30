# ZeroClaw

ZeroClaw is a Rust workspace for an Instagram-style social application. The
workspace starts with two binaries and one shared library:

- `crates/api`: HTTP API entry point. The Axum server is added in a later issue.
- `crates/worker`: background worker entry point for durable jobs added later.
- `crates/core`: shared models, configuration types, and common errors.

## Requirements

- Rust 1.75 or newer
- PostgreSQL for persistent state

## Build

```bash
cargo build
```

## Database

Database access is centralized in `zeroclaw-core::db`. The API initializes a
PostgreSQL connection pool, runs embedded SQLx migrations from `migrations/`,
and verifies connectivity with a `SELECT 1` health query during startup. The
worker initializes the same pool and health check path.

## Object Storage

S3-compatible object storage access is centralized in `zeroclaw-core::storage`.
The client is configured with the custom endpoint from `S3_ENDPOINT` and uses
path-style addressing for local and S3-compatible providers.

## Runtime configuration

Both binaries read configuration from environment variables. Copy
`.env.example` for local development and provide real secrets through the
environment in production.

- `DATABASE_URL`: required PostgreSQL connection string
- `REDIS_URL`: required Redis connection string
- `REDIS_KEY_PREFIX`: optional Redis key/channel namespace, defaults to `zeroclaw`
- `S3_ENDPOINT`, `S3_BUCKET`, `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY`: required S3-compatible object storage settings
- `JWT_SECRET`: required signing secret
- `SMTP_HOST`, `SMTP_USERNAME`, `SMTP_PASSWORD`, `SMTP_FROM`: required email settings
- `PUBLIC_BASE_URL`: required externally visible base URL
- `HOST`: optional bind host, defaults to `0.0.0.0`
- `PORT`: optional port, defaults to `8080` for the API and `0` for the worker
- `SERVICE_NAME`: optional service label for logs

Example:

```bash
set -a
. ./.env.example
set +a
cargo run -p zeroclaw-api
```
