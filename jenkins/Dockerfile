ARG JENKINS_VERSION=2.568.3
ARG DOCKER_CLI_VERSION=28.5.2
ARG NODE_VERSION=22.23.2
ARG TRIVY_VERSION=0.74.0

FROM docker:${DOCKER_CLI_VERSION}-cli AS docker-cli
FROM node:${NODE_VERSION}-trixie-slim AS node
FROM aquasec/trivy:${TRIVY_VERSION} AS trivy

FROM jenkins/jenkins:${JENKINS_VERSION}-lts-jdk21

ARG RUST_VERSION=1.95.0
ARG PNPM_VERSION=10.24.0
ARG SONAR_SCANNER_VERSION=8.1.0.6389
ARG CARGO_BINSTALL_VERSION=1.23.0
ARG CARGO_NEXTEST_VERSION=0.9.144
ARG CARGO_LLVM_COV_VERSION=0.9.1
ARG CARGO_AUDIT_VERSION=0.22.1
ARG CARGO_DENY_VERSION=0.20.2
ARG SQLX_CLI_VERSION=0.8.6

USER root

RUN apt-get update \
 && apt-get install -y --no-install-recommends build-essential pkg-config ca-certificates curl git jq unzip openssh-client \
 && rm -rf /var/lib/apt/lists/*

COPY --from=docker-cli /usr/local/bin/docker /usr/local/bin/docker
COPY --from=docker-cli /usr/local/libexec/docker/cli-plugins /usr/local/libexec/docker/cli-plugins
COPY --from=node /usr/local/bin/node /usr/local/bin/node
COPY --from=node /usr/local/lib/node_modules /usr/local/lib/node_modules
COPY --from=trivy /usr/local/bin/trivy /usr/local/bin/trivy
RUN ln -s ../lib/node_modules/npm/bin/npm-cli.js /usr/local/bin/npm \
 && ln -s ../lib/node_modules/corepack/dist/corepack.js /usr/local/bin/corepack \
 && corepack enable --install-directory /usr/local/bin pnpm \
 && COREPACK_HOME=/opt/corepack corepack install --global pnpm@${PNPM_VERSION} \
 && chmod -R a+rX /opt/corepack

RUN curl -fsSL -o /tmp/scanner.zip "https://binaries.sonarsource.com/Distribution/sonar-scanner-cli/sonar-scanner-cli-${SONAR_SCANNER_VERSION}-linux-x64.zip" \
 && unzip -q /tmp/scanner.zip -d /opt \
 && mv /opt/sonar-scanner-${SONAR_SCANNER_VERSION}-linux-x64 /opt/sonar-scanner \
 && ln -s /opt/sonar-scanner/bin/sonar-scanner /usr/local/bin/sonar-scanner \
 && rm /tmp/scanner.zip

ENV RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo \
    COREPACK_HOME=/opt/corepack \
    PATH=/opt/cargo/bin:$PATH

RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile minimal \
      --default-toolchain ${RUST_VERSION} --component clippy,rustfmt,llvm-tools-preview \
 && curl -fsSL "https://github.com/cargo-bins/cargo-binstall/releases/download/v${CARGO_BINSTALL_VERSION}/cargo-binstall-x86_64-unknown-linux-gnu.tgz" | tar xz -C /opt/cargo/bin \
 && cargo binstall --no-confirm --locked --disable-strategies quick-install \
      cargo-nextest@${CARGO_NEXTEST_VERSION} \
      cargo-llvm-cov@${CARGO_LLVM_COV_VERSION} \
      cargo-audit@${CARGO_AUDIT_VERSION} \
      cargo-deny@${CARGO_DENY_VERSION} \
 && cargo install --locked sqlx-cli@${SQLX_CLI_VERSION} --no-default-features --features rustls,postgres \
 && rm -rf /opt/cargo/registry /opt/cargo/git \
 && mkdir -p /opt/cargo/registry /opt/cargo/git /var/cache/cargo-target /var/cache/pnpm /var/cache/trivy /var/cache/sonar \
 && chown -R jenkins:jenkins /opt/cargo /opt/rustup /var/cache/cargo-target /var/cache/pnpm /var/cache/trivy /var/cache/sonar

COPY --chown=jenkins:jenkins plugins.txt /usr/share/jenkins/ref/plugins.txt
USER jenkins
RUN jenkins-plugin-cli --plugin-file /usr/share/jenkins/ref/plugins.txt

ENV JAVA_OPTS="-Djenkins.install.runSetupWizard=false -Xmx2g" \
    CASC_JENKINS_CONFIG=/etc/jenkins/casc.yaml \
    CARGO_TARGET_DIR=/var/cache/cargo-target \
    npm_config_store_dir=/var/cache/pnpm \
    TRIVY_CACHE_DIR=/var/cache/trivy \
    SONAR_USER_HOME=/var/cache/sonar
