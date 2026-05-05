CREATE TABLE tribes (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT NOT NULL UNIQUE,
    region          TEXT,
    language_group  TEXT,
    description     TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Functional index for case-insensitive name lookups.
CREATE INDEX tribes_name_lower_idx ON tribes(LOWER(name));

CREATE TRIGGER tribes_set_updated_at
    BEFORE UPDATE ON tribes
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
