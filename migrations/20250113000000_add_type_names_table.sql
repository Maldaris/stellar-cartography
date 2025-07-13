-- Add type names table for EVE type ID to name mappings

CREATE TABLE IF NOT EXISTS type_names (
    type_id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Index for fast name lookups
CREATE INDEX IF NOT EXISTS idx_type_names_name ON type_names(name);
CREATE INDEX IF NOT EXISTS idx_type_names_name_lower ON type_names(LOWER(name)); 