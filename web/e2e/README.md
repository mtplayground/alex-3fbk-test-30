# Playwright E2E

The default suite runs the browser app against a mocked API contract:

```bash
cd web
npm run test:e2e
```

For live smoke coverage, provide disposable services and run:

```bash
cd web
ZEROCLAW_E2E_LIVE=1 \
TEST_DATABASE_URL=postgresql://... \
REDIS_URL=redis://... \
S3_ENDPOINT=http://... \
S3_BUCKET=zeroclaw-e2e \
S3_ACCESS_KEY_ID=... \
S3_SECRET_ACCESS_KEY=... \
JWT_SECRET=... \
SMTP_HOST=... \
SMTP_USERNAME=... \
SMTP_PASSWORD=... \
SMTP_FROM=... \
PUBLIC_BASE_URL=http://127.0.0.1:8081 \
npm run test:e2e
```

Live mode starts the API, worker, and Vite web server through `scripts/e2e.sh`.
Use a clean, disposable PostgreSQL database for `TEST_DATABASE_URL`; the harness
intentionally refuses to treat the shared development database as disposable.
