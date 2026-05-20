# Yet another Instagram Deployment Runbook

This runbook describes a bare self-hosted deployment for one Linux host running
PostgreSQL, Redis, the Rust API, the Rust worker, and the built React SPA.

## Prerequisites

- A Linux host with systemd.
- Rust 1.75 or newer and a C compiler toolchain.
- Node.js 20 or newer for building the SPA.
- PostgreSQL 16 and Redis 7.
- S3-compatible object storage.
- SMTP credentials for auth email flows.
- A reverse proxy such as nginx or Caddy with TLS.

Yet another Instagram stores persistent application state only in PostgreSQL. Redis is used
for cache, rate limits, presence, and realtime fan-out.

## Build

Clone the repository and build release binaries:

```bash
git clone https://github.com/mtplayground/alex-3fbk-test-30.git /opt/zeroclaw
cd /opt/zeroclaw
cargo build --release
cd web
npm ci
npm run build
```

The release binaries are:

- `/opt/zeroclaw/target/release/api`
- `/opt/zeroclaw/target/release/worker`

The SPA build output is `/opt/zeroclaw/web/dist`.

## Environment

Create an environment file readable by the service user:

```bash
sudo install -d -o zeroclaw -g zeroclaw /etc/zeroclaw
sudo install -m 0600 -o zeroclaw -g zeroclaw /dev/null /etc/zeroclaw/zeroclaw.env
```

Example `/etc/zeroclaw/zeroclaw.env`:

```dotenv
SERVICE_NAME=zeroclaw-api
HOST=127.0.0.1
PORT=8080
PUBLIC_BASE_URL=https://example.com
RUST_LOG=info

DATABASE_URL=postgres://zeroclaw:replace-me@127.0.0.1:5432/zeroclaw

REDIS_URL=redis://127.0.0.1:6379
REDIS_KEY_PREFIX=zeroclaw

S3_ENDPOINT=https://s3.example.com
S3_BUCKET=zeroclaw
S3_ACCESS_KEY_ID=replace-me
S3_SECRET_ACCESS_KEY=replace-me
S3_REGION=us-east-1

JWT_SECRET=replace-with-at-least-32-random-bytes

SMTP_HOST=smtp.example.com
SMTP_PORT=587
SMTP_USERNAME=replace-me
SMTP_PASSWORD=replace-me
SMTP_FROM=no-reply@example.com
```

Use a direct PostgreSQL URL for the worker if PgBouncer prevents
`LISTEN/NOTIFY` from working. Do not commit this file.

## Database

Create the database and user:

```bash
sudo -u postgres createuser --pwprompt zeroclaw
sudo -u postgres createdb --owner=zeroclaw zeroclaw
```

The API and worker run embedded SQLx migrations at startup. To verify manually:

```bash
set -a
. /etc/zeroclaw/zeroclaw.env
set +a
cargo run --release -p zeroclaw-api --bin api
```

Stop the foreground process after the migration log confirms startup.

## Systemd Units

Create `/etc/systemd/system/zeroclaw-api.service`:

```ini
[Unit]
Description=Yet another Instagram API
After=network-online.target postgresql.service redis-server.service
Wants=network-online.target

[Service]
Type=simple
User=zeroclaw
Group=zeroclaw
WorkingDirectory=/opt/zeroclaw
EnvironmentFile=/etc/zeroclaw/zeroclaw.env
ExecStart=/opt/zeroclaw/target/release/api
Restart=always
RestartSec=5
NoNewPrivileges=true
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

Create `/etc/systemd/system/zeroclaw-worker.service`:

```ini
[Unit]
Description=Yet another Instagram Worker
After=network-online.target postgresql.service redis-server.service
Wants=network-online.target

[Service]
Type=simple
User=zeroclaw
Group=zeroclaw
WorkingDirectory=/opt/zeroclaw
EnvironmentFile=/etc/zeroclaw/zeroclaw.env
Environment=SERVICE_NAME=zeroclaw-worker
ExecStart=/opt/zeroclaw/target/release/worker
Restart=always
RestartSec=5
NoNewPrivileges=true
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now zeroclaw-api zeroclaw-worker
sudo systemctl status zeroclaw-api zeroclaw-worker
```

View logs:

```bash
journalctl -u zeroclaw-api -f
journalctl -u zeroclaw-worker -f
```

## Reverse Proxy

Terminate TLS at the proxy, serve the SPA from `web/dist`, and proxy API and
WebSocket traffic to the API service.

Minimal nginx shape:

```nginx
server {
    listen 443 ssl http2;
    server_name example.com;

    root /opt/zeroclaw/web/dist;
    index index.html;

    client_max_body_size 50m;

    location /api/ {
        proxy_pass http://127.0.0.1:8080/;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /ws {
        proxy_pass http://127.0.0.1:8080/ws;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_read_timeout 75s;
    }

    location / {
        try_files $uri $uri/ /index.html;
    }
}
```

If the frontend is configured to call the API without an `/api` prefix, proxy
the relevant API paths directly instead of using the `/api/` location.

## Seed Data

The seed command creates demo users, media rows, posts, comments, follows,
stories, a DM conversation, and sample notifications. It is idempotent: once the
seed sentinel post exists, it leaves existing seed content untouched.

Run only against local or staging databases:

```bash
set -a
. /etc/zeroclaw/zeroclaw.env
set +a
ZEROCLAW_SEED_CONFIRM=1 cargo run --release -p zeroclaw-api --bin seed
```

Demo credentials:

- `alice@example.test` / `password123`
- `bob@example.test` / `password123`
- `mira@example.test` / `password123`
- `admin@example.test` / `password123`

The seed media rows reference deterministic S3 keys under `seed/`. Upload real
objects to those keys if the target environment needs fully rendered media.

## Operations

Health check:

```bash
curl -fsS https://example.com/healthz
```

Roll forward:

```bash
cd /opt/zeroclaw
git pull --ff-only
cargo build --release
cd web && npm ci && npm run build
sudo systemctl restart zeroclaw-api zeroclaw-worker
```

Roll back by checking out the previous known-good commit, rebuilding, and
restarting both services. Keep database backups before deploying migrations.
