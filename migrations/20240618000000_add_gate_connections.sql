-- Create gate connections table for NPC gates and jump bridges
CREATE TABLE IF NOT EXISTS gate_connections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_system_id INTEGER NOT NULL,
    to_system_id INTEGER NOT NULL,
    connection_type TEXT NOT NULL DEFAULT 'stargate', -- stargate, jump_bridge, wormhole
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (from_system_id) REFERENCES systems(id),
    FOREIGN KEY (to_system_id) REFERENCES systems(id),
    UNIQUE(from_system_id, to_system_id, connection_type)
);

-- Indices for fast gate connection lookups
CREATE INDEX IF NOT EXISTS idx_gate_connections_from ON gate_connections(from_system_id);
CREATE INDEX IF NOT EXISTS idx_gate_connections_to ON gate_connections(to_system_id);
CREATE INDEX IF NOT EXISTS idx_gate_connections_type ON gate_connections(connection_type);

-- Add bidirectional index for efficient queries in both directions
CREATE INDEX IF NOT EXISTS idx_gate_connections_bidirectional ON gate_connections(from_system_id, to_system_id); 