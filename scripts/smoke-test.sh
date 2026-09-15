#!/usr/bin/env bash
# End-to-end smoke test against a deployed stack.
#
# Usage: scripts/smoke-test.sh <web-url> <api-url> <env-file> [junit-output]
#   scripts/smoke-test.sh http://localhost:8091 http://localhost:8081 .env.staging
#
# Checks the API health probe and metrics, the SPA and its /api proxy, the
# public catalogue, rejection of anonymous admin calls, and an admin sign-in
# followed by authenticated calls. Exits non-zero if any check fails and
# optionally writes the results as JUnit XML under the suite name in
# SMOKE_SUITE (default "smoke").

source "$(dirname "$0")/lib/common.sh"
require curl jq

WEB_URL="${1:?web url}"
API_URL="${2:?api url}"
ENV_FILE="${3:?env file with SEED_ADMIN_EMAIL and SEED_ADMIN_PASSWORD}"
JUNIT_OUT="${4:-}"
SUITE="${SMOKE_SUITE:-smoke}"

ADMIN_EMAIL="$(env_get "$ENV_FILE" SEED_ADMIN_EMAIL)"
ADMIN_PASSWORD="$(env_get "$ENV_FILE" SEED_ADMIN_PASSWORD)"
[[ -n "$ADMIN_EMAIL" && -n "$ADMIN_PASSWORD" ]] || die "SEED_ADMIN_EMAIL or SEED_ADMIN_PASSWORD missing in $ENV_FILE"

declare -a RESULTS=()
FAILURES=0
TOKEN=""
BODY="$(mktemp)"
trap 'rm -f "$BODY"' EXIT

# http <method> <url> [curl args...]: prints the status code, body goes to $BODY.
http() {
  local method="$1" url="$2"
  shift 2
  curl -sS -o "$BODY" -w '%{http_code}' --max-time 10 -X "$method" "$url" "$@" || printf '000'
}

# check <name> <command...>: runs one check and records pass or fail.
check() {
  local name="$1" started=$SECONDS
  shift
  if "$@"; then
    log "PASS  ${name}"
    RESULTS+=("pass|${name}|$((SECONDS - started))|")
  else
    log "FAIL  ${name}: $(head -c 300 "$BODY" 2>/dev/null)"
    RESULTS+=("fail|${name}|$((SECONDS - started))|$(head -c 300 "$BODY" | tr -d '\n<>&"')")
    FAILURES=$((FAILURES + 1))
  fi
}

api_health() {
  [[ "$(http GET "$API_URL/health")" == 200 ]] && jq -e '.status == "ok" and .db == "ok"' "$BODY" >/dev/null
}

api_metrics() {
  [[ "$(http GET "$API_URL/metrics")" == 200 ]] && grep -q '^# TYPE axum_http_requests_total counter' "$BODY"
}

spa_index() {
  [[ "$(http GET "$WEB_URL/")" == 200 ]] && grep -q '<div id="app">' "$BODY"
}

spa_deep_link() {
  [[ "$(http GET "$WEB_URL/artists")" == 200 ]] && grep -q '<div id="app">' "$BODY"
}

proxy_health() {
  [[ "$(http GET "$WEB_URL/api/health")" == 200 ]] && jq -e '.db == "ok"' "$BODY" >/dev/null
}

public_catalogue() {
  [[ "$(http GET "$WEB_URL/api/artists")" == 200 ]] && jq -e 'type == "array" and length > 0' "$BODY" >/dev/null
}

anonymous_admin_rejected() {
  [[ "$(http GET "$WEB_URL/api/users")" == 401 ]]
}

admin_login() {
  local payload
  payload="$(jq -n --arg e "$ADMIN_EMAIL" --arg p "$ADMIN_PASSWORD" '{email: $e, password: $p}')"
  [[ "$(http POST "$WEB_URL/api/auth/login" -H 'Content-Type: application/json' --data-binary "$payload")" == 200 ]] || return 1
  TOKEN="$(jq -r '.token // empty' "$BODY")"
  [[ -n "$TOKEN" ]]
}

authenticated_me() {
  [[ "$(http GET "$WEB_URL/api/auth/me" -H "Authorization: Bearer $TOKEN")" == 200 ]] \
    && jq -e --arg e "$ADMIN_EMAIL" '.email == $e and .role == "Admin"' "$BODY" >/dev/null
}

authenticated_admin_list() {
  [[ "$(http GET "$WEB_URL/api/users" -H "Authorization: Bearer $TOKEN")" == 200 ]] \
    && jq -e 'type == "array" and length > 0' "$BODY" >/dev/null
}

write_junit() {
  local out="$1" entry status name duration message
  mkdir -p "$(dirname "$out")"
  {
    printf '<?xml version="1.0" encoding="UTF-8"?>\n'
    printf '<testsuite name="%s" tests="%d" failures="%d">\n' "$SUITE" "${#RESULTS[@]}" "$FAILURES"
    for entry in "${RESULTS[@]}"; do
      IFS='|' read -r status name duration message <<<"$entry"
      printf '  <testcase classname="%s" name="%s" time="%s">' "$SUITE" "$name" "$duration"
      [[ "$status" == fail ]] && printf '<failure message="%s"/>' "$message"
      printf '</testcase>\n'
    done
    printf '</testsuite>\n'
  } >"$out"
}

log "smoke testing web=${WEB_URL} api=${API_URL}"
wait_until 120 "API health at ${API_URL}" api_health

check "API /health reports ok with database" api_health
check "SPA index is served" spa_index
check "SPA deep link falls back to index.html" spa_deep_link
check "nginx proxies /api/health to the API" proxy_health
check "public catalogue lists seeded artists" public_catalogue
check "admin endpoint rejects anonymous callers with 401" anonymous_admin_rejected
check "seeded admin can sign in" admin_login
check "GET /auth/me with the token returns the admin" authenticated_me
check "GET /users with the admin token succeeds" authenticated_admin_list
check "API /metrics counted the requests above" api_metrics

[[ -n "$JUNIT_OUT" ]] && write_junit "$JUNIT_OUT"

if ((FAILURES > 0)); then
  die "${FAILURES} of ${#RESULTS[@]} smoke checks failed"
fi
log "all ${#RESULTS[@]} smoke checks passed"
