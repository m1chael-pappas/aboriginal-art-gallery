# Security findings and allowlist

The Security stage runs five scanners, and any HIGH or CRITICAL finding in code we ship fails the build.
This file records every finding the stage has raised, what we did about it, and the few we accepted.
An accepted finding needs an entry here and a matching entry in the tool's ignore file, otherwise the next run fails again.

| Tool | Scope | Gate | Ignore file |
| --- | --- | --- | --- |
| `cargo audit` | Every crate in `api/Cargo.lock` against the RustSec advisory DB | Any vulnerability, unsound or yanked crate | `api/.cargo/audit.toml` |
| `cargo deny check` | Compiled dependency graph: advisories, licenses, banned crates, sources | Any error | `api/deny.toml` |
| `pnpm audit --audit-level=high` | `web/pnpm-lock.yaml` against the GitHub advisory DB | HIGH or CRITICAL | none needed |
| `trivy image` | `gallery-api` and `gallery-web` images (OS packages and app dependencies) | HIGH or CRITICAL | `.trivyignore.yaml` |
| `trivy fs` | Dockerfiles and compose files for misconfiguration, whole repo for committed secrets | HIGH or CRITICAL | `.trivyignore.yaml` |
| `trivy image` | `gallery-datadog-agent` (vendor image) | Report only, see below | none |

## Accepted findings

### RUSTSEC-2023-0071, `rsa` 0.9.10, Marvin attack timing side channel

- Severity: medium (CVSS 5.9, `AV:N/AC:H/PR:N/UI:N/S:U/C:H/I:N/A:N`). No fixed version exists.
- Tool: `cargo audit`.
- Why it appears: `sqlx-macros-core` lists `sqlx-mysql` as an optional dependency, and `sqlx-mysql` depends on `rsa`. Cargo writes optional crates into `Cargo.lock` even when their feature is off, and `cargo audit` scans the lockfile.
- Why it is not exploitable here: the API only enables the `postgres` feature. `cargo tree -i rsa --target all -e all` prints nothing, so `rsa` is never compiled into any binary. `cargo deny`, which checks the real dependency graph, does not report it.
- Mitigation: ignored in `api/.cargo/audit.toml`. Remove the ignore when sqlx stops locking MySQL crates for Postgres-only builds or `rsa` ships a constant-time release.

### DS-0002 on `monitoring/datadog/Dockerfile`, image runs as root

- Severity: HIGH (Trivy misconfiguration rule).
- Why it is accepted: the Datadog agent reads `/var/run/docker.sock`, `/proc` and cgroups to collect container metrics, which requires root. The base image is Datadog's own and runs as root by design.
- Mitigation: the socket, `/proc` and cgroups are mounted read-only. The container publishes no ports. APM, log collection and the process agent are disabled, so only the core collector and checks run. Scoped to this one path in `.trivyignore.yaml`, so every other Dockerfile still has to set a non-root user.

## Vendor image scanned in report-only mode

`gallery-datadog-agent` extends `gcr.io/datadoghq/agent:7.83.1`.
We cannot rebuild Datadog's Go binaries, so a failing gate would only block releases without any fix available to us.
The pipeline still scans it and archives `trivy-datadog-agent.json` with every build.

Findings from the scan on 2026-09-15:

| CVE | Package | Installed | Fixed in | Severity |
| --- | --- | --- | --- | --- |
| CVE-2026-56854 | `golang.org/x/crypto` | 0.54.0 | 0.55.0 | HIGH |
| CVE-2026-56864 | `golang.org/x/mod` | 0.38.0 | 0.40.0 | HIGH |
| CVE-2026-56865 | `golang.org/x/mod` | 0.38.0 | 0.40.0 | HIGH |
| CVE-2026-84304 | `google.golang.org/grpc` | 1.83.0 | 1.83.1 | HIGH |
| CVE-2026-84445 | `google.golang.org/grpc` | 1.83.0 | 1.83.2 | HIGH |

Mitigation: the agent container exposes no ports, so its gRPC and IPC listeners are reachable only inside the container.
Action: move `AGENT_VERSION` to the first 7.84 GA release, then rescan.

## Findings fixed

| Finding | Severity | Where | Fix |
| --- | --- | --- | --- |
| RUSTSEC-2026-0285, `rustls` 0.23.40 accepts TLS 1.3 handshake messages across encryption levels | Advisory published 2026-09-14 | `api` | `cargo update -p rustls` to 0.23.45 |
| RUSTSEC-2026-0190, `anyhow` 1.0.102 unsound `Error::downcast_mut()` | Unsound | `api` | Updated to 1.0.104 |
| RUSTSEC-2026-0221, `event-listener` 5.4.1 lets `!Send` values cross threads | Unsound | `api` via `sqlx-core` | Updated to 5.4.2 |
| `spin` 0.9.8 yanked | Yanked | `api` lockfile | Updated to 0.9.9 |
| Duplicate `tower-http` 0.6 and 0.7 in the graph | Hygiene | `api` | Direct dependency moved to 0.7 |
| GHSA-gcfj-64vw-6mp9 and 9 more `axios` advisories (proxy reuse, prototype pollution, DoS) | 1 HIGH, 9 moderate | `web` runtime | `axios` 1.16.0 to 1.20.0 |
| `form-data` CRLF injection | HIGH | `web` via `axios` | Resolved by the `axios` update |
| `vite` `server.fs.deny` bypass on Windows, `launch-editor` NTLM hash leak | HIGH, moderate | `web` dev server | `vite` 8.0.10 to 8.3.0 |
| `postcss` source map path traversal, `nanoid` infinite loops | HIGH | `web` build via `vite` | Resolved by the `vite` update |
| `js-yaml` quadratic CPU in merge keys (4 advisories), `brace-expansion` exponential expansion (3 advisories) | HIGH | `web` dev tooling via `openapi-typescript` | `openapi-typescript` 7.5.0 to 7.13.0 |
| DS-0002, `web/Dockerfile` without an explicit `USER` | HIGH | `gallery-web` image | Added `USER 101:101`. The base already ran as `nginx`, and the explicit user keeps a base image change from reverting to root |
| Secret scan flagged `secrets/github_deploy_key` | HIGH | Local working copy only | Not a leak: `secrets/` is gitignored and never reaches the Jenkins workspace. Local scans skip it with `--skip-dirs secrets` |
