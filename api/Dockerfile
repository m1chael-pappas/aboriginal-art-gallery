ARG RUST_VERSION=1.95.0
ARG CHEF_VERSION=0.1.77

FROM lukemathwalker/cargo-chef:${CHEF_VERSION}-rust-${RUST_VERSION}-trixie AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ENV SQLX_OFFLINE=true
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --locked --recipe-path recipe.json
COPY . .
RUN cargo build --release --locked --bins

FROM gcr.io/distroless/cc-debian13:nonroot AS runtime
ARG VERSION=dev
LABEL org.opencontainers.image.title="gallery-api" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.source="https://github.com/m1chael-pappas/aboriginal-art-gallery"
WORKDIR /app
COPY --from=builder /app/target/release/gallery-api /app/target/release/seed /app/target/release/healthcheck /app/
ENV BIND_ADDR=0.0.0.0:8080 \
    APP_VERSION=${VERSION}
EXPOSE 8080
USER nonroot:nonroot
HEALTHCHECK --interval=10s --timeout=5s --start-period=30s --retries=3 CMD ["/app/healthcheck"]
ENTRYPOINT ["/app/gallery-api"]
