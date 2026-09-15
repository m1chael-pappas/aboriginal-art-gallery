#!/usr/bin/env bash
# Brings up the whole CI/CD toolchain on a clean machine. Safe to re-run.
#
#   1. checks Docker, compose, curl, jq, openssl and ssh-keygen
#   2. creates .env.jenkins, .env.staging and .env.prod with random secrets
#      (existing files are kept, missing keys are added)
#   3. generates the GitHub deploy key in secrets/ if it does not exist
#   4. builds the Jenkins image and starts SonarQube, then configures it
#   5. starts Jenkins, which configures itself from jenkins/casc.yaml
#
# Usage: scripts/ci-up.sh

source "$(dirname "$0")/lib/common.sh"
require docker curl jq openssl ssh-keygen
docker compose version >/dev/null 2>&1 || die "the docker compose plugin is required"

cd "$REPO_ROOT" || exit 1
umask 077

# ensure_env <file> <example> <KEY=generator...>: copies the example if the
# file is missing, then fills every listed key that is still empty.
ensure_env() {
  local file="$1" example="$2" pair key generator
  shift 2
  [[ -f "$file" ]] || cp "$example" "$file"
  for pair in "$@"; do
    key="${pair%%=*}"
    generator="${pair#*=}"
    [[ -n "$(env_get "$file" "$key" || true)" ]] || env_set "$file" "$key" "$($generator)"
  done
}

hex24() { openssl rand -hex 24; }
hex32() { openssl rand -hex 32; }
docker_gid() { stat -c %g /var/run/docker.sock; }

ensure_env .env.jenkins .env.jenkins.example \
  DOCKER_GID=docker_gid JENKINS_ADMIN_PASSWORD=hex24 SONAR_DB_PASSWORD=hex24 SONAR_ADMIN_PASSWORD=gen_password
ensure_env .env.staging .env.staging.example \
  POSTGRES_PASSWORD=hex24 JWT_SECRET=hex32 SEED_ADMIN_PASSWORD=hex24
ensure_env .env.prod .env.prod.example \
  POSTGRES_PASSWORD=hex24 JWT_SECRET=hex32 SEED_ADMIN_PASSWORD=hex24

mkdir -p secrets
[[ -f secrets/github_deploy_key ]] \
  || ssh-keygen -q -t ed25519 -N "" -C "gallery-jenkins-release" -f secrets/github_deploy_key

compose() { docker compose -f docker-compose.jenkins.yml --env-file .env.jenkins "$@"; }

log "building the Jenkins image (first run takes a few minutes)"
compose build jenkins
compose up -d --wait sonarqube
"$REPO_ROOT/scripts/sonar-bootstrap.sh" .env.jenkins
compose up -d --wait --force-recreate jenkins

for key in DD_API_KEY DD_APP_KEY ALERT_EMAIL; do
  [[ -n "$(env_get .env.jenkins "$key" || true)" ]] \
    || log "WARNING: ${key} is empty in .env.jenkins; the Release and Monitoring stages need it (then re-run this script)"
done

cat <<SUMMARY

Jenkins   http://localhost:8090   user $(env_get .env.jenkins JENKINS_ADMIN_USER)   password in .env.jenkins
SonarQube http://localhost:9000   user admin   password in .env.jenkins (SONAR_ADMIN_PASSWORD)

Add this public key to GitHub > Settings > Deploy keys with "Allow write access":
$(cat secrets/github_deploy_key.pub)
SUMMARY
