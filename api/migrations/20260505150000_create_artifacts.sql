CREATE TABLE artifacts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title           TEXT NOT NULL,
    artist_id       UUID NOT NULL REFERENCES artists(id) ON DELETE RESTRICT,
    art_type        TEXT,
    art_style       TEXT,
    medium          TEXT,
    year_created    SMALLINT,
    height_cm       SMALLINT,
    width_cm        SMALLINT,
    depth_cm        SMALLINT,
    description     TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX artifacts_artist_id_idx ON artifacts(artist_id);

CREATE TRIGGER artifacts_set_updated_at
    BEFORE UPDATE ON artifacts
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
