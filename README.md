# Aboriginal Art Gallery

Backend service (with a thin Vue web client) for the curatorial catalogue of
an **Aboriginal art gallery of Australia** - University Practical Task 5.2
(HD). It records artists, the artifacts they produced, and the tribes /
language groups they belong to, including each tribe's traditional **Country**
as real PostGIS geographic data, with all curatorial edits gated behind
authenticated, role-restricted access while the catalogue stays publicly
readable.

> We acknowledge the Traditional Custodians of the lands across Australia and
> pay our respects to Elders past and present. Territory polygons in this
> project are coarse demo approximations, **not** authoritative cultural
> boundaries.

## Stack

| Layer | Choice | Notable for the brief |
|---|---|---|
| Web API | **Rust + Axum 0.8** | Non-ASP.NET stack |
| Data access | **sqlx 0.8** - compile-time-checked SQL | Not an ORM; queries verified against a live DB at build time |
| Database | **PostgreSQL 16 + PostGIS 3.4** | CITEXT, GiST, functional index, CHECKs, triggers, spatial queries |
| Auth | **Argon2id + HS256 JWT**, typed role extractors | Auth approach not covered by the unit |
| Frontend | **Vue 3 + Vite + Pinia + Tailwind v4** | Added front-end |
| API docs | **OpenAPI / Swagger UI** + rustdoc | Custom + generated documentation |
| Types | **openapi-typescript** | Frontend types generated from the spec - cannot drift |
| Testing | **`sqlx::test`** ephemeral DBs + HTTP-level tests | Testing framework |

## Bounded contexts

Four (the brief allows 3-5): **Artists**, **Artifacts**, **Tribes** (subsumes
*Maps* - territory + spatial search), **Users & Access** (subsumes
*Membership and Roles* - JWT auth + `User`/`Admin`). Rationale in
[`docs/brd.md`](./docs/brd.md).

## Quick start

Prereqs: Docker, Rust toolchain + [`sqlx-cli`](https://crates.io/crates/sqlx-cli)
(`cargo install sqlx-cli --no-default-features --features native-tls,postgres`),
Node + `pnpm` (or `npm`).

```bash
# 1. Start Postgres + PostGIS
docker compose up -d db

# 2. Apply migrations. sqlx checks queries against a live DB at COMPILE time,
#    so the schema must exist before `cargo build`/`cargo run`.
cd api
sqlx migrate run

# 3. Seed curated data (tribes + territory polygons, artists, artifacts, admin)
cargo run --bin seed          # prints: admin@gallery.local / admin-demo-pw

# 4. Start the API (http://localhost:8080)
cargo run --bin gallery-api

# 5. In another shell - regenerate FE types from the live spec, then run Vite
cd web
pnpm install
pnpm run generate:types       # reads /api-docs/openapi.json -> src/api/generated.ts
pnpm run dev                  # http://localhost:5173
```

- Swagger UI: <http://localhost:8080/docs> · OpenAPI JSON:
  `/api-docs/openapi.json`
- Sign in (web or Swagger "Authorize") with the seeded admin to see
  curatorial CRUD and the admin **Users** page.

## Common commands

```bash
cd api && cargo test                 # full integration suite (ephemeral DBs)
cd api && cargo test --test auth      # one file (auth | territory | …)
cd api && cargo doc --no-deps         # rustdoc site -> target/doc/gallery_api/
docker compose down -v                # wipe the DB volume (fresh start)
```

## Project structure

```
api/                Rust backend (Axum + sqlx)
  src/<context>/    model.rs · repo.rs · routes.rs · mod.rs  (one shape per BC)
  src/auth/         JWT, Argon2id, AuthUser/AdminUser extractors
  src/error.rs      single AppError -> HTTP funnel
  src/openapi.rs    OpenAPI document assembly
  migrations/       forward-only SQL migrations
  tests/            HTTP-level integration tests
web/                Vue 3 SPA (views, Pinia stores, generated API types)
docs/               BRD, ERD, architecture, ADCs, original brief
docker-compose.yml  Postgres + PostGIS for local dev
```

## Documentation

| Document | Contents |
|---|---|
| [`docs/brd.md`](./docs/brd.md) | Business Requirements Document - vision, stakeholders, scope, ubiquitous language, functional/non-functional requirements, traceability to the brief |
| [`docs/erd.md`](./docs/erd.md) | Database schema, ERD (Mermaid), FK policies, PostgreSQL-specific features |
| [`docs/architecture.md`](./docs/architecture.md) | C4 context/container/component views, auth & request flow, source-of-truth pipeline (Mermaid) |
| [`docs/aggregate-design-canvases.md`](./docs/aggregate-design-canvases.md) | Aggregate Design Canvas per aggregate (invariants, commands, events) |
| [`docs/brief.md`](./docs/brief.md) | Original task brief (preserved verbatim) |
| Swagger UI (`/docs`) + rustdoc | Generated, always in sync with the running code |

### API client

There is no hand-maintained REST-client collection - it would drift from the
code the moment a route changed. Instead, import the live OpenAPI spec, which
is generated from the handlers themselves:

- **Swagger UI** - browse and call every endpoint at
  <http://localhost:8080/docs> (use *Authorize* to paste a JWT).
- **Insomnia** - *Create > Import > From URL* with
  `http://localhost:8080/api-docs/openapi.json`. Insomnia builds a request for
  every route automatically. Set a Bearer token at the collection level
  (Auth tab) so the write endpoints inherit it.
- **Postman / Bruno / Hoppscotch** - same idea: import the OpenAPI 3 URL.

The spec is always in sync with the running API by construction (see the
source-of-truth pipeline in [`docs/architecture.md`](./docs/architecture.md)).

## Notes

Local-demo build by design: Docker Compose database, dev JWT secret in
`.env.example`, permissive CORS - not production-hardened, and deliberately
scoped (no Exhibitions/Comments/Tags, no map rendering) to keep the boundaries
honest and the solution submittable. See `docs/brd.md` §3.2 for the full
out-of-scope list.
