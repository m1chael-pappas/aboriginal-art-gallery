# Jenkins setup

This guide takes a clean machine to a running pipeline.
Jenkins and SonarQube configure themselves from files in this repository, so the manual steps are limited to accounts and keys only you can create.

## Prerequisites

| Requirement | Why | Check |
| --- | --- | --- |
| Docker Engine 24+ with the compose v2 plugin, running natively on Linux or inside WSL2 | Jenkins drives the host Docker daemon through the mounted socket | `docker compose version` |
| 16 GB RAM available to Docker (24 GB recommended on WSL2) | Jenkins, SonarQube, a Rust release build and two app stacks run together | `docker info --format '{{.MemTotal}}'` |
| `curl`, `jq`, `openssl`, `ssh-keygen` on the host | Used by the bootstrap scripts | `which curl jq openssl ssh-keygen` |
| Free ports 8081, 8082, 8090, 8091, 8092, 9000 | Staging, production, Jenkins and SonarQube | `ss -ltn` |
| A GitHub repository you can add a deploy key to | The Release stage pushes version tags | |
| A Datadog account (the free plan is enough) | Production monitoring and alerting | |

On WSL2 set the VM memory in `%UserProfile%\.wslconfig`, then run `wsl --shutdown`:

```ini
[wsl2]
memory=24GB
```

## Quick start

```bash
git clone https://github.com/m1chael-pappas/aboriginal-art-gallery.git
cd aboriginal-art-gallery
scripts/ci-up.sh
```

`ci-up.sh` is safe to re-run.
It creates `.env.jenkins`, `.env.staging` and `.env.prod` with random secrets, generates a deploy key in `secrets/`, builds the Jenkins image, starts and configures SonarQube, and starts Jenkins.
It prints the URLs and the deploy public key when it finishes.

Then do the three manual steps below and run `scripts/ci-up.sh` once more so Jenkins picks up the keys.

## Manual steps

### 1. GitHub deploy key

1. Open the repository on GitHub, then Settings > Deploy keys > Add deploy key.
2. Title: `gallery-jenkins-release`.
3. Key: the contents of `secrets/github_deploy_key.pub`.
4. Tick "Allow write access" and save.

If you forked the repository, change `GALLERY_REPO_URL` and `GALLERY_PUSH_URL` in `.env.jenkins` to your fork.

### 2. Datadog keys

