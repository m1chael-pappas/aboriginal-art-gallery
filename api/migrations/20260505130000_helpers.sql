-- Generic trigger function for maintaining `updated_at` columns.
-- Defined once here, attached per-table by every BC migration that needs it:
--
--   CREATE TRIGGER <table>_set_updated_at
--       BEFORE UPDATE ON <table>
--       FOR EACH ROW EXECUTE FUNCTION set_updated_at();
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at := NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
