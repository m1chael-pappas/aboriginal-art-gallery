#!/usr/bin/env bash
# Rolls production back to the previous release, or to a named one.
#
# Usage: scripts/rollback.sh [release-version]
#   scripts/rollback.sh            # back to the release before the current one
#   scripts/rollback.sh v0.40.0    # to a specific release still on this host
#
# Reads the release record that release.sh keeps on the gallery-prod-releases
# volume, redeploys, smoke tests, then records the rollback so running it
# twice flips back again. Reads .env.prod and the Datadog keys from
# .env.jenkins when they are not already exported.

source "$(dirname "$0")/lib/common.sh"
source "$(dirname "$0")/lib/deploy.sh"
source "$(dirname "$0")/lib/datadog.sh"
require docker jq curl

ENV_FILE="$(prod_env_file)"
PROD_WEB_URL="${PROD_WEB_URL:-http://localhost:8092}"
PROD_API_URL="${PROD_API_URL:-http://localhost:8082}"

load_datadog_env
: "${DD_API_KEY:?DD_API_KEY is required by the production Datadog agent}"

current="$(current_release)"
target="${1:-$(previous_release)}"

[[ -n "$target" ]] || die "no previous release recorded yet"
docker image inspect "gallery-api:${target}" >/dev/null 2>&1 || die "release ${target} is not on this host"
[[ "$target" != "$current" ]] || die "${target} is already live"

log "rolling production back from ${current:-nothing} to ${target}"
deploy_stack prod "$ENV_FILE" "$target" --if-empty

SMOKE_SUITE=smoke.rollback "$REPO_ROOT/scripts/smoke-test.sh" "$PROD_WEB_URL" "$PROD_API_URL" "$ENV_FILE" \
  || die "smoke test failed after rolling back to ${target}"

record_release "$target" rollback
tag_images "$target" latest

if [[ -n "${DD_APP_KEY:-}" ]]; then
  dd_event "Rolled back gallery-prod to ${target}" \
    "Production rolled back from ${current:-unknown} to ${target} with scripts/rollback.sh." warning "release:${target}" \
    || log "could not post the Datadog event, continuing"
fi
log "production is on ${target}; run scripts/rollback.sh again to return to ${current:-the previous release}"
