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

The React SPA lives in `web/`:

```bash
cd web
npm install
npm run build
```

## Local Development

Create `.env` from `.env.example`, replace every secret and service URL, then
install the web dependencies and start all local processes:

```bash
cd web
npm install
cd ..
scripts/dev.sh
```

`scripts/dev.sh` loads `.env` when present and requires:

- `DATABASE_URL`
- `REDIS_URL`
- `S3_ENDPOINT`, `S3_BUCKET`, `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY`
- `JWT_SECRET`
- `SMTP_HOST`, `SMTP_USERNAME`, `SMTP_PASSWORD`, `SMTP_FROM`
- `PUBLIC_BASE_URL`

Local ports default to API `8081`, web `8080`, and worker `0`. Override with
`API_PORT`, `WEB_PORT`, `WORKER_PORT`, and `HOST`.

## API

The API binary starts an Axum server on `HOST:PORT` and exposes `GET /healthz`.
Tracing is emitted as JSON through `tracing-subscriber`.

### WebSocket Gateway

Realtime clients connect to `GET /ws` with a valid access token. Browser clients
should send `?token=<access-token>`; non-browser clients may use
`Authorization: Bearer <access-token>`. Optional conversation subscriptions are
passed as comma-separated UUIDs in `conversations`, for example:

```text
/ws?token=<access-token>&conversations=<conversation-id>,<conversation-id>
```

On connect the server validates the JWT, verifies the user still exists, and
subscribes the connection to Redis fan-out channels:

- `user:{user_id}`
- `conversation:{conversation_id}` for every requested conversation

The configured `REDIS_KEY_PREFIX` is applied to all channel names. Redis payloads
are forwarded to the socket as text frames unchanged. The server sends a `ready`
message after subscribing and then sends WebSocket ping frames plus a JSON
`heartbeat` message every 30 seconds. Clients should reconnect with a refreshed
access token and the same `conversations` parameter, using exponential backoff
capped at 10 seconds. Clients may send `{"type":"ping"}` and will receive
`{"type":"pong"}`.

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
