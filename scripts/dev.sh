#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WEB_DIR="$ROOT_DIR/web"

required_vars=(
  DATABASE_URL
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

load_env() {
  if [[ -f "$ROOT_DIR/.env" ]]; then
    set -a
    # shellcheck disable=SC1091
    source "$ROOT_DIR/.env"
    set +a
  fi
}

require_command() {
  local command_name="$1"

  if ! command -v "$command_name" >/dev/null 2>&1; then
    printf 'missing required command: %s\n' "$command_name" >&2
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
    printf 'missing required environment variables:\n' >&2
    printf '  %s\n' "${missing[@]}" >&2
    printf '\nCreate .env from .env.example or export the values before running scripts/dev.sh.\n' >&2
    exit 1
  fi
}

require_web_dependencies() {
  if [[ ! -d "$WEB_DIR/node_modules" ]]; then
    printf 'web dependencies are not installed. Run: cd web && npm install\n' >&2
    exit 1
  fi
}

run_process() {
  local name="$1"
  shift

  printf '[dev] starting %s\n' "$name"
  "$@" &
  pids+=("$!")
}

cleanup() {
  local status=$?

  trap - EXIT INT TERM

  if ((${#pids[@]} > 0)); then
    printf '[dev] stopping processes\n'
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

  load_env
  require_command cargo
  require_command npm
  require_env
  require_web_dependencies

  local host="${HOST:-0.0.0.0}"
  local api_port="${API_PORT:-8081}"
  local worker_port="${WORKER_PORT:-0}"
  local web_port="${WEB_PORT:-8080}"

  pids=()
  trap cleanup EXIT INT TERM

  printf '[dev] api:    http://%s:%s\n' "$host" "$api_port"
  printf '[dev] web:    http://%s:%s\n' "$host" "$web_port"
  printf '[dev] worker: background job process\n'

  run_process api env \
    SERVICE_NAME="${API_SERVICE_NAME:-zeroclaw-api}" \
    HOST="$host" \
    PORT="$api_port" \
    cargo run -p zeroclaw-api

  run_process worker env \
    SERVICE_NAME="${WORKER_SERVICE_NAME:-zeroclaw-worker}" \
    HOST="$host" \
    PORT="$worker_port" \
    cargo run -p zeroclaw-worker

  run_process web bash -c \
    'cd "$1" && npm run dev -- --host "$2" --port "$3"' \
    _ "$WEB_DIR" "$host" "$web_port"

  wait -n "${pids[@]}"
}

pids=()
main "$@"
