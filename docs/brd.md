# Business Requirements Document - Aboriginal Art Gallery

> University Practical Task 5.2 (HD). This document specifies *what* the
> backend service must do and *why*. The original assignment brief is
> preserved verbatim in [`brief.md`](./brief.md); this BRD elaborates it into
> a structured requirements set and is the authority when the two are read
> together. Schema is in [`erd.md`](./erd.md); runtime structure in
> [`architecture.md`](./architecture.md); aggregate rules in
> [`aggregate-design-canvases.md`](./aggregate-design-canvases.md).

## 1. Vision

A backend service, with a thin web client, for the curatorial catalogue of an
**Aboriginal art gallery of Australia**. It records the artists, the works
they produced, and the tribes / language groups they belong to - including the
*Country* each tribe is traditionally connected to as real geographic data -
and gates all curatorial edits behind authenticated, role-restricted access
while keeping the catalogue itself publicly readable.

The goal is not an exhaustive gallery ERP. It is a focused, correct slice that
demonstrates domain analysis, a ubiquitous language, well-bounded contexts,
and depth in the technology stack.

## 2. Stakeholders

| Stakeholder | Interest in the system |
|---|---|
| **Public visitor** | Browse artists, artifacts, and tribes without an account. Read-only. |
| **Curator / Admin** | Maintain the catalogue: create and correct artists, artifacts, tribes, and tribal territories. |
| **Registered user** | Holds an account; can manage their own credentials. (Foundation for future per-user features - comments, favourites.) |
| **System administrator** | Manages user accounts and role assignment. |
| **Teaching staff (assessor)** | Verifies domain understanding, correctness, and stack depth at demonstration. |

## 3. Scope

### 3.1 In scope - bounded contexts

The brief permits 3-5 bounded contexts and warns against more than 5. Four
are implemented:

| # | Bounded context | Responsibility | Key capability |
|---|---|---|---|
| 1 | **Artists** | Identity and biography of an artist; their tribal affiliation. | CRUD; lifespan validation; affiliation to a Tribe. |
| 2 | **Artifacts** | A piece of art and its physical/descriptive metadata, attributed to exactly one artist. | CRUD; mandatory attribution; dimension/era metadata. |
| 3 | **Tribes** | Aboriginal tribe / language group and its traditional Country. Subsumes the brief's *Maps* context as a capability rather than a separate BC. | CRUD; **PostGIS** territory polygons; "tribes near a point" spatial query. |
| 4 | **Users & Access** | Authentication and authorisation. Subsumes the brief's *Membership and Roles* context. | Register / login; JWT issuance; `User` / `Admin` roles; account management. |

*Maps* and *Membership and Roles* are listed in the brief as candidate
contexts; here they are deliberately folded into Tribes and Users
respectively, because their state and invariants are owned by those aggregates
(territory belongs to a tribe; a role belongs to a user) and splitting them
would add ceremony without a real boundary.

### 3.2 Out of scope

Exhibitions, Iconography/Symbols, Art Facts, Art Types, Art Styles (as a
context - `art_style` exists only as artifact metadata), Comments, Tags,
Art Eras. Interactive map rendering on the frontend. Production deployment,
email verification, password reset, refresh tokens, rate limiting. These are
acknowledged, not attempted, to keep the solution submittable and the
boundaries honest.

## 4. Ubiquitous language

The vocabulary below is used identically in conversation, the brief, this
document, the database, the API, and the code. Establishing it is an explicit
learning goal of the task.

| Term | Definition |
|---|---|
| **Artist** | A person who produced one or more artifacts. Has a display name, an optional lifespan (`birth_year` / `death_year`), an optional region and biography, and an optional tribal affiliation. |
| **Artifact** | A single piece of art. Always attributed to exactly one Artist. Carries type, style, medium, year, and physical dimensions in centimetres. |
| **Tribe** | An Aboriginal tribe or language group. Has a unique name, optional region/language group/description, and an optional **Territory**. |
| **Territory** | The traditional **Country** a Tribe is connected to, stored as a geographic `MultiPolygon` (WGS-84). `MultiPolygon`, not `Polygon`, because Country can be non-contiguous. Demo approximations, not authoritative boundaries. |
| **Country** | (Aboriginal sense) The land a tribe holds traditional connection to. Modelled by Territory. Capitalised to distinguish from a nation-state. |
| **Affiliation** | The link from an Artist to a Tribe. Optional and severable: deleting a Tribe un-affiliates its artists rather than deleting them. |
| **Attribution** | The link from an Artifact to its Artist. Mandatory and protective: an Artist with artifacts cannot be deleted. |
| **User** | An account that can authenticate. Has an email, a password (stored only as an Argon2id hash), and a Role. |
| **Role** | `User` or `Admin`. Determines whether a request may perform curatorial writes and account administration. |
| **Curatorial write** | Any create/update/delete on Artists, Artifacts, or Tribes. Admin-only. |
| **Self record** | The User a token belongs to. A user may read/update their own record without being an Admin, but may not change their own Role. |

## 5. Functional requirements

IDs are referenced by the Aggregate Design Canvases and the test suite.

### 5.1 Artists (FR-AR)

- **FR-AR-1** Anyone may list all artists and read a single artist (public).
- **FR-AR-2** An Admin may create an artist. `display_name` is required.
- **FR-AR-3** An Admin may update or delete an artist.
- **FR-AR-4** If both are given, `death_year` must be ≥ `birth_year`
  (enforced in DB by `artists_lifespan_valid` and in the handler).
