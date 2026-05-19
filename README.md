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

## Runtime configuration

Both binaries read configuration from environment variables:

- `DATABASE_URL`: required PostgreSQL connection string
- `HOST`: optional bind host, defaults to `0.0.0.0`
- `PORT`: optional port, defaults to `8080` for the API
- `SERVICE_NAME`: optional service label for logs

Example:

```bash
export DATABASE_URL=postgres://user:password@localhost:5432/zeroclaw
cargo run -p zeroclaw-api
```
