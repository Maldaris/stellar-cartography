-- Update schema to match JSON structure

-- First, rename the old security_status column to avoid conflicts
ALTER TABLE systems RENAME COLUMN security_status TO old_security_status;

-- Add new columns to systems table to store the full JSON structure
ALTER TABLE systems ADD COLUMN security_class TEXT;
ALTER TABLE systems ADD COLUMN security_status TEXT;
ALTER TABLE systems ADD COLUMN star_id INTEGER;
ALTER TABLE systems ADD COLUMN planet_ids TEXT; -- JSON array as text
ALTER TABLE systems ADD COLUMN planet_count_by_type TEXT; -- JSON object as text
ALTER TABLE systems ADD COLUMN neighbours TEXT; -- JSON array as text
ALTER TABLE systems ADD COLUMN stargates TEXT; -- JSON array as text
ALTER TABLE systems ADD COLUMN sovereignty TEXT;
ALTER TABLE systems ADD COLUMN disallowed_anchor_categories TEXT; -- JSON array as text
ALTER TABLE systems ADD COLUMN disallowed_anchor_groups TEXT; -- JSON array as text

-- Add new columns to constellations table
ALTER TABLE constellations ADD COLUMN solar_system_ids TEXT; -- JSON array as text
ALTER TABLE constellations ADD COLUMN constellation_faction_id INTEGER;
ALTER TABLE constellations ADD COLUMN constellation_sovereignty TEXT;

-- Add indices for the new searchable fields
CREATE INDEX IF NOT EXISTS idx_systems_security_class ON systems(security_class);
CREATE INDEX IF NOT EXISTS idx_systems_security_status ON systems(security_status);
CREATE INDEX IF NOT EXISTS idx_systems_star_id ON systems(star_id);
CREATE INDEX IF NOT EXISTS idx_constellations_faction_id ON constellations(constellation_faction_id); 