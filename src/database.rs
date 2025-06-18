use anyhow::Result;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool, Row};
use tokio::fs;
use tracing::{info, warn};
use serde_json;
use std::path::Path;
use crate::models::{SolarSystem, Constellation};

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

    pub async fn get_system_name(&self, system_id: u32) -> Result<Option<String>> {
        let row = sqlx::query("SELECT name FROM systems WHERE id = ?")
            .bind(system_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.get("name")))
    }

    pub async fn get_region_name(&self, region_id: u32) -> Result<Option<String>> {
        let row = sqlx::query("SELECT name FROM regions WHERE id = ?")
            .bind(region_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.get("name")))
    }

    pub async fn get_constellation_name(&self, constellation_id: u32) -> Result<Option<String>> {
        let row = sqlx::query("SELECT name FROM constellations WHERE id = ?")
            .bind(constellation_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.get("name")))
    }

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
            "SELECT id, name, center_x, center_y, center_z, region_id, constellation_id, faction_id 
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

            let system = SolarSystem {
                center: [center_x, center_y, center_z],
                region_id,
                constellation_id,
                faction_id,
                planet_item_ids: Vec::new(), // We might not store this in DB
                planet_count_by_type: std::collections::HashMap::new(),
                neighbours: Vec::new(), // We might not store this in DB
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
        let starmap_path = Path::new(data_dir).join("starmapcache.json");
        let labels_path = Path::new(data_dir).join("stellar_labels.json");
        
        if !starmap_path.exists() || !labels_path.exists() {
            warn!("Required data files missing, database may be outdated");
            return Ok(false); // Don't update if source files are missing
        }

        // Get the most recent modification time of our source files
        let starmap_modified = fs::metadata(&starmap_path).await?.modified()?;
        let labels_modified = fs::metadata(&labels_path).await?.modified()?;
        let latest_file_time = starmap_modified.max(labels_modified);

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
        sqlx::query("DELETE FROM systems").execute(&mut *tx).await?;
        sqlx::query("DELETE FROM constellations").execute(&mut *tx).await?;
        sqlx::query("DELETE FROM regions").execute(&mut *tx).await?;
        tx.commit().await?;

        // Load stellar labels
        let labels_path = Path::new(data_dir).join("stellar_labels.json");
        let labels_content = fs::read_to_string(&labels_path).await?;
        let labels: serde_json::Value = serde_json::from_str(&labels_content)?;
        
        let empty_map = serde_json::Map::new();
        let system_labels = labels.get("systems").and_then(|s| s.as_object()).unwrap_or(&empty_map);
        let region_labels = labels.get("regions").and_then(|r| r.as_object()).unwrap_or(&empty_map);
        let constellation_labels = labels.get("constellations").and_then(|c| c.as_object()).unwrap_or(&empty_map);

        // Load starmap data
        let starmap_path = Path::new(data_dir).join("starmapcache.json");
        let starmap_content = fs::read_to_string(&starmap_path).await?;
        let starmap: serde_json::Value = serde_json::from_str(&starmap_content)?;

        let mut systems_inserted = 0;
        let mut regions_inserted = 0;
        let mut constellations_inserted = 0;

        // Insert regions
        if let Some(regions_data) = starmap.get("regions").and_then(|r| r.as_object()) {
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
        }

        // Insert constellations
        if let Some(constellations_data) = starmap.get("constellations").and_then(|c| c.as_object()) {
            for (id_str, constellation_data) in constellations_data {
                if let Ok(constellation_id) = id_str.parse::<u32>() {
                    if let Ok(constellation) = serde_json::from_value::<Constellation>(constellation_data.clone()) {
                        let default_name = format!("Constellation_{}", constellation_id);
                        let name = constellation_labels.get(&constellation_id.to_string())
                            .and_then(|n| n.as_str())
                            .unwrap_or(&default_name);

                        sqlx::query("INSERT INTO constellations (id, name, region_id) VALUES (?, ?, ?)")
                            .bind(constellation_id)
                            .bind(name)
                            .bind(constellation.region_id)
                            .execute(&self.pool)
                            .await?;
                        
                        constellations_inserted += 1;
                    }
                }
            }
        }

        // Insert systems
        if let Some(systems_data) = starmap.get("solarSystems").and_then(|s| s.as_object()) {
            for (id_str, system_data) in systems_data {
                if let Ok(system_id) = id_str.parse::<u32>() {
                    if let Ok(system) = serde_json::from_value::<SolarSystem>(system_data.clone()) {
                        let default_name = format!("System_{}", system_id);
                        let name = system_labels.get(&system_id.to_string())
                            .and_then(|n| n.as_str())
                            .unwrap_or(&default_name);

                        sqlx::query(
                            "INSERT INTO systems (id, name, center_x, center_y, center_z, region_id, constellation_id, faction_id) 
                             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
                        )
                        .bind(system_id)
                        .bind(name)
                        .bind(system.center[0])
                        .bind(system.center[1])
                        .bind(system.center[2])
                        .bind(system.region_id)
                        .bind(system.constellation_id)
                        .bind(system.faction_id)
                        .execute(&self.pool)
                        .await?;
                        
                        systems_inserted += 1;
                    }
                }
            }
        }

        // Update metadata with current timestamp
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        sqlx::query(
            "INSERT OR REPLACE INTO metadata (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)"
        )
        .bind("last_update")
        .bind(now.to_string())
        .execute(&self.pool)
        .await?;

        info!(
            "Database seeded successfully: {} systems, {} regions, {} constellations",
            systems_inserted, regions_inserted, constellations_inserted
        );

        Ok(())
    }
} 