- **FR-AR-5** An artist may be affiliated with at most one tribe; the
  affiliation is optional.
- **FR-AR-6** Deleting an artist that still has artifacts is rejected (409).

### 5.2 Artifacts (FR-AF)

- **FR-AF-1** Anyone may list and read artifacts (public).
- **FR-AF-2** An Admin may create an artifact; `title` and a valid
  `artist_id` are required.
- **FR-AF-3** An Admin may update or delete an artifact.
- **FR-AF-4** Creating/updating an artifact with an unknown `artist_id` is
  rejected (400/409, not a 500).

### 5.3 Tribes & Territory (FR-TR)

- **FR-TR-1** Anyone may list and read tribes (public).
- **FR-TR-2** An Admin may create/update/delete a tribe. `name` is required
  and unique (case-insensitive).
- **FR-TR-3** An Admin may set or clear a tribe's territory by submitting
  GeoJSON; invalid GeoJSON is rejected as a validation error (400), never a
  500.
- **FR-TR-4** Anyone may query "tribes whose territory is within *N* km of a
  latitude/longitude", returned nearest-first with distance in metres.
- **FR-TR-5** Territory is serialised back to the client as GeoJSON.

### 5.4 Users & Access (FR-US)

- **FR-US-1** Anyone may register; registration creates a `User`-role
  account and returns a token (no separate login step needed).
- **FR-US-2** Anyone may log in with email + password and receive a JWT.
- **FR-US-3** Passwords are stored only as Argon2id PHC hashes; never logged,
  never serialised in any response.
- **FR-US-4** A bearer token identifies the caller for all protected routes;
  missing/expired/tampered tokens yield 401.
- **FR-US-5** A caller may read and update their **own** user record.
- **FR-US-6** Only an Admin may list all users, read/update/delete an
  arbitrary user, or change any `role`.
- **FR-US-7** A non-admin attempting a role change on themselves is rejected
  (403) - no privilege escalation.
- **FR-US-8** An Admin cannot delete their own account (409) - the gallery
  cannot be left unadministrable.

## 6. Non-functional requirements

| ID | Requirement | How it is met |
|---|---|---|
| **NFR-1 Correctness** | Domain invariants enforced at the lowest safe layer. | DB `CHECK`/`UNIQUE`/FK constraints *and* handler validation; invariants restated in the ADCs. |
| **NFR-2 Security** | Credentials and access controlled to a recognised standard. | Argon2id hashing; HS256 JWT with 24 h TTL; role checks via typed extractors; secret rejected if < 32 bytes. |
| **NFR-3 Type safety** | Bugs caught before runtime, end to end. | Rust's type system; `sqlx` compile-time-checked SQL; OpenAPI-generated TypeScript so the frontend cannot drift from the API. |
| **NFR-4 Observability** | Failures are diagnosable, errors uniform. | `tracing` request logs; single `AppError` funnel producing a uniform `{"error": "..."}` body and correct status codes. |
| **NFR-5 Documentation** | The system is explainable without reading every line. | OpenAPI/Swagger UI, rustdoc on every item, this docs pack (BRD, ERD, C4). |
| **NFR-6 Testability** | Behaviour is verified, not asserted by hand. | `sqlx::test` per-test ephemeral databases; HTTP-level tests over `tower::oneshot` covering the auth/role matrix and spatial queries. |
| **NFR-7 Maintainability** | A new context can be added by pattern, not invention. | Every BC follows the same `model` / `repo` / `routes` / `mod` layout; one shared error type and state. |

## 7. Traceability to the brief

Mapping the brief's "what can be done differently to get maximum credit"
list to where each item is satisfied:

| Brief item | Satisfied by |
|---|---|
| Different Web API tech stack | Rust + Axum (not ASP.NET) |
| Different ORM library | `sqlx` (compile-time checked queries, not an ORM) |
| Complex DB structure (indexes, PG-specific functions) | CITEXT, GiST, functional index, `CHECK`s, trigger function, upserts - see [`erd.md`](./erd.md) |
| Front-end | Vue 3 SPA in `web/` |
| Complex documentation (DB + architecture diagrams, custom doc) | This pack + [`erd.md`](./erd.md) + [`architecture.md`](./architecture.md) + rustdoc + OpenAPI |
| Auth layer using uncovered approaches | Stateless JWT bearer auth with Argon2id, typed role extractors |
| PostGIS for geographical data | Tribe territory `geography(MultiPolygon,4326)` + spatial search |
| Testing framework | Rust integration tests on ephemeral databases |
| Complete BRD with Aggregate Design Canvases | This document + [`aggregate-design-canvases.md`](./aggregate-design-canvases.md) |
| Group work with git + task tool | Solo; git history + in-repo task tracking |

## 8. Assumptions & constraints

- Local-demo build: Docker Compose Postgres/PostGIS, a dev JWT secret in
  `.env.example`, permissive CORS. Not production-hardened by design.
- Territory polygons are coarse demo approximations seeded by the seed
  binary, **not** authoritative cultural boundaries; the frontend states this
  and renders an Acknowledgement of Country.
- One artist per artifact (no collaborative-work modelling).
- No soft deletes; deletes are physical, guarded by FK policies.
