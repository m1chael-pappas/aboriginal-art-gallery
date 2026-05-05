CREATE TABLE artists (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    display_name TEXT NOT NULL,
    birth_year   SMALLINT,
    death_year   SMALLINT,
    region       TEXT,
    biography    TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT artists_lifespan_valid
        CHECK (death_year IS NULL OR birth_year IS NULL OR death_year >= birth_year)
);

CREATE TRIGGER artists_set_updated_at
    BEFORE UPDATE ON artists
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
