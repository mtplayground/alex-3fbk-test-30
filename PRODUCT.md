# ZeroClaw Product Contract

ZeroClaw is an Instagram-style social application. The merged project includes a
Rust backend, a background worker, and a Vite React SPA that together support
accounts, profiles, media posts, comments, social graph, feeds, stories, reels,
direct messages, notifications, moderation, and local/staging seed data.

## Current Capabilities

- Account flows: signup, login, guest/demo login from the login page using the
  seeded `alice@example.test` demo account, refresh/logout, email verification,
  password reset, bearer-token auth, httpOnly refresh cookie, and protected
  frontend routes.
- Profiles: public profile lookup, current-user profile editing, avatar upload
  presign flow, profile grids, follow/unfollow, private-account follow requests,
  followers/following lists, blocks, reports, and admin report review actions.
- Media: S3-compatible presigned uploads, upload completion, media job queue,
  image variants, video HLS processing through `ffmpeg`, and reusable frontend
  uploader/composer controls.
- Posts: create/read/soft-delete posts, carousel media, caption hashtags and
  mentions, comments with one-level replies, likes, saves, user post lists, home
  feed, explore feed, search, and frontend post/feed/detail/explore screens.
- Stories and reels: database schema, APIs, worker TTL cleanup for stories,
  stories rail/viewer UI, reel creation/feed/read APIs, and vertical reels UI
  using `hls.js`.
- Messaging and realtime: conversation/message schema, DM REST APIs, WebSocket
  gateway with JWT auth, Redis fan-out, typing/read/presence events, frontend
  realtime hooks, inbox/thread UI, and notification drawer/badges.
- Operations and tests: local dev orchestration, Playwright E2E setup, Rust unit
  tests, deployment runbook, and an idempotent seed command for demo users and
  content.

## Architecture

- Rust workspace with `crates/api` for Axum HTTP/WebSocket serving,
  `crates/worker` for background jobs, and `crates/core` for shared config,
  models, auth, database, Redis, storage, and repositories.
- The API service also serves the built React app from `web/dist`, including
  SPA fallback routing for browser routes such as `/`, `/login`, and `/p/:id`.
- PostgreSQL is the only persistent datastore. Schema changes live in
  `migrations/` and are wired through embedded SQLx migrations.
- Redis is used for cache, rate limiting, presence, and realtime fan-out.
- S3-compatible object storage holds media originals and variants.
- The SPA lives in `web/` and uses React, TypeScript, Vite, Tailwind, TanStack
  Query, Zustand, React Router, and Playwright.

## Runtime Conventions

- Services read configuration from environment variables; `.env.example`
  documents required values.
- API defaults to `0.0.0.0:8080`; local dev scripts may override ports.
- Backend logging uses JSON tracing.
- Access tokens are short-lived JWTs; refresh tokens are stored and rotated in
  PostgreSQL.
- Write endpoints are guarded by Redis-backed rate limiting; login is throttled
  by IP and email.
- The worker claims jobs with Postgres `FOR UPDATE SKIP LOCKED` semantics and
  also listens for job notifications.

## Repository Conventions

- Do not hardcode service URLs or secrets; use environment variables.
- Persistent state must remain PostgreSQL-backed.
- Keep API behavior in `crates/api`, durable domain/storage helpers in
  `crates/core`, and asynchronous media/cleanup work in `crates/worker`.
- Use the existing repository modules and model types before adding raw SQL.
- Frontend routes live under `web/src/routes`, reusable feature code under
  `web/src/features`, and shell/provider wiring under `web/src/app`.
