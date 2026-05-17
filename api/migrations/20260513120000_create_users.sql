-- Case-insensitive text - `email CITEXT UNIQUE` rejects "Foo@x" and "foo@x"
-- as duplicates without app-level lowercasing, and the unique constraint
-- doubles as the login lookup index.
CREATE EXTENSION IF NOT EXISTS citext;

CREATE TABLE users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email         CITEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role          TEXT NOT NULL DEFAULT 'User',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT users_role_valid    CHECK (role IN ('User', 'Admin')),
    CONSTRAINT users_email_shape   CHECK (email LIKE '%_@_%.__%')
);

CREATE TRIGGER users_set_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
