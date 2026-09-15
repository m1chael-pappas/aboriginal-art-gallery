#!/usr/bin/env bash
# Simulates a production outage and proves the alerting loop end to end.
#
# Usage: scripts/simulate-incident.sh [record-file]
#
# 1. waits for the "API is down" monitor to be OK
# 2. stops the production API container
# 3. waits for Datadog to move the monitor to Alert, which emails ALERT_EMAIL
# 4. starts the API again and waits for its health check
# 5. waits for the monitor to resolve back to OK
#
# Posts Datadog events at each step so the dashboard shows the incident
# window, and writes the measured timings as JSON.

source "$(dirname "$0")/lib/common.sh"
source "$(dirname "$0")/lib/deploy.sh"
source "$(dirname "$0")/lib/datadog.sh"
require docker jq curl

RECORD_FILE="${1:-}"
MONITOR_NAME="[gallery-prod] API is down"
PROJECT="gallery-prod"
STATE_TIMEOUT="${STATE_TIMEOUT:-600}"

load_datadog_env

monitor_id="$(dd_monitor_id "$MONITOR_NAME")"
[[ -n "$monitor_id" ]] || die "monitor '${MONITOR_NAME}' not found, run scripts/datadog-apply.sh first"
log "monitor ${monitor_id}: $(dd_app_url)/monitors/${monitor_id}"

# wait_for_state <state> <timeout>: polls the monitor, prints seconds waited.
wait_for_state() {
  local want="$1" timeout="$2" started=$SECONDS state=""
  while ((SECONDS - started < timeout)); do
    state="$(dd_monitor_state "$monitor_id" || true)"
    if [[ "$state" == "$want" ]]; then
      log "monitor is ${want} after $((SECONDS - started))s"
      printf '%s' "$((SECONDS - started))"
      return 0
    fi
    log "monitor is ${state:-unknown}, waiting for ${want} ($((SECONDS - started))s)"
    sleep 15
  done
  die "monitor did not reach ${want} within ${timeout}s (last state: ${state:-unknown})"
}

restart_api() {
  docker compose -p "$PROJECT" start api >/dev/null
}

wait_for_state OK "$STATE_TIMEOUT" >/dev/null

dd_event "Incident simulation: stopping gallery-prod API" \
  "scripts/simulate-incident.sh stopped the production API container to test alerting." warning "incident:simulation"
trap 'log "restarting the API after an interrupted simulation"; restart_api' EXIT
log "stopping ${PROJECT} api"
stopped_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
docker compose -p "$PROJECT" stop api >/dev/null

seconds_to_alert="$(wait_for_state Alert "$STATE_TIMEOUT")"
alerted_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

log "starting ${PROJECT} api"
restart_api
trap - EXIT
wait_until 180 "gallery-prod api healthy" \
  bash -c "[[ \"\$(docker inspect -f '{{.State.Health.Status}}' \$(docker compose -p ${PROJECT} ps -q api))\" == healthy ]]"
restarted_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

seconds_to_resolve="$(wait_for_state OK "$STATE_TIMEOUT")"
resolved_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

dd_event "Incident simulation resolved" \
  "Monitor ${monitor_id} alerted ${seconds_to_alert}s after the API stopped and resolved ${seconds_to_resolve}s after it restarted." success "incident:simulation"

if [[ -n "$RECORD_FILE" ]]; then
  mkdir -p "$(dirname "$RECORD_FILE")"
  jq -n --arg monitor "$monitor_id" --arg stopped "$stopped_at" --arg alerted "$alerted_at" \
    --arg restarted "$restarted_at" --arg resolved "$resolved_at" \
    --argjson to_alert "$seconds_to_alert" --argjson to_resolve "$seconds_to_resolve" \
    '{monitor_id: $monitor, api_stopped_at: $stopped, alert_at: $alerted, seconds_to_alert: $to_alert,
      api_restarted_at: $restarted, resolved_at: $resolved, seconds_to_resolve: $to_resolve}' >"$RECORD_FILE"
fi
log "incident simulation passed: alert after ${seconds_to_alert}s, resolved ${seconds_to_resolve}s after restart"
