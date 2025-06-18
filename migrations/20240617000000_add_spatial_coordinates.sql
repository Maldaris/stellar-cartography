-- Add spatial coordinates to systems table

ALTER TABLE systems ADD COLUMN center_x REAL;
ALTER TABLE systems ADD COLUMN center_y REAL;
ALTER TABLE systems ADD COLUMN center_z REAL;

-- Add indices for spatial queries (although we'll primarily use the KD-tree)
CREATE INDEX IF NOT EXISTS idx_systems_center_x ON systems(center_x);
CREATE INDEX IF NOT EXISTS idx_systems_center_y ON systems(center_y);
CREATE INDEX IF NOT EXISTS idx_systems_center_z ON systems(center_z); 