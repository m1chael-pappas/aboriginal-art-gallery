# Aboriginal Art Gallery

Backend service and web frontend for an aboriginal art gallery of Australia.
University Practical Task 5.2.

## Stack

- **Backend** - Rust, Axum, sqlx, PostgreSQL 16 + PostGIS 3.4
- **Frontend** - Vue 3, Vite, Pinia, Vue Router, TailwindCSS
- **Infra** - Docker Compose

## Project structure

```
api/                Rust backend (Axum + sqlx)
web/                Vue 3 frontend
docs/               BRD, schema, architecture diagrams, task brief
docker-compose.yml  Postgres + PostGIS for local dev
PLAN.md             Milestones and decisions
TODO.md             Active work items
```

## Quick start

> To be filled in once the Cargo and Vue scaffolds land.

```bash
# Start the database
docker compose up -d

# Backend (coming soon)
# cd api && cargo run

# Frontend (coming soon)
# cd web && npm install && npm run dev
```

## Documentation

- [`PLAN.md`](./PLAN.md) - milestones, tech decisions
- [`TODO.md`](./TODO.md) - active todo list
- [`docs/brief.md`](./docs/brief.md) - original task brief
- [`docs/insomnia.json`](./docs/insomnia.json) - Insomnia REST client collection (import via *Application > Preferences > Data > Import Data*)
