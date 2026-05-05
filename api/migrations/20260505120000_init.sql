-- Enable PostGIS up front. Harmless if unused, and saves a migration later
-- if/when Tribes gains a territory polygon column.
CREATE EXTENSION IF NOT EXISTS postgis;

-- Smoke-test table to verify the migration pipeline ran end-to-end.
CREATE TABLE _meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT INTO _meta (key, value) VALUES ('schema_initialized', 'true');
