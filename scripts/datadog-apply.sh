#!/usr/bin/env bash
# Creates or updates the Datadog monitors, metric configuration and dashboard
# in monitoring/datadog/.
# Monitors are matched by exact name among those tagged
# managed-by:gallery-pipeline, and the dashboard by title, so re-running
# updates in place instead of duplicating. Prints the dashboard URL on stdout.
#
# Env: DD_API_KEY, DD_APP_KEY, DD_SITE (optional), ALERT_EMAIL.

source "$(dirname "$0")/lib/common.sh"
source "$(dirname "$0")/lib/datadog.sh"
require curl jq

: "${ALERT_EMAIL:?ALERT_EMAIL is not set}"
CONFIG_DIR="$REPO_ROOT/monitoring/datadog"

dd GET /api/v1/validate | jq -e '.valid == true' >/dev/null || die "Datadog rejected DD_API_KEY"

for file in "$CONFIG_DIR"/monitors/*.json; do
  definition="$(sed "s/__ALERT_EMAIL__/${ALERT_EMAIL}/g" "$file")"
  name="$(jq -r .name <<<"$definition")"
  dd POST /api/v1/monitor/validate --data-binary "$definition" >/dev/null \
    || die "monitor definition rejected: $file"
  id="$(dd_monitor_id "$name")"
  if [[ -n "$id" ]]; then
    dd PUT "/api/v1/monitor/${id}" --data-binary "$definition" >/dev/null
    log "updated monitor ${id}: ${name}"
  else
    id="$(dd POST /api/v1/monitor --data-binary "$definition" | jq -r .id)"
    log "created monitor ${id}: ${name}"
  fi
done

for file in "$CONFIG_DIR"/metrics/*.json; do
  metric="$(jq -r .data.id "$file")"
  if DD_QUIET=1 dd POST "/api/v2/metrics/${metric}/tags" --data-binary @"$file" >/dev/null; then
    log "configured metric ${metric}"
  else
    dd PATCH "/api/v2/metrics/${metric}/tags" --data-binary @"$file" >/dev/null
    log "updated metric ${metric}"
  fi
done

dashboard="$(cat "$CONFIG_DIR/dashboard.json")"
title="$(jq -r .title <<<"$dashboard")"
dashboard_id="$(dd GET /api/v1/dashboard | jq -r --arg t "$title" '[.dashboards[] | select(.title == $t)][0].id // empty')"
if [[ -n "$dashboard_id" ]]; then
  response="$(dd PUT "/api/v1/dashboard/${dashboard_id}" --data-binary "$dashboard")"
  log "updated dashboard ${dashboard_id}"
else
  response="$(dd POST /api/v1/dashboard --data-binary "$dashboard")"
  log "created dashboard $(jq -r .id <<<"$response")"
fi
printf '%s%s\n' "$(dd_app_url)" "$(jq -r .url <<<"$response")"
