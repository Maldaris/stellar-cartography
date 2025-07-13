use anyhow::Result;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool, Row};
use tokio::fs;
use tracing::{info, warn};
use serde_json;
use std::path::Path;
use crate::models::{SolarSystem, Constellation, SystemHierarchy, SystemInfo, RegionInfo, ConstellationInfo, GateConnection, SystemConnections, CompleteSystemHierarchy, ConstellationWithSystems, RegionWithConstellations, SecurityInfo, CelestialInfo, NavigationInfo, SystemMetadata, ConstellationMetadata, TypeName, TypeNameResponse};

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_path: &str) -> Result<Self> {
        // Ensure the directory exists
        if let Some(parent) = std::path::Path::new(database_path).parent() {
            fs::create_dir_all(parent).await?;
        }

        // Create connection pool with create if missing
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&format!("sqlite://{}?mode=rwc", database_path))
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;

        info!("Database initialized at {}", database_path);

        Ok(Self { pool })
    }

    #[allow(dead_code)]
    pub async fn get_system_name(&self, system_id: u32) -> Result<Option<String>> {
        let row = sqlx::query("SELECT name FROM systems WHERE id = ?")
            .bind(system_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.get("name")))
    }

    #[allow(dead_code)]
    pub async fn get_region_name(&self, region_id: u32) -> Result<Option<String>> {
        let row = sqlx::query("SELECT name FROM regions WHERE id = ?")
            .bind(region_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.get("name")))
    }

    #[allow(dead_code)]
    pub async fn get_constellation_name(&self, constellation_id: u32) -> Result<Option<String>> {
        let row = sqlx::query("SELECT name FROM constellations WHERE id = ?")
            .bind(constellation_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.get("name")))
    }

    pub async fn get_type_name(&self, type_id: u32) -> Result<Option<String>> {
        let row = sqlx::query("SELECT name FROM type_names WHERE type_id = ?")
            .bind(type_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.get("name")))
    }

    pub async fn search_type_names(&self, query: &str, limit: usize) -> Result<TypeNameResponse> {
        let limit = limit.min(100).max(1);
        
        let rows = sqlx::query(
            "SELECT type_id, name FROM type_names 
             WHERE LOWER(name) LIKE LOWER(?) 
             ORDER BY name ASC 
             LIMIT ?"
        )
        .bind(format!("%{}%", query))
        .bind(limit as i32)
        .fetch_all(&self.pool)
        .await?;

        let type_names: Vec<TypeName> = rows
            .into_iter()
            .map(|row| TypeName {
                type_id: row.get::<i32, _>("type_id") as u32,
                name: row.get("name"),
            })
            .collect();

        let total_found = type_names.len();

        Ok(TypeNameResponse {
            type_names,
            query: query.to_string(),
            total_found,
        })
    }

    pub async fn get_all_type_names(&self, limit: usize, offset: usize) -> Result<Vec<TypeName>> {
        let rows = sqlx::query(
            "SELECT type_id, name FROM type_names 
             ORDER BY type_id ASC 
             LIMIT ? OFFSET ?"
        )
        .bind(limit as i32)
        .bind(offset as i32)
        .fetch_all(&self.pool)
        .await?;

        let type_names: Vec<TypeName> = rows
            .into_iter()
            .map(|row| TypeName {
                type_id: row.get::<i32, _>("type_id") as u32,
                name: row.get("name"),
            })
            .collect();

        Ok(type_names)
    }

    #[allow(dead_code)]
    pub async fn search_systems(&self, query: &str, limit: u32) -> Result<Vec<(u32, String)>> {
        let rows = sqlx::query("SELECT id, name FROM systems WHERE name LIKE ? ORDER BY name LIMIT ?")
            .bind(format!("%{}%", query))
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| (row.get("id"), row.get("name")))
            .collect())
    }

    pub async fn load_all_systems(&self) -> Result<Vec<(u32, SolarSystem, String)>> {
        let rows = sqlx::query(
            "SELECT id, name, center_x, center_y, center_z, region_id, constellation_id, faction_id,
                    security_class, security_status, star_id, planet_ids, planet_count_by_type,
                    neighbours, stargates, sovereignty, disallowed_anchor_categories, disallowed_anchor_groups
             FROM systems ORDER BY id"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut systems = Vec::new();
        for row in rows {
            let id: u32 = row.get("id");
            let name: String = row.get("name");
            let center_x: f64 = row.get("center_x");
            let center_y: f64 = row.get("center_y");  
            let center_z: f64 = row.get("center_z");
            let region_id: Option<u32> = row.get("region_id");
            let constellation_id: Option<u32> = row.get("constellation_id");
            let faction_id: Option<u32> = row.get("faction_id");

            // Parse JSON fields with fallbacks
            let planet_ids: Vec<u32> = row.get::<Option<String>, _>("planet_ids")
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            
            let planet_count_by_type: std::collections::HashMap<String, u32> = row.get::<Option<String>, _>("planet_count_by_type")
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            
            let neighbours: Vec<u32> = row.get::<Option<String>, _>("neighbours")
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            
            let stargates: Vec<u32> = row.get::<Option<String>, _>("stargates")
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            
            let disallowed_anchor_categories: Vec<String> = row.get::<Option<String>, _>("disallowed_anchor_categories")
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            
            let disallowed_anchor_groups: Vec<String> = row.get::<Option<String>, _>("disallowed_anchor_groups")
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();

            let system = SolarSystem {
                id,
                name: name.clone(),
                center: [center_x, center_y, center_z],
                region_id,
                constellation_id,
                security: SecurityInfo {
                    class: row.get("security_class"),
                    status: row.get("security_status"),
                },
                celestials: CelestialInfo {
                    star_id: row.get("star_id"),
                    planet_ids,
                    planet_count_by_type,
                },
                navigation: NavigationInfo {
                    neighbours,
                    stargates,
                },
                metadata: SystemMetadata {
                    faction_id,
                    sovereignty: row.get("sovereignty"),
                    disallowed_anchor_categories,
                    disallowed_anchor_groups,
                },
            };

            systems.push((id, system, name));
        }

        Ok(systems)
    }

    pub async fn load_all_regions(&self) -> Result<Vec<(u32, String)>> {
        let rows = sqlx::query("SELECT id, name FROM regions ORDER BY id")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| (row.get("id"), row.get("name")))
            .collect())
    }

    pub async fn load_all_constellations(&self) -> Result<Vec<(u32, String, u32)>> {
        let rows = sqlx::query("SELECT id, name, region_id FROM constellations ORDER BY id")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| (row.get("id"), row.get("name"), row.get("region_id")))
            .collect())
    }

    pub async fn is_empty(&self) -> Result<bool> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM systems")
            .fetch_one(&self.pool)
            .await?;
        
        Ok(count.0 == 0)
    }

    pub async fn needs_update(&self, data_dir: &str) -> Result<bool> {
        // Check if database is empty
        if self.is_empty().await? {
            return Ok(true);
        }

        // Check file modification times
        let stellar_cartography_path = Path::new(data_dir).join("stellar_cartography.json");
        let labels_path = Path::new(data_dir).join("stellar_labels.json");
        
        if !stellar_cartography_path.exists() || !labels_path.exists() {
            warn!("Required data files missing, database may be outdated");
            return Ok(false); // Don't update if source files are missing
        }

        // Get the most recent modification time of our source files
        let stellar_cartography_modified = fs::metadata(&stellar_cartography_path).await?.modified()?;
        let labels_modified = fs::metadata(&labels_path).await?.modified()?;
        let latest_file_time = stellar_cartography_modified.max(labels_modified);

        // Check when the database was last updated (using a metadata table)
        let last_update: Option<String> = sqlx::query_scalar(
            "SELECT value FROM metadata WHERE key = 'last_update' LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(last_update_str) = last_update {
            if let Ok(last_update_time) = last_update_str.parse::<i64>() {
                let db_time = std::time::UNIX_EPOCH + std::time::Duration::from_secs(last_update_time as u64);
                return Ok(latest_file_time > db_time);
            }
        }

        // If no metadata found, assume we need to update
        Ok(true)
    }

    pub async fn seed_from_json(&self, data_dir: &str) -> Result<()> {
        info!("Seeding database from JSON files...");

        // Create metadata table if it doesn't exist
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )"
        )
        .execute(&self.pool)
        .await?;

        // Clear existing data in a transaction
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM gate_connections").execute(&mut *tx).await?;
        sqlx::query("DELETE FROM systems").execute(&mut *tx).await?;
        sqlx::query("DELETE FROM constellations").execute(&mut *tx).await?;
        sqlx::query("DELETE FROM regions").execute(&mut *tx).await?;
        sqlx::query("DELETE FROM type_names").execute(&mut *tx).await?;
        tx.commit().await?;

        // Load stellar labels
        let labels_path = Path::new(data_dir).join("stellar_labels.json");
        let labels_content = fs::read_to_string(&labels_path).await?;
        let labels: serde_json::Value = serde_json::from_str(&labels_content)?;
        
        let empty_map = serde_json::Map::new();
        let system_labels = labels.get("systems").and_then(|s| s.as_object()).unwrap_or(&empty_map);
        let region_labels = labels.get("regions").and_then(|r| r.as_object()).unwrap_or(&empty_map);
        let constellation_labels = labels.get("constellations").and_then(|c| c.as_object()).unwrap_or(&empty_map);

        // Load stellar cartography data
        let stellar_cartography_path = Path::new(data_dir).join("stellar_cartography.json");
        let stellar_cartography_content = fs::read_to_string(&stellar_cartography_path).await?;
        let starmap: serde_json::Value = serde_json::from_str(&stellar_cartography_content)?;

        // Debug: Log the top-level keys in the JSON
        if let Some(obj) = starmap.as_object() {
            info!("Top-level keys in stellar_cartography.json: {:?}", obj.keys().collect::<Vec<_>>());
        }

        let mut systems_inserted = 0;
        let mut regions_inserted = 0;
        let mut constellations_inserted = 0;
        let mut connections_inserted = 0;

        // Insert regions
        if let Some(regions_data) = starmap.get("regions").and_then(|r| r.as_object()) {
            info!("Found {} regions in data", regions_data.len());
            for (id_str, _region_data) in regions_data {
                if let Ok(region_id) = id_str.parse::<u32>() {
                    let default_name = format!("Region_{}", region_id);
                    let name = region_labels.get(&region_id.to_string())
                        .and_then(|n| n.as_str())
                        .unwrap_or(&default_name);

                    sqlx::query("INSERT INTO regions (id, name) VALUES (?, ?)")
                        .bind(region_id)
                        .bind(name)
                        .execute(&self.pool)
                        .await?;
                    
                    regions_inserted += 1;
                }
            }
        } else {
            warn!("No regions found in stellar cartography data or not an object");
        }

        // Insert constellations
        if let Some(constellations_data) = starmap.get("constellations").and_then(|c| c.as_object()) {
            info!("Found {} constellations in data", constellations_data.len());
            for (id_str, constellation_data) in constellations_data {
                if let Ok(constellation_id) = id_str.parse::<u32>() {
                    match serde_json::from_value::<Constellation>(constellation_data.clone()) {
                        Ok(constellation) => {
                            let name = constellation_labels.get(&constellation_id.to_string())
                                .and_then(|n| n.as_str())
                                .unwrap_or(&constellation.name);

                            sqlx::query("INSERT INTO constellations (id, name, region_id, solar_system_ids, constellation_faction_id, constellation_sovereignty) VALUES (?, ?, ?, ?, ?, ?)")
                                .bind(constellation_id)
                                .bind(name)
                                .bind(constellation.region_id)
                                .bind(serde_json::to_string(&constellation.solar_system_ids).unwrap_or_default())
                                .bind(constellation.metadata.faction_id)
                                .bind(constellation.metadata.sovereignty)
                                .execute(&self.pool)
                                .await?;
                            
                            constellations_inserted += 1;
                        }
                        Err(e) => {
                            warn!("Failed to parse constellation {}: {}", constellation_id, e);
                        }
                    }
                }
            }
        } else {
            warn!("No constellations found in stellar cartography data or not an object");
        }

        // Insert systems
        if let Some(systems_data) = starmap.get("systems").and_then(|s| s.as_object()) {
            info!("Found {} systems in data", systems_data.len());
            for (id_str, system_data) in systems_data {
                if let Ok(system_id) = id_str.parse::<u32>() {
                    match serde_json::from_value::<SolarSystem>(system_data.clone()) {
                        Ok(system) => {
                            let name = system_labels.get(&system_id.to_string())
                                .and_then(|n| n.as_str())
                                .unwrap_or(&system.name);

                            sqlx::query(
                                "INSERT INTO systems (id, name, center_x, center_y, center_z, region_id, constellation_id, faction_id,
                                                    security_class, security_status, star_id, planet_ids, planet_count_by_type,
                                                    neighbours, stargates, sovereignty, disallowed_anchor_categories, disallowed_anchor_groups) 
                                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
                            )
                            .bind(system_id)
                            .bind(name)
                            .bind(system.center[0])
                            .bind(system.center[1])
                            .bind(system.center[2])
                            .bind(system.region_id)
                            .bind(system.constellation_id)
                            .bind(system.metadata.faction_id)
                            .bind(system.security.class)
                            .bind(system.security.status)
                            .bind(system.celestials.star_id)
                            .bind(serde_json::to_string(&system.celestials.planet_ids).unwrap_or_default())
                            .bind(serde_json::to_string(&system.celestials.planet_count_by_type).unwrap_or_default())
                            .bind(serde_json::to_string(&system.navigation.neighbours).unwrap_or_default())
                            .bind(serde_json::to_string(&system.navigation.stargates).unwrap_or_default())
                            .bind(system.metadata.sovereignty)
                            .bind(serde_json::to_string(&system.metadata.disallowed_anchor_categories).unwrap_or_default())
                            .bind(serde_json::to_string(&system.metadata.disallowed_anchor_groups).unwrap_or_default())
                            .execute(&self.pool)
                            .await?;
                            
                            systems_inserted += 1;
                        }
                        Err(e) => {
                            warn!("Failed to parse system {}: {}", system_id, e);
                        }
                    }
                }
            }
        } else {
            warn!("No systems found in stellar cartography data or not an object");
        }

        // Insert gate connections
        if let Some(systems_data) = starmap.get("systems").and_then(|s| s.as_object()) {
            for (id_str, system_data) in systems_data {
                if let Ok(from_system_id) = id_str.parse::<u32>() {
                    match serde_json::from_value::<SolarSystem>(system_data.clone()) {
                        Ok(system) => {
                            // Use navigation.neighbours from the new structure
                            for to_system_id in &system.navigation.neighbours {
                                // Insert bidirectional connection (only if from_system_id <= to_system_id to avoid duplicates)
                                if from_system_id <= *to_system_id {
                                    sqlx::query(
                                        "INSERT INTO gate_connections (from_system_id, to_system_id, connection_type) 
                                         VALUES (?, ?, ?)"
                                    )
                                    .bind(from_system_id)
                                    .bind(*to_system_id)
                                    .bind("stargate")
                                    .execute(&self.pool)
                                    .await?;
                                    
                                    connections_inserted += 1;
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse system {} for connections: {}", from_system_id, e);
                        }
                    }
                }
            }
        }

        // Update metadata with current timestamp
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        // Load type names from extracted data
        let mut type_names_inserted = 0;
        let type_names_path = Path::new(data_dir).join("type_names_all.json");
        if type_names_path.exists() {
            info!("Loading type names from type_names_all.json...");
            let type_names_content = fs::read_to_string(&type_names_path).await?;
            let type_names_data: serde_json::Value = serde_json::from_str(&type_names_content)?;
            
            if let Some(type_names_obj) = type_names_data.as_object() {
                info!("Found {} type names in data", type_names_obj.len());
                
                for (type_id_str, name_value) in type_names_obj {
                    if let (Ok(type_id), Some(name)) = (type_id_str.parse::<u32>(), name_value.as_str()) {
                        sqlx::query(
                            "INSERT OR REPLACE INTO type_names (type_id, name) VALUES (?, ?)"
                        )
                        .bind(type_id)
                        .bind(name)
                        .execute(&self.pool)
                        .await?;
                        
                        type_names_inserted += 1;
                    }
                }
            }
        } else {
            warn!("Type names file not found at {:?}, skipping type names loading", type_names_path);
        }

        sqlx::query(
            "INSERT OR REPLACE INTO metadata (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)"
        )
        .bind("last_update")
        .bind(now.to_string())
        .execute(&self.pool)
        .await?;

        info!(
            "Database seeded successfully: {} systems, {} regions, {} constellations, {} gate connections, {} type names",
            systems_inserted, regions_inserted, constellations_inserted, connections_inserted, type_names_inserted
        );

        Ok(())
    }

    /// Get complete hierarchical information for a system (system -> constellation -> region)
    pub async fn get_system_hierarchy(&self, system_id: u32) -> Result<Option<SystemHierarchy>> {
        let row = sqlx::query(
            "SELECT s.id, s.name as system_name, s.x, s.y, s.z, s.region_id, s.constellation_id, s.faction_id,
                    c.name as constellation_name, c.region_id as constellation_region_id,
                    r.name as region_name
             FROM systems s
             LEFT JOIN constellations c ON s.constellation_id = c.id
             LEFT JOIN regions r ON s.region_id = r.id
             WHERE s.id = ?"
        )
        .bind(system_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let system = SystemInfo {
                id: row.get("id"),
                name: row.get("system_name"),
                center: [
                    row.get::<Option<f64>, _>("x").unwrap_or(0.0),
                    row.get::<Option<f64>, _>("y").unwrap_or(0.0),
                    row.get::<Option<f64>, _>("z").unwrap_or(0.0),
                ],
                region_id: row.get("region_id"),
                constellation_id: row.get("constellation_id"),
                faction_id: row.get("faction_id"),
                distance: None,
            };

            let constellation = if let (Some(constellation_id), Some(constellation_name)) = 
                (row.get::<Option<u32>, _>("constellation_id"), row.get::<Option<String>, _>("constellation_name")) {
                Some(ConstellationInfo {
                    id: constellation_id,
                    name: constellation_name,
                    region_id: row.get::<Option<u32>, _>("constellation_region_id").unwrap_or(0),
                })
            } else {
                None
            };

            let region = if let (Some(region_id), Some(region_name)) = 
                (row.get::<Option<u32>, _>("region_id"), row.get::<Option<String>, _>("region_name")) {
                Some(RegionInfo {
                    id: region_id,
                    name: region_name,
                })
            } else {
                None
            };

            Ok(Some(SystemHierarchy {
                system,
                constellation,
                region,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get complete hierarchical information with all related systems and constellations
    pub async fn get_complete_system_hierarchy(&self, system_id: u32) -> Result<Option<CompleteSystemHierarchy>> {
        use crate::models::*;

        // First get the target system
        let target_system_row = sqlx::query(
            "SELECT s.id, s.name as system_name, s.x, s.y, s.z, s.region_id, s.constellation_id, s.faction_id
             FROM systems s
             WHERE s.id = ?"
        )
        .bind(system_id)
        .fetch_optional(&self.pool)
        .await?;

        let target_system = if let Some(row) = target_system_row {
            SystemInfo {
                id: row.get("id"),
                name: row.get("system_name"),
                center: [
                    row.get::<Option<f64>, _>("x").unwrap_or(0.0),
                    row.get::<Option<f64>, _>("y").unwrap_or(0.0),
                    row.get::<Option<f64>, _>("z").unwrap_or(0.0),
                ],
                region_id: row.get("region_id"),
                constellation_id: row.get("constellation_id"),
                faction_id: row.get("faction_id"),
                distance: None,
            }
        } else {
            return Ok(None);
        };

        let target_constellation = if let Some(constellation_id) = target_system.constellation_id {
            // Get constellation info
            let constellation_row = sqlx::query(
                "SELECT id, name, region_id FROM constellations WHERE id = ?"
            )
            .bind(constellation_id)
            .fetch_optional(&self.pool)
            .await?;

            if let Some(constellation_row) = constellation_row {
                // Get all systems in this constellation
                let systems_in_constellation = sqlx::query(
                    "SELECT id, name, x, y, z, region_id, constellation_id, faction_id
                     FROM systems 
                     WHERE constellation_id = ?"
                )
                .bind(constellation_id)
                .fetch_all(&self.pool)
                .await?;

                let systems = systems_in_constellation.into_iter()
                    .map(|row| SystemInfo {
                        id: row.get("id"),
                        name: row.get("name"),
                        center: [
                            row.get::<Option<f64>, _>("x").unwrap_or(0.0),
                            row.get::<Option<f64>, _>("y").unwrap_or(0.0),
                            row.get::<Option<f64>, _>("z").unwrap_or(0.0),
                        ],
                        region_id: row.get("region_id"),
                        constellation_id: row.get("constellation_id"),
                        faction_id: row.get("faction_id"),
                        distance: None,
                    })
                    .collect();

                Some(ConstellationWithSystems {
                    id: constellation_row.get("id"),
                    name: constellation_row.get("name"),
                    region_id: constellation_row.get("region_id"),
                    systems,
                })
            } else {
                None
            }
        } else {
            None
        };

        let target_region = if let Some(region_id) = target_system.region_id {
            // Get region info
            let region_row = sqlx::query(
                "SELECT id, name FROM regions WHERE id = ?"
            )
            .bind(region_id)
            .fetch_optional(&self.pool)
            .await?;

            if let Some(region_row) = region_row {
                // Get all constellations in this region
                let constellations_in_region = sqlx::query(
                    "SELECT id, name, region_id FROM constellations WHERE region_id = ?"
                )
                .bind(region_id)
                .fetch_all(&self.pool)
                .await?;

                let mut constellations = Vec::new();
                for constellation_row in constellations_in_region {
                    let constellation_id: u32 = constellation_row.get("id");
                    
                    // Get all systems in this constellation
                    let systems_in_constellation = sqlx::query(
                        "SELECT id, name, x, y, z, region_id, constellation_id, faction_id
                         FROM systems 
                         WHERE constellation_id = ?"
                    )
                    .bind(constellation_id)
                    .fetch_all(&self.pool)
                    .await?;

                    let systems = systems_in_constellation.into_iter()
                        .map(|row| SystemInfo {
                            id: row.get("id"),
                            name: row.get("name"),
                            center: [
                                row.get::<Option<f64>, _>("x").unwrap_or(0.0),
                                row.get::<Option<f64>, _>("y").unwrap_or(0.0),
                                row.get::<Option<f64>, _>("z").unwrap_or(0.0),
                            ],
                            region_id: row.get("region_id"),
                            constellation_id: row.get("constellation_id"),
                            faction_id: row.get("faction_id"),
                            distance: None,
                        })
                        .collect();

                    constellations.push(ConstellationWithSystems {
                        id: constellation_id,
                        name: constellation_row.get("name"),
                        region_id: constellation_row.get("region_id"),
                        systems,
                    });
                }

                Some(RegionWithConstellations {
                    id: region_row.get("id"),
                    name: region_row.get("name"),
                    constellations,
                })
            } else {
                None
            }
        } else {
            None
        };

        Ok(Some(CompleteSystemHierarchy {
            target_system,
            target_constellation,
            target_region,
        }))
    }

    /// Get gate connections for a specific system
    pub async fn get_system_connections(&self, system_id: u32, connection_type: Option<&str>) -> Result<Vec<GateConnection>> {
        // Use a simpler approach without dynamic query building
        let rows = if let Some(conn_type) = connection_type {
            sqlx::query(
                "SELECT id, from_system_id, to_system_id, connection_type 
                 FROM gate_connections 
                 WHERE (from_system_id = ? OR to_system_id = ?) AND connection_type = ?"
            )
            .bind(system_id)
            .bind(system_id)
            .bind(conn_type)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                "SELECT id, from_system_id, to_system_id, connection_type 
                 FROM gate_connections 
                 WHERE from_system_id = ? OR to_system_id = ?"
            )
            .bind(system_id)
            .bind(system_id)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows
            .into_iter()
            .map(|row| GateConnection {
                id: row.get("id"),
                from_system_id: row.get("from_system_id"),
                to_system_id: row.get("to_system_id"),
                connection_type: row.get("connection_type"),
            })
            .collect())
    }

        /// Get gate connections for multiple systems in bulk
    pub async fn get_bulk_connections(&self, system_ids: &[u32], connection_type: Option<&str>) -> Result<Vec<SystemConnections>> {
        if system_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = system_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        
        let mut query = format!(
            "SELECT id, from_system_id, to_system_id, connection_type 
             FROM gate_connections 
             WHERE from_system_id IN ({}) OR to_system_id IN ({})",
            placeholders, placeholders
        );

        if connection_type.is_some() {
            query.push_str(" AND connection_type = ?");
        }

        let mut query_builder = sqlx::query(&query);
        
        // Bind system_ids twice (for from_system_id and to_system_id)
        for &system_id in system_ids {
            query_builder = query_builder.bind(system_id);
        }
        for &system_id in system_ids {
            query_builder = query_builder.bind(system_id);
        }
        
        // Bind connection_type if provided
        if let Some(conn_type) = connection_type {
            query_builder = query_builder.bind(conn_type);
        }

        let rows = query_builder.fetch_all(&self.pool).await?;

        // Group connections by system_id
        let mut system_connections_map: std::collections::HashMap<u32, Vec<GateConnection>> = 
            std::collections::HashMap::new();

        for row in rows {
            let connection = GateConnection {
                id: row.get("id"),
                from_system_id: row.get("from_system_id"),
                to_system_id: row.get("to_system_id"),
                connection_type: row.get("connection_type"),
            };

            // Add to both from and to systems
            system_connections_map
                .entry(connection.from_system_id)
                .or_insert_with(Vec::new)
                .push(connection.clone());
            
            if connection.from_system_id != connection.to_system_id {
                system_connections_map
                    .entry(connection.to_system_id)
                    .or_insert_with(Vec::new)
                    .push(connection);
            }
        }

        // Convert to the response format
        let mut result = Vec::new();
        for &system_id in system_ids {
            let connections = system_connections_map
                .remove(&system_id)
                .unwrap_or_else(Vec::new);

            result.push(SystemConnections {
                system_id,
                connections,
            });
        }

        Ok(result)
    }

    /// Get all gate connections with pagination
    pub async fn get_all_connections(&self, limit: usize, offset: usize, connection_type: Option<&str>) -> Result<(Vec<GateConnection>, usize)> {
        // Get total count first
        let total_query = if let Some(conn_type) = connection_type {
            sqlx::query_scalar("SELECT COUNT(*) FROM gate_connections WHERE connection_type = ?")
                .bind(conn_type)
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM gate_connections")
        };

        let total_count: i64 = total_query.fetch_one(&self.pool).await?;

        // Get paginated connections
        let connections_query = if let Some(conn_type) = connection_type {
            sqlx::query(
                "SELECT id, from_system_id, to_system_id, connection_type 
                 FROM gate_connections 
                 WHERE connection_type = ?
                 ORDER BY id
                 LIMIT ? OFFSET ?"
            )
            .bind(conn_type)
            .bind(limit as i64)
            .bind(offset as i64)
        } else {
            sqlx::query(
                "SELECT id, from_system_id, to_system_id, connection_type 
                 FROM gate_connections 
                 ORDER BY id
                 LIMIT ? OFFSET ?"
            )
            .bind(limit as i64)
            .bind(offset as i64)
        };

        let rows = connections_query.fetch_all(&self.pool).await?;

        let connections = rows
            .into_iter()
            .map(|row| GateConnection {
                id: row.get("id"),
                from_system_id: row.get("from_system_id"),
                to_system_id: row.get("to_system_id"),
                connection_type: row.get("connection_type"),
            })
            .collect();

        Ok((connections, total_count as usize))
    }
} 