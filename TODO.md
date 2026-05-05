# TODO

Active work items for the Aboriginal Art Gallery project.
See `PLAN.md` for the milestone plan and `docs/brief.md` for the task brief.

## Phase 0 — Setup
- [x] Repo structure, README, PLAN, TODO
- [x] Docker Compose with Postgres + PostGIS
- [x] `.gitignore` for Rust + Node
- [ ] Cargo project (`api/`) with Axum hello-world
- [ ] sqlx-cli installed, first migration applied
- [ ] Vue + Vite + Pinia + Router + Tailwind scaffold (`web/`)
- [ ] `.env.example` with `DATABASE_URL`, `JWT_SECRET`
- [ ] README quick-start filled in

## Phase 1 — Design (Week 1)
- [ ] BRD draft covering all bounded contexts
- [ ] Aggregate Design Canvas — Artists
- [ ] Aggregate Design Canvas — Artifacts
- [ ] Aggregate Design Canvas — Tribes
- [ ] DB schema diagram (dbdiagram.io)
- [ ] Architecture diagram (C4 context + container)

## Phase 2 — Artists BC (Week 1)
- [ ] Migration: `artists` table
- [ ] Domain model + sqlx repository
- [ ] Routes: `GET /artists`, `GET /artists/:id`, `POST`, `PUT`, `DELETE`
- [ ] Integration tests (reqwest against test DB)
- [ ] Vue: artist list page
- [ ] Vue: artist detail page
- [ ] Vue: create / edit form
- [ ] Vue: delete confirmation

## Phase 3 — Artifacts BC (Week 2)
- [ ] Migration: `artifacts` table + FK to `artists`
- [ ] Migration: `art_types`, `art_styles` lookup tables (or enums)
- [ ] Domain model + repository
- [ ] Routes: full CRUD
- [ ] Integration tests
- [ ] Vue: artifact CRUD UI
- [ ] Vue: artist picker on artifact form

## Phase 4 — Tribes BC (Week 2)
- [ ] Migration: `tribes` table
- [ ] Domain model + repository
- [ ] Routes: full CRUD
- [ ] FK from `artists.tribe_id` → `tribes.id`
- [ ] Integration tests
- [ ] Vue: tribes CRUD UI
- [ ] Vue: tribe picker on artist form

## Phase 5 — Auth + Users (Week 3)
- [ ] Migration: `users` table
- [ ] Argon2 password hashing
- [ ] JWT issuance + verification
- [ ] Axum extractor for authenticated user
- [ ] Routes: `POST /auth/register`, `POST /auth/login`, `GET /auth/me`
- [ ] Vue: login + register pages
- [ ] Vue: token storage (decide cookie vs localStorage), auth store
- [ ] Vue: route guards on protected pages
- [ ] Decide which mutations require auth

## Phase 6 — Polish (Week 4)
- [ ] README — full setup, run, test instructions
- [ ] Rustdoc comments on public modules and handlers
- [ ] Architecture diagram finalized
- [ ] DB schema diagram finalized
- [ ] Full integration test pass
- [ ] Demo script written
- [ ] Demo rehearsal (allow 2 days)

## Stretch
- [ ] Exhibitions BC (4th bounded context, ties everything together)
- [ ] PostGIS: `tribes.territory geography(MultiPolygon, 4326)` + GIST index
- [ ] PostGIS: spatial query — artists by region
- [ ] Comments on artifacts
- [ ] Pagination + filtering on list endpoints
