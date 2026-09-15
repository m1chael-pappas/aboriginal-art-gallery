#!/usr/bin/env bash
# Shared helpers sourced by the scripts in scripts/.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
export REPO_ROOT

log() { printf '[%s] %s\n' "$(date +%H:%M:%S)" "$*" >&2; }
die() { log "ERROR: $*"; exit 1; }

require() {
  local cmd
  for cmd in "$@"; do
    command -v "$cmd" >/dev/null 2>&1 || die "missing required command: $cmd"
  done
}

# wait_until <timeout-seconds> <description> <command...>
# Re-runs the command every 3 seconds until it succeeds or the timeout passes.
wait_until() {
  local timeout="$1" what="$2"
  shift 2
  local deadline=$((SECONDS + timeout))
  until "$@" >/dev/null 2>&1; do
    ((SECONDS < deadline)) || die "timed out after ${timeout}s waiting for ${what}"
    sleep 3
  done
  log "ready: ${what}"
}

# env_set <file> <KEY> <value>: replaces KEY=... in an env file, or appends it.
env_set() {
  local file="$1" key="$2" value="$3" tmp
  tmp="$(mktemp)"
  grep -v "^${key}=" "$file" >"$tmp" || true
  printf '%s=%s\n' "$key" "$value" >>"$tmp"
  cat "$tmp" >"$file"
  rm -f "$tmp"
}

# env_get <file> <KEY>: prints the value of KEY from an env file, without
# surrounding whitespace or quotes, the same way docker compose reads it.
env_get() {
  grep -E "^$2=" "$1" | tail -n1 | cut -d= -f2- \
    | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//; s/^"(.*)"$/\1/; s/^'"'"'(.*)'"'"'$/\1/'
}

# gen_password: 128 random bits as hex, plus a fixed suffix so the result
# also passes SonarQube's rule of upper, lower, digit and special characters.
gen_password() {
  printf '%sXq7-\n' "$(openssl rand -hex 16)"
}
