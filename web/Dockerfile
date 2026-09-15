ARG NODE_VERSION=22.23.2
ARG NGINX_VERSION=1.31.5

FROM node:${NODE_VERSION}-trixie-slim AS build
WORKDIR /app
RUN corepack enable
COPY package.json pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY . .
ENV VITE_API_URL=/api
RUN pnpm build

FROM nginxinc/nginx-unprivileged:${NGINX_VERSION}-alpine AS runtime
ARG VERSION=dev
LABEL org.opencontainers.image.title="gallery-web" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.source="https://github.com/m1chael-pappas/aboriginal-art-gallery"
ENV API_UPSTREAM=api:8080 \
    NGINX_ENVSUBST_FILTER=^API_UPSTREAM$
COPY nginx.conf.template /etc/nginx/templates/default.conf.template
COPY --from=build /app/dist /usr/share/nginx/html
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
    CMD wget -qO /dev/null http://127.0.0.1:8080/healthz || exit 1