1. Sign up at [datadoghq.com](https://www.datadoghq.com) and note the site in the page footer or URL, for example `ap2.datadoghq.com`.
2. The onboarding wizard waits for an agent. Pick Docker, copy the `DD_API_KEY` and `DD_SITE` values from the command it shows, and do not run the command. Production starts its own agent.
3. Organization Settings > Application Keys > New Key, name it `jenkins`.
4. Put the values in `.env.jenkins` without quotes:

```ini
DD_SITE=ap2.datadoghq.com
DD_API_KEY=<32 characters>
DD_APP_KEY=<40 characters>
ALERT_EMAIL=you@example.com
```

### 3. Re-run the bootstrap

```bash
scripts/ci-up.sh
```

Open <http://localhost:8090>, sign in as `admin` with `JENKINS_ADMIN_PASSWORD` from `.env.jenkins`, and start the `aboriginal-art-gallery` job with Build Now.
The job also polls GitHub every 2 minutes.

When the Release stage pauses, click Promote in the stage view or the console log.

## What is configured as code

| Thing | Where | Applied by |
| --- | --- | --- |
| Jenkins image: Docker CLI, compose, buildx, Rust 1.95 + clippy/rustfmt/llvm-tools, nextest, llvm-cov, cargo-audit, cargo-deny, sqlx-cli, Node 22 + pnpm, Trivy, sonar-scanner | `jenkins/Dockerfile` | `docker compose build` |
| Plugins | `jenkins/plugins.txt` | Image build |
| Admin user, security realm, SonarQube server, credentials, pipeline job | `jenkins/casc.yaml` | Configuration as Code on every Jenkins start |
| SonarQube admin password, project, `jenkins` user and token, quality gate, webhook | `scripts/sonar-bootstrap.sh` | `ci-up.sh` |
| SonarQube analysis scope and report paths | `sonar-project.properties` | Code Quality stage |
| Datadog monitors and dashboard | `monitoring/datadog/` | Monitoring stage |
| Staging and production stacks | `compose.base.yml`, `compose.staging.yml`, `compose.prod.yml` | Deploy and Release stages |

The setup wizard is disabled, so no plugin or credential is installed by clicking.

## Plugins

| Plugin id | Name | Used for |
| --- | --- | --- |
| `workflow-aggregator` | Pipeline | Declarative `Jenkinsfile` |
| `docker-workflow` | Docker Pipeline | `docker.build()` in the Build stage |
| `junit` | JUnit | Rust, web and smoke test results |
| `sonar` | SonarQube Scanner | `withSonarQubeEnv`, `waitForQualityGate` |
| `warnings-ng` | Warnings Next Generation | Clippy issues with a zero-issue gate |
| `coverage` | Coverage | LCOV coverage for Rust and web |
| `configuration-as-code` | Configuration as Code | `jenkins/casc.yaml` |
| `job-dsl` | Job DSL | The pipeline job definition inside `casc.yaml` |
| `git` | Git | SCM checkout and polling |
| `credentials-binding` | Credentials Binding | `withCredentials` for env files and Datadog keys |
| `ssh-agent` | SSH Agent | Pushing release tags with the deploy key |
| `pipeline-utility-steps` | Pipeline Utility Steps | JSON helpers |
| `pipeline-stage-view`, `pipeline-graph-view` | Stage View, Pipeline Graph View | Stage visualisation |
| `timestamper` | Timestamper | Timestamps in the console |
| `ws-cleanup` | Workspace Cleanup | Workspace hygiene |

## Credentials

All six are created by `jenkins/casc.yaml` from `.env.jenkins` and the mounted secret files.
To add them by hand instead, use Manage Jenkins > Credentials > System > Global credentials.

| ID | Kind | Source | Used by |
| --- | --- | --- | --- |
| `sonar-token` | Secret text | `SONAR_TOKEN`, written by `sonar-bootstrap.sh` | Code Quality |
| `datadog-api-key` | Secret text | `DD_API_KEY` | Release (agent), Monitoring |
| `datadog-app-key` | Secret text | `DD_APP_KEY` | Monitoring |
| `github-deploy-key` | SSH username with private key, user `git` | `secrets/github_deploy_key` | Release (tag push) |
| `env-staging` | Secret file | `.env.staging` | Deploy |
| `env-prod` | Secret file | `.env.prod` | Release |

## SonarQube server and webhook

Both are automated. To do it by hand:

1. Manage Jenkins > System > SonarQube servers: name `sonarqube`, URL `http://sonarqube:9000`, token credential `sonar-token`, and tick "Environment variables".
2. In SonarQube, Project Settings > Webhooks > Create: name `Jenkins`, URL `http://jenkins:8080/sonarqube-webhook/`.

Jenkins and SonarQube share the `gallery-ci` Docker network, so they reach each other by service name.
Without the webhook, `waitForQualityGate` waits until its 10 minute timeout.

## Quality gate

`sonar-bootstrap.sh` creates a gate named "Gallery gate" and assigns it to the project.
To create it by hand: Quality Gates > Create > "Gallery gate", add the conditions below on New Code, then Projects > aboriginal-art-gallery > Project Settings > Quality Gate.

| Metric | Operator | Threshold | Metric key |
| --- | --- | --- | --- |
| Coverage on new code | is less than | 60% | `new_coverage` |
| Duplicated lines on new code | is greater than | 3% | `new_duplicated_lines_density` |
| Blocker issues on new code | is greater than | 0 | `new_software_quality_blocker_issues` |
| High (critical) issues on new code | is greater than | 0 | `new_software_quality_high_issues` |

SonarQube 26 runs in Multi-Quality Rule mode, where "blocker" and "high" replace the older blocker and critical severities.

The new code period is "previous version", and the pipeline sets `sonar.projectVersion` to the build tag.
New code is therefore what changed since the last analysed build, which is what the push being built introduced.
The first analysis of a project has no new code, so the gate passes trivially and starts enforcing from the second build.

The Code Quality stage runs `waitForQualityGate abortPipeline: true`, so a failed gate stops the pipeline before Security.
Activity > Graphs in SonarQube shows coverage, duplication and issue trends across builds.

## Operating the pipeline

| Task | Command |
| --- | --- |
| Roll production back one release | `scripts/rollback.sh` |
| Roll production to a specific release | `scripts/rollback.sh v0.12.0` |
| Show the release record | `docker run --rm -v gallery-prod-releases:/s alpine:3.22 cat /s/prod.json` |
| Re-run the outage drill | `scripts/simulate-incident.sh` |
| Smoke test staging | `scripts/smoke-test.sh http://localhost:8091 http://localhost:8081 .env.staging` |
| Re-apply Datadog monitors and dashboard | `set -a; . ./.env.jenkins; set +a; scripts/datadog-apply.sh` |
| Stop the toolchain | `docker compose -f docker-compose.jenkins.yml --env-file .env.jenkins down` |

The scripts read `.env.prod` and `.env.jenkins` from the repository root when run by hand.

## Troubleshooting

| Symptom | Cause | Fix |
| --- | --- | --- |
| SonarQube exits with `max virtual memory areas vm.max_map_count [65530] is too low` | The `sonarqube-sysctl` init container could not run privileged | `sudo sysctl -w vm.max_map_count=524288`, or on WSL2 add `kernelCommandLine="sysctl.vm.max_map_count=524288"` to `.wslconfig` |
| `permission denied while trying to connect to the Docker daemon socket` in a build | `DOCKER_GID` does not match the socket group | Set `DOCKER_GID=$(stat -c %g /var/run/docker.sock)` in `.env.jenkins` and re-run `ci-up.sh` |
| Monitoring stage fails with HTTP 403 from Datadog | Wrong site or missing application key | Check `DD_SITE` matches your account's site and `DD_APP_KEY` is set |
| Release fails with `tag vX already exists on the remote` | Jenkins build numbers restarted after `jenkins_home` was wiped | Delete the old tag on GitHub, or run builds until the number passes it |
| Build killed with exit code 137 | Docker ran out of memory | Raise WSL2 memory or lower `CARGO_BUILD_JOBS` in the `Jenkinsfile` |
