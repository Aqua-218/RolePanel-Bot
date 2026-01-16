-- Ensure panel names are unique per guild (not globally)

-- Drop legacy unique constraints/indexes that might enforce global uniqueness on name
ALTER TABLE panels DROP CONSTRAINT IF EXISTS panels_name_key;
ALTER TABLE panels DROP CONSTRAINT IF EXISTS uq_panels_name;
DROP INDEX IF EXISTS idx_panels_name;
DROP INDEX IF EXISTS panels_name_idx;

-- Create (or ensure) per-guild unique index
CREATE UNIQUE INDEX IF NOT EXISTS idx_panels_guild_name ON panels(guild_id, name);
