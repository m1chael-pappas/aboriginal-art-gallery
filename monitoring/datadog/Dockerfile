ARG AGENT_VERSION=7.83.1
# The agent runs as root to read the Docker socket, /proc and cgroups.
# Accepted in security/allowlist.md.
FROM gcr.io/datadoghq/agent:${AGENT_VERSION}
COPY conf.d/ /etc/datadog-agent/conf.d/
