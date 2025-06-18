-- Initial schema for stellar cartography

-- Systems table for storing solar system metadata
CREATE TABLE IF NOT EXISTS systems (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    region_id INTEGER,
    constellation_id INTEGER,
    faction_id INTEGER,
    security_status REAL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Regions table
CREATE TABLE IF NOT EXISTS regions (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Constellations table
CREATE TABLE IF NOT EXISTS constellations (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    region_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (region_id) REFERENCES regions(id)
);

-- Indices for fast lookups
CREATE INDEX IF NOT EXISTS idx_systems_name ON systems(name);
CREATE INDEX IF NOT EXISTS idx_systems_region ON systems(region_id);
CREATE INDEX IF NOT EXISTS idx_systems_constellation ON systems(constellation_id);
CREATE INDEX IF NOT EXISTS idx_constellations_region ON constellations(region_id); 