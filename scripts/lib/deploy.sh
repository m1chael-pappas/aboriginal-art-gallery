#!/usr/bin/env bash
# Deployment helpers shared by the Jenkinsfile stages, release.sh and
# rollback.sh. Source after lib/common.sh.

GALLERY_IMAGES=(gallery-api gallery-web gallery-datadog-agent)

# stack_compose <staging|prod> <env-file> <image-tag> <compose args...>
stack_compose() {
  local stack="$1" env_file="$2" tag="$3"
  shift 3
  [[ -f "$env_file" ]] || die "env file not found: $env_file"
  IMAGE_TAG="$tag" docker compose \
    --project-directory "$REPO_ROOT" \
    -f "$REPO_ROOT/compose.${stack}.yml" \
    --env-file "$env_file" \
    "$@"
}

# deploy_stack <staging|prod> <env-file> <image-tag> <seed args...>
# Starts or updates the stack, waits for every health check, then runs the
# seed job with the given arguments.
deploy_stack() {
  local stack="$1" env_file="$2" tag="$3"
  shift 3
  log "deploying ${tag} to ${stack}"
  stack_compose "$stack" "$env_file" "$tag" up -d --wait --wait-timeout 180 --remove-orphans
  stack_compose "$stack" "$env_file" "$tag" run --rm seed "$@"
  stack_compose "$stack" "$env_file" "$tag" ps --format 'table {{.Service}}\t{{.Image}}\t{{.Status}}'
}

# tag_images <source-tag> <target-tag>: tags every gallery image.
tag_images() {
  local image
  for image in "${GALLERY_IMAGES[@]}"; do
    docker image inspect "${image}:$1" >/dev/null 2>&1 || die "missing image ${image}:$1"
    docker tag "${image}:$1" "${image}:$2"
  done
}

RELEASE_STATE_VOLUME="gallery-prod-releases"
RELEASE_STATE_IMAGE="alpine:3.22"

# release_state: prints the production release record kept on a Docker
# volume, or {} before the first release. The record lives next to the
# containers instead of in Jenkins, so a rollback works from any shell.
release_state() {
  docker run --rm -v "${RELEASE_STATE_VOLUME}:/state" "$RELEASE_STATE_IMAGE" \
    sh -c 'cat /state/prod.json 2>/dev/null || echo "{}"'
}

current_release() { release_state | jq -r '.current // empty'; }
previous_release() { release_state | jq -r '.previous // empty'; }

# record_release <release> <action>: makes <release> current, keeps the old
# current as previous and appends to the history.
record_release() {
  local release="$1" action="$2"
  release_state \
    | jq --arg r "$release" --arg a "$action" --arg at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
      '.previous = (if .current and .current != $r then .current else .previous end)
       | .current = $r
       | .history = ((.history // []) + [{release: $r, action: $a, at: $at}])' \
    | docker run --rm -i -v "${RELEASE_STATE_VOLUME}:/state" "$RELEASE_STATE_IMAGE" \
      sh -c 'cat >/state/prod.json.tmp && mv /state/prod.json.tmp /state/prod.json'
}

# prod_env_file: env file for production. Jenkins passes PROD_ENV_FILE from
# the env-prod credential, and a local shell falls back to .env.prod.
prod_env_file() {
  printf '%s' "${PROD_ENV_FILE:-$REPO_ROOT/.env.prod}"
}

# load_datadog_env: when DD_API_KEY is unset, reads the Datadog variables
# from .env.jenkins so scripts run by hand use the same keys as Jenkins.
load_datadog_env() {
  local file="$REPO_ROOT/.env.jenkins" key
  [[ -n "${DD_API_KEY:-}" || ! -f "$file" ]] && return 0
  for key in DD_API_KEY DD_APP_KEY DD_SITE ALERT_EMAIL; do
    export "$key=$(env_get "$file" "$key")"
  done
}
