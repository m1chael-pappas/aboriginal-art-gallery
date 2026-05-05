-- Wire the artists ↔ tribes relationship. SET NULL on tribe delete: an
-- artist outlives their tribe record disappearing — they just become
-- unaffiliated, not deleted.
ALTER TABLE artists
    ADD COLUMN tribe_id UUID REFERENCES tribes(id) ON DELETE SET NULL;

CREATE INDEX artists_tribe_id_idx ON artists(tribe_id);
