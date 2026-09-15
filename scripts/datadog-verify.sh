#!/usr/bin/env bash
# Confirms Datadog is receiving production data after a release, lists the
# pipeline-managed monitors with their state, and posts a release event.
#
# Usage: scripts/datadog-verify.sh <release-version>
# Env: DD_API_KEY, DD_APP_KEY, DD_SITE (optional).

source "$(dirname "$0")/lib/common.sh"
source "$(dirname "$0")/lib/datadog.sh"
require curl jq

RELEASE="${1:?release version}"
EXPECTED_MONITORS=$(find "$REPO_ROOT/monitoring/datadog/monitors" -name '*.json' | wc -l)

# has_recent_data <query>: true when the query returned a point in the last 3 minutes.
has_recent_data() {
  local now from latest
  now=$(date +%s)
  from=$((now - 600))
  latest="$(dd GET "/api/v1/query?from=${from}&to=${now}&query=$(jq -rn --arg q "$1" '$q|@uri')" \
    | jq -r '[.series[]?.pointlist[]? | select(.[1] != null) | .[0]] | max // 0')"
  ((${latest%.*} / 1000 >= now - 180))
}

wait_until 420 "http_check data from the gallery-prod agent" \
  has_recent_data 'avg:network.http.can_connect{instance:gallery_api,env:prod}'
wait_until 420 "OpenMetrics request counters from the API" \
  has_recent_data 'sum:gallery.http.requests.count{service:gallery-api,env:prod}.as_count()'

monitors="$(dd GET "/api/v1/monitor?monitor_tags=${MANAGED_TAG}")"
count="$(jq length <<<"$monitors")"
jq -r '.[] | "  [\(.overall_state)] \(.name)  \(.id)"' <<<"$monitors"
((count >= EXPECTED_MONITORS)) || die "expected ${EXPECTED_MONITORS} managed monitors, found ${count}"
log "${count} alert rules loaded: $(dd_app_url)/monitors/manage?q=tag%3A%22${MANAGED_TAG}%22"

dd_event "Released ${RELEASE} to gallery-prod" \
  "Jenkins build ${BUILD_URL:-local} promoted ${RELEASE} to production." success "release:${RELEASE}"
log "posted release event for ${RELEASE}"
