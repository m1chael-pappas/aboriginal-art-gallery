#!/usr/bin/env bash
# Configures SonarQube for the pipeline through its Web API. Safe to re-run.
#
#   - replaces the default admin/admin password with SONAR_ADMIN_PASSWORD
#   - creates the project and a least-privilege "jenkins" user
#   - creates the "Gallery gate" quality gate and assigns it to the project
#   - registers the Jenkins webhook that waitForQualityGate listens for
#   - generates an analysis token and writes it to SONAR_TOKEN in .env.jenkins
#
# Usage: scripts/sonar-bootstrap.sh [path/to/.env.jenkins]

source "$(dirname "$0")/lib/common.sh"
require curl jq openssl

ENV_FILE="${1:-$REPO_ROOT/.env.jenkins}"
[[ -f "$ENV_FILE" ]] || die "$ENV_FILE not found"

SONAR_URL="${SONAR_URL:-http://localhost:9000}"
PROJECT_KEY="aboriginal-art-gallery"
PROJECT_NAME="Aboriginal Art Gallery"
GATE_NAME="Gallery gate"
WEBHOOK_URL="http://jenkins:8080/sonarqube-webhook/"
CI_LOGIN="jenkins"
TOKEN_NAME="jenkins-ci"

ADMIN_PASSWORD="$(env_get "$ENV_FILE" SONAR_ADMIN_PASSWORD)"
[[ -n "$ADMIN_PASSWORD" ]] || die "SONAR_ADMIN_PASSWORD is empty in $ENV_FILE"

api() {
  local method="$1" path="$2"
  shift 2
  curl -fsS -u "admin:${ADMIN_PASSWORD}" -X "$method" "${SONAR_URL}${path}" "$@"
}

wait_until 600 "SonarQube at ${SONAR_URL}" \
  bash -c "curl -fsS '${SONAR_URL}/api/system/status' | grep -q '\"status\":\"UP\"'"

if curl -fsS -u admin:admin "${SONAR_URL}/api/authentication/validate" | grep -q '"valid":true'; then
  log "changing the default admin password"
  curl -fsS -u admin:admin -X POST "${SONAR_URL}/api/users/change_password" \
    --data-urlencode "login=admin" \
    --data-urlencode "previousPassword=admin" \
    --data-urlencode "password=${ADMIN_PASSWORD}"
fi
api GET /api/authentication/validate | grep -q '"valid":true' || die "admin login failed, check SONAR_ADMIN_PASSWORD"

if ! api GET "/api/projects/search?projects=${PROJECT_KEY}" | jq -e '.components | length > 0' >/dev/null; then
  log "creating project ${PROJECT_KEY}"
  api POST /api/projects/create --data-urlencode "project=${PROJECT_KEY}" --data-urlencode "name=${PROJECT_NAME}" >/dev/null
fi
api POST /api/new_code_periods/set --data-urlencode "project=${PROJECT_KEY}" --data-urlencode "type=PREVIOUS_VERSION"

if ! api GET "/api/users/search?q=${CI_LOGIN}" | jq -e --arg l "$CI_LOGIN" '.users[] | select(.login == $l)' >/dev/null; then
  log "creating user ${CI_LOGIN}"
  api POST /api/users/create \
    --data-urlencode "login=${CI_LOGIN}" \
    --data-urlencode "name=Jenkins CI" \
    --data-urlencode "password=$(gen_password)" >/dev/null
fi
for permission in user scan; do
  api POST /api/permissions/add_user \
    --data-urlencode "login=${CI_LOGIN}" \
    --data-urlencode "projectKey=${PROJECT_KEY}" \
    --data-urlencode "permission=${permission}"
done

if ! api GET /api/qualitygates/list | jq -e --arg n "$GATE_NAME" '.qualitygates[] | select(.name == $n)' >/dev/null; then
  log "creating quality gate ${GATE_NAME}"
  api POST /api/qualitygates/create --data-urlencode "name=${GATE_NAME}" >/dev/null
fi
api GET "/api/qualitygates/show?name=$(jq -rn --arg n "$GATE_NAME" '$n|@uri')" \
  | jq -r '.conditions[]?.id' \
  | while read -r condition; do
      api POST /api/qualitygates/delete_condition --data-urlencode "id=${condition}"
    done
add_condition() {
  api POST /api/qualitygates/create_condition \
    --data-urlencode "gateName=${GATE_NAME}" \
    --data-urlencode "metric=$1" \
    --data-urlencode "op=$2" \
    --data-urlencode "error=$3" >/dev/null
}
add_condition new_coverage LT 60
add_condition new_duplicated_lines_density GT 3
add_condition new_software_quality_blocker_issues GT 0
add_condition new_software_quality_high_issues GT 0
api POST /api/qualitygates/select --data-urlencode "gateName=${GATE_NAME}" --data-urlencode "projectKey=${PROJECT_KEY}"
log "quality gate ${GATE_NAME} assigned to ${PROJECT_KEY}"

api GET "/api/webhooks/list?project=${PROJECT_KEY}" | jq -r '.webhooks[]?.key' \
  | while read -r hook; do api POST /api/webhooks/delete --data-urlencode "webhook=${hook}"; done
api POST /api/webhooks/create \
  --data-urlencode "name=Jenkins" \
  --data-urlencode "project=${PROJECT_KEY}" \
  --data-urlencode "url=${WEBHOOK_URL}" >/dev/null
log "webhook registered: ${WEBHOOK_URL}"

current_token="$(env_get "$ENV_FILE" SONAR_TOKEN || true)"
if [[ -n "$current_token" ]] && curl -fsS -u "${current_token}:" "${SONAR_URL}/api/authentication/validate" | grep -q '"valid":true'; then
  log "existing SONAR_TOKEN is valid, keeping it"
else
  api POST /api/user_tokens/revoke --data-urlencode "login=${CI_LOGIN}" --data-urlencode "name=${TOKEN_NAME}" || true
  token="$(api POST /api/user_tokens/generate \
    --data-urlencode "login=${CI_LOGIN}" \
    --data-urlencode "name=${TOKEN_NAME}" \
    --data-urlencode "type=USER_TOKEN" | jq -r .token)"
  env_set "$ENV_FILE" SONAR_TOKEN "$token"
  log "generated a new token for ${CI_LOGIN} and wrote SONAR_TOKEN to ${ENV_FILE}"
fi
