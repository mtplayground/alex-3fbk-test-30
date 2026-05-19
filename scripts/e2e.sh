#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WEB_DIR="$ROOT_DIR/web"

required_vars=(
  TEST_DATABASE_URL
  REDIS_URL
  S3_ENDPOINT
  S3_BUCKET
  S3_ACCESS_KEY_ID
  S3_SECRET_ACCESS_KEY
  JWT_SECRET
  SMTP_HOST
  SMTP_USERNAME
  SMTP_PASSWORD
  SMTP_FROM
  PUBLIC_BASE_URL
)

require_command() {
  local command_name="$1"

  if ! command -v "$command_name" >/dev/null 2>&1; then
    printf '[e2e] missing required command: %s\n' "$command_name" >&2
    exit 1
  fi
}

require_env() {
  local missing=()

  for name in "${required_vars[@]}"; do
    if [[ -z "${!name:-}" ]]; then
      missing+=("$name")
    fi
  done

  if ((${#missing[@]} > 0)); then
    printf '[e2e] missing required environment variables:\n' >&2
    printf '  %s\n' "${missing[@]}" >&2
    printf '\n[e2e] Use TEST_DATABASE_URL for a disposable PostgreSQL database.\n' >&2
    exit 1
  fi
}

run_process() {
  local name="$1"
  shift

  printf '[e2e] starting %s\n' "$name"
  "$@" &
  pids+=("$!")
}

cleanup() {
  local status=$?

  trap - EXIT INT TERM

  if ((${#pids[@]} > 0)); then
    printf '[e2e] stopping processes\n'
    for pid in "${pids[@]}"; do
      if kill -0 "$pid" >/dev/null 2>&1; then
        kill "$pid" >/dev/null 2>&1 || true
      fi
    done

    wait "${pids[@]}" 2>/dev/null || true
  fi

  exit "$status"
}

main() {
  cd "$ROOT_DIR"

  require_command cargo
  require_command npm
  require_env

  local host="${E2E_HOST:-127.0.0.1}"
  local api_port="${E2E_API_PORT:-8081}"
  local worker_port="${E2E_WORKER_PORT:-0}"
  local web_port="${E2E_WEB_PORT:-8080}"
  local api_base_url="${E2E_API_BASE_URL:-http://${host}:${api_port}}"

  pids=()
  trap cleanup EXIT INT TERM

  run_process api env \
    DATABASE_URL="$TEST_DATABASE_URL" \
    SERVICE_NAME="${API_SERVICE_NAME:-zeroclaw-api-e2e}" \
    HOST="$host" \
    PORT="$api_port" \
    cargo run -p zeroclaw-api

  run_process worker env \
    DATABASE_URL="$TEST_DATABASE_URL" \
    SERVICE_NAME="${WORKER_SERVICE_NAME:-zeroclaw-worker-e2e}" \
    HOST="$host" \
    PORT="$worker_port" \
    cargo run -p zeroclaw-worker

  run_process web bash -c \
    'cd "$1" && VITE_API_BASE_URL="$2" VITE_WS_BASE_URL="$3" npm run dev -- --host "$4" --port "$5"' \
    _ "$WEB_DIR" "$api_base_url" "${api_base_url/http/ws}/ws" "$host" "$web_port"

  wait -n "${pids[@]}"
}

pids=()
main "$@"
