#!/usr/bin/env bash
# Datadog API helpers. Source after lib/common.sh.
#
# Requires DD_API_KEY and DD_APP_KEY. DD_SITE defaults to datadoghq.com
# (US1); set it to datadoghq.eu, us3.datadoghq.com, us5.datadoghq.com or
# ap1.datadoghq.com for other regions.

MANAGED_TAG="managed-by:gallery-pipeline"

# dd_site: DD_SITE, read at call time so scripts can load it after sourcing.
dd_site() {
  printf '%s' "${DD_SITE:-datadoghq.com}"
}

# dd_app_url: base URL of the Datadog web app for the site.
dd_app_url() {
  case "$(dd_site)" in
    datadoghq.com | datadoghq.eu) printf 'https://app.%s' "$(dd_site)" ;;
    *) printf 'https://%s' "$(dd_site)" ;;
  esac
}

# dd <METHOD> <path> [curl args...]: authenticated call, prints the JSON body,
# fails with the response body on any non-2xx status. DD_QUIET=1 skips the
# error log for calls where a failure is an expected branch.
dd() {
  local method="$1" path="$2" body status
  shift 2
  : "${DD_API_KEY:?DD_API_KEY is not set}" "${DD_APP_KEY:?DD_APP_KEY is not set}"
  body="$(mktemp)"
  status="$(curl -sS -o "$body" -w '%{http_code}' -X "$method" "https://api.$(dd_site)${path}" \
    -H "DD-API-KEY: ${DD_API_KEY}" \
    -H "DD-APPLICATION-KEY: ${DD_APP_KEY}" \
    -H 'Content-Type: application/json' \
    "$@")"
  if [[ "$status" != 2* ]]; then
    [[ -n "${DD_QUIET:-}" ]] || log "Datadog ${method} ${path} returned HTTP ${status}: $(cat "$body")"
    rm -f "$body"
    return 1
  fi
  cat "$body"
  rm -f "$body"
}

# dd_monitor_id <name>: id of the pipeline-managed monitor with this exact name.
dd_monitor_id() {
  dd GET "/api/v1/monitor?monitor_tags=${MANAGED_TAG}" \
    | jq -r --arg name "$1" '[.[] | select(.name == $name)][0].id // empty'
}

# dd_monitor_state <id>: overall state (OK, Alert, Warn, No Data, ...).
dd_monitor_state() {
  dd GET "/api/v1/monitor/$1" | jq -r '.overall_state'
}

# dd_event <title> <text> [alert_type] [extra tag...]: posts an event that
# dashboards overlay on their graphs.
dd_event() {
  local title="$1" text="$2" alert_type="${3:-info}"
  shift 3 || shift $#
  local tags
  tags="$(printf '%s\n' "$MANAGED_TAG" "env:prod" "service:gallery-api" "$@" | jq -R . | jq -s .)"
  jq -n --arg title "$title" --arg text "$text" --arg type "$alert_type" --argjson tags "$tags" \
    '{title: $title, text: $text, alert_type: $type, source_type_name: "jenkins", tags: $tags}' \
    | dd POST /api/v1/events --data-binary @- >/dev/null
}
