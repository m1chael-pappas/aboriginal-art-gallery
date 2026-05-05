# Project Plan

## Domain

Aboriginal Art Gallery of Australia — backend service with web frontend.
University Practical Task 5.2 (HD). See `docs/brief.md` for the full task brief.

## Tech Stack

**Backend**
- Rust + Axum
- sqlx (compile-time checked SQL, no ORM)
- PostgreSQL 16 + PostGIS 3.4
- argon2 (password hashing)
- jsonwebtoken (JWT)
- cargo test + reqwest (integration tests)

**Frontend**
- Vue 3 + Vite
- Pinia (state)
- Vue Router
- TailwindCSS

**Infra**
- Docker Compose (Postgres + PostGIS locally)

## Bounded Contexts

### MVP (3)

1. **Artists** — biography, tribe affiliation, region, date ranges
2. **Artifacts** — references Artist, Art Type, Art Style; the main aggregate
3. **Tribes** — name, region, language, description (metadata-only initially)

### Stretch

4. **Exhibitions** — m2m with Artifacts, gallery location, date range
5. **PostGIS on Tribes** — territory polygons + spatial queries (not a separate BC)

## Milestones

### Week 1 — Foundation + Artists
- Design: BRD, ADCs for 3 BCs, DB schema diagram
- Scaffold: Cargo workspace, Axum, sqlx, Docker Compose
- **Artists BC end-to-end** (migration → repo → handlers → tests) as the vertical-slice template

### Week 2 — Replicate the pattern
- Artifacts BC (FK to Artists)
- Tribes BC (metadata-only)
- Vue scaffold + API client + first CRUD page

### Week 3 — Frontend depth + auth
- Full CRUD UI for all 3 BCs
- Users + JWT auth (argon2, jsonwebtoken)
- Vue: login, token storage, route guards

### Week 4 — Stretch + polish
- Exhibitions OR PostGIS (whichever fits)
- Architecture diagram (C4)
- Rustdoc comments
- README finalized, demo rehearsal

## Decisions log

- **Maps as a separate BC: dropped.** Spatial data, if used, lives inside Tribes (territory) and Exhibitions (gallery point). Maps wasn't a true bounded context — no domain language of its own.
- **Auth: hand-rolled JWT, not OAuth/Auth0.** Just enough Users BC to satisfy that bonus; not chasing it.
- **Frontend: full CRUD UI, not map-first.** All 3 BCs get list / detail / create / edit / delete pages.
- **PostGIS in MVP: deferred.** Docker image includes it (free), but Tribes ships as plain metadata. Add polygons + spatial queries only if Week 4 has slack.
- **5-context cap: starting with 3.** Brief warns against >5 and risk of incomplete work; better to ship 3 polished than 5 half-done.
- **Password hashing: argon2** (not bcrypt, definitely not sha-anything).
