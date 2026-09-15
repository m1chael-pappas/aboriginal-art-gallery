#!/usr/bin/env bash
# Promotes an already-built and tested image set to production.
#
# Usage: scripts/release.sh <build-tag> <release-version> [record-file]
#   scripts/release.sh 42-1a2b3c4d v0.42.0 reports/release.json
#
# 1. tags gallery-{api,web,datadog-agent}:<build-tag> as <release-version>
# 2. deploys the release to gallery-prod and runs the production smoke test
# 3. on any failure, redeploys the release that was live before and exits 1
# 4. on success, records the release on the gallery-prod-releases volume
#    (the old one becomes "previous"), tags it latest and writes a JSON
#    release record for Jenkins to archive
#
# scripts/rollback.sh reads the same record, so rolling back never depends
# on Jenkins history.

source "$(dirname "$0")/lib/common.sh"
source "$(dirname "$0")/lib/deploy.sh"
require docker jq curl

BUILD_TAG="${1:?build tag}"
RELEASE="${2:?release version, e.g. v0.42.0}"
RECORD_FILE="${3:-}"
ENV_FILE="$(prod_env_file)"
PROD_WEB_URL="${PROD_WEB_URL:-http://localhost:8092}"
PROD_API_URL="${PROD_API_URL:-http://localhost:8082}"

load_datadog_env
: "${DD_API_KEY:?DD_API_KEY is required by the production Datadog agent}"

previous="$(current_release)"
log "releasing ${BUILD_TAG} as ${RELEASE}, currently live: ${previous:-nothing}"

tag_images "$BUILD_TAG" "$RELEASE"

restore_previous() {
  if [[ -n "$previous" ]]; then
    log "restoring ${previous}"
    deploy_stack prod "$ENV_FILE" "$previous" --if-empty
  else
    log "no earlier release to restore, stopping the failed release"
    stack_compose prod "$ENV_FILE" "$RELEASE" down
  fi
}

if ! deploy_stack prod "$ENV_FILE" "$RELEASE" --if-empty; then
  restore_previous
  die "deploying ${RELEASE} failed; production is back on ${previous:-nothing}"
fi

if ! SMOKE_SUITE=smoke.production "$REPO_ROOT/scripts/smoke-test.sh" \
  "$PROD_WEB_URL" "$PROD_API_URL" "$ENV_FILE" "$REPO_ROOT/reports/smoke-production.xml"; then
  restore_previous
  die "production smoke test failed for ${RELEASE}; production is back on ${previous:-nothing}"
fi

record_release "$RELEASE" release
tag_images "$RELEASE" latest

if [[ -n "$RECORD_FILE" ]]; then
  mkdir -p "$(dirname "$RECORD_FILE")"
  jq -n --arg release "$RELEASE" --arg build "$BUILD_TAG" --arg previous "$previous" \
    --arg at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    '{release: $release, build_tag: $build, previous_release: $previous, deployed_at: $at, rollback: "scripts/rollback.sh"}' \
    >"$RECORD_FILE"
  release_state >"$(dirname "$RECORD_FILE")/release-history.json"
fi
log "${RELEASE} is live, rollback target is ${previous:-none}"
