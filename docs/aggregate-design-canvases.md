# Aggregate Design Canvases - Aboriginal Art Gallery

> One canvas per aggregate, following the DDD Aggregate Design Canvas
> structure (name · description · state transitions · enforced invariants ·
> corrective policies · handled commands · created events · throughput).
> Requirement IDs (FR-…) trace to [`brd.md`](./brd.md); persistence to
> [`erd.md`](./erd.md).
>
> Each table is its own aggregate with the row as the aggregate root - small,
> consistency-focused boundaries rather than a single tangled graph.
> "Events" are conceptual: the system has no event bus, but naming the state
> changes clarifies the model and would be the seam for a future audit log or
> per-user features.

---

## Aggregate: Artist - *Artists* context

**Description.** The identity and biography of an artist, and their (optional)
affiliation to a Tribe. Root of attribution: artifacts point *at* an artist.

**State transitions.** `(none) → Registered → Updated* → Deleted`
Affiliation is part of Updated: `Unaffiliated ↔ AffiliatedTo(tribe)`.
`Deleted` is reachable only when the artist has zero artifacts.

**Enforced invariants.**
- `display_name` is non-empty (FR-AR-2).
- If both present, `death_year ≥ birth_year` - DB `artists_lifespan_valid`
  *and* handler validation (FR-AR-4).
- At most one tribe affiliation; `tribe_id` must reference an existing tribe
  (FR-AR-5).
- Cannot be deleted while any Artifact attributes it - FK `ON DELETE
  RESTRICT` → 409 (FR-AR-6).

**Corrective policies.** Affiliated tribe deleted → affiliation set to NULL
(artist becomes Unaffiliated, not deleted) via FK `ON DELETE SET NULL`.

**Handled commands.** `CreateArtist` · `UpdateArtist` (incl. set/clear
affiliation) · `DeleteArtist`. All Admin-only.

**Created events (conceptual).** `ArtistRegistered` · `ArtistUpdated` ·
`ArtistAffiliationChanged` · `ArtistDeleted`.

**Throughput.** Very low write rate (curatorial). High public read rate.
Decision-making complexity: low. Contention: negligible.

---

## Aggregate: Artifact - *Artifacts* context

**Description.** A single piece of art and its descriptive/physical metadata,
always attributed to exactly one Artist.

**State transitions.** `(none) → Catalogued → Updated* → Deleted`

**Enforced invariants.**
- `title` non-empty; `artist_id` present and references an existing artist -
  mandatory attribution (FR-AF-2).
- An unknown `artist_id` is a validation/conflict failure (400/409), never a
  500 (FR-AF-4): FK violation is mapped, not leaked.
- Dimensions and `year_created` are `SMALLINT`; out-of-range values are
  rejected before reaching the row.

**Corrective policies.** None. The protective side of the relationship lives
on the Artist aggregate (RESTRICT), so Artifact never needs to react to an
Artist disappearing - it cannot.

**Handled commands.** `CreateArtifact` · `UpdateArtifact` ·
`DeleteArtifact`. All Admin-only.

**Created events (conceptual).** `ArtifactCatalogued` · `ArtifactUpdated` ·
`ArtifactDeleted`.

**Throughput.** Low write rate; the highest-cardinality entity. Read-heavy.
Complexity: low.

---

## Aggregate: Tribe - *Tribes* context (subsumes *Maps*)

**Description.** An Aboriginal tribe / language group and its traditional
Country, the latter stored as real PostGIS geographic data. The *Maps* domain
capability is owned here because territory has no identity independent of its
tribe.

**State transitions.**
`(none) → Created → Updated* → Deleted`
Territory sub-lifecycle: `NoTerritory ↔ TerritorySet(multipolygon)`.

**Enforced invariants.**
- `name` non-empty and **unique, case-insensitive** - `UNIQUE` +
  `LOWER(name)` functional index (FR-TR-2).
- `territory`, when present, is a valid `geography(MultiPolygon, 4326)`;
  submitted GeoJSON that fails to parse is a 400, never a 500 (FR-TR-3).
- Deleting a tribe must not delete its affiliated artists - they are
  un-affiliated instead (the policy is enforced from the Artist FK side).

**Corrective policies.** On delete, dependent artists' `tribe_id` → NULL
(see Artist canvas). No corrective policy needed for territory.

**Handled commands.** `CreateTribe` · `UpdateTribe` · `DeleteTribe` ·
`SetTribeTerritory` · `ClearTribeTerritory` (Admin-only).
*Query, not a command:* `FindTribesNear(lat, lng, km)` - public, returns
nearest-first with metre distances (FR-TR-4).

**Created events (conceptual).** `TribeCreated` · `TribeUpdated` ·
`TribeTerritoryChanged` · `TribeDeleted`.

**Throughput.** Lowest write rate of all aggregates. Spatial query is read
-only and GiST-indexed. Complexity: medium (GeoJSON ↔ geography round-trip).

---

## Aggregate: User - *Users & Access* context (subsumes *Membership and Roles*)

**Description.** An authenticatable account: email, password (only ever an
Argon2id hash), and a Role. The authorisation authority for every protected
command in the other aggregates.

**State transitions.**
`(none) → Registered(role=User) → CredentialsUpdated* → Deleted`
Role sub-lifecycle: `User ↔ Admin` (Admin-driven only).
Session: `Registered/LoggedIn → TokenIssued` (token is stateless, not
persisted; no server-side session state to transition).

**Enforced invariants.**
- `email` unique and case-insensitive (`CITEXT UNIQUE`) and shaped like an
  address (`users_email_shape` CHECK) (FR-US-1).
- `role ∈ {User, Admin}` - DB `users_role_valid` CHECK; unknown DB value is a
  500 (data-integrity bug), not silently coerced.
- Password is never stored in clear, never serialised in any response, never
  logged - Argon2id PHC string only (FR-US-3).
- A caller may mutate only their **own** record unless Admin (FR-US-5/6).
- A non-admin cannot change any `role`, including their own - no privilege
  escalation (FR-US-7).
- An Admin cannot delete their own account - the system cannot be left
  unadministrable (FR-US-8).

**Corrective policies.** None automated. The self-delete and
self-escalation guards are *preventive* (reject the command) rather than
corrective (no compensating action exists or is desirable).

**Handled commands.** `Register` · `Login` (→ `IssueToken`) ·
`UpdateOwnCredentials` · `AdminUpdateUser` (incl. `ChangeRole`) ·
`AdminDeleteUser`.

**Created events (conceptual).** `UserRegistered` · `TokenIssued` ·
`UserCredentialsUpdated` · `UserRoleChanged` · `UserDeleted`.

**Throughput.** Low overall. `Login` is the hot path and intentionally
expensive (Argon2id is CPU-hard by design). Stateless tokens mean no
session-store contention. Complexity: medium - the authorisation rules are
the most invariant-dense in the system.

---

## Cross-aggregate consistency

All consistency is **transactional within a single aggregate**; there are no
multi-aggregate transactions. The only cross-aggregate rules are expressed
declaratively as foreign-key policies (RESTRICT / SET NULL) so the database,
not application code, guarantees them even under concurrent writes. This
keeps every aggregate boundary a true consistency boundary and avoids
distributed-transaction or eventual-consistency machinery the task does not
require.
