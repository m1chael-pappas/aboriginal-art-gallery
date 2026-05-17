-- Adds an optional traditional-Country polygon to each tribe.
--
-- `geography(MultiPolygon, 4326)` stores lat/lng on the WGS-84 spheroid, so
-- ST_Distance/ST_DWithin return real-world metres without per-call projection
-- arithmetic. MultiPolygon (not Polygon) because traditional Country can be
-- non-contiguous - islands, separated regions, etc.
--
-- These are demo approximations seeded by the seed binary, not authoritative
-- boundaries.

ALTER TABLE tribes
    ADD COLUMN territory geography(MultiPolygon, 4326);

-- GiST index for ST_DWithin / ST_Intersects spatial lookups. Without it,
-- "tribes within N km of point" would table-scan and call ST_Distance on
-- every row.
CREATE INDEX tribes_territory_gix
    ON tribes USING GIST (territory);
