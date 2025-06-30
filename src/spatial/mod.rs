use anyhow::Result;
use kiddo::float::kdtree::KdTree;
use kiddo::SquaredEuclidean;
use rustc_hash::FxHashMap;
use tracing::{info, warn};
use std::path::Path;
use tokio::fs;
use sha2::{Sha256, Digest};

use crate::models::{SolarSystem, Region, Constellation};
use crate::database::Database;

pub type Point3D = [f64; 3];
pub type SystemId = u32;

#[derive(serde::Serialize, serde::Deserialize)]
struct SerializableSpatialData {
    // Data fingerprint for integrity and change detection
    data_fingerprint: String,
    version: String,
    created_at: u64,
    
    // Spatial data
    kdtree: KdTree<f64, usize, 3, 32, u32>,
    systems: FxHashMap<SystemId, SolarSystem>,
    system_names: FxHashMap<String, SystemId>,
    regions: FxHashMap<u32, Region>,
    constellations: FxHashMap<u32, Constellation>,
    localized_names: FxHashMap<u32, String>,
    system_name_list: Vec<(String, SystemId)>,
    system_positions: Vec<(Point3D, SystemId)>,
}

#[derive(Debug)]
pub struct SpatialIndex {
    // KD-tree for fast spatial queries
    // KdTree<A, T, K, B, IDX> where:
    // A = f64 (coordinate type)
    // T = usize (value type - we'll store indices)
    // K = 3 (dimensions)
    // B = 32 (bucket size)
    // IDX = u32 (index type)
    kdtree: KdTree<f64, usize, 3, 32, u32>,
    
    // Fast lookups
    systems: FxHashMap<SystemId, SolarSystem>,
    system_names: FxHashMap<String, SystemId>,
    regions: FxHashMap<u32, Region>,
    constellations: FxHashMap<u32, Constellation>,
    
    // Localization data
    localized_names: FxHashMap<u32, String>,
    
    // Name mappings for autocomplete
    system_name_list: Vec<(String, SystemId)>,
    
    // Store system positions to map back from KdTree indices
    system_positions: Vec<(Point3D, SystemId)>,
}

impl SpatialIndex {
    async fn compute_data_fingerprint(data_dir: &str) -> Result<String> {
        let starmap_path = Path::new(data_dir).join("starmapcache.json");
        let labels_path = Path::new(data_dir).join("stellar_labels.json");
        
        let mut hasher = Sha256::new();
        
        // Hash the starmapcache.json file
        if starmap_path.exists() {
            let starmap_data = fs::read(&starmap_path).await?;
            hasher.update(&starmap_data);
        }
        
        // Hash the stellar_labels.json file
        if labels_path.exists() {
            let labels_data = fs::read(&labels_path).await?;
            hasher.update(&labels_data);
        }
        
        // Include file modification times for additional change detection
        if starmap_path.exists() {
            let starmap_modified = fs::metadata(&starmap_path).await?.modified()?;
            let timestamp = starmap_modified.duration_since(std::time::UNIX_EPOCH)?.as_secs();
            hasher.update(timestamp.to_le_bytes());
        }
        
        if labels_path.exists() {
            let labels_modified = fs::metadata(&labels_path).await?.modified()?;
            let timestamp = labels_modified.duration_since(std::time::UNIX_EPOCH)?.as_secs();
            hasher.update(timestamp.to_le_bytes());
        }
        
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    pub async fn load_from_database(database: &Database, data_dir: &str) -> Result<Self> {
        let mut kdtree = KdTree::new();
        let mut systems = FxHashMap::default();
        let mut system_names = FxHashMap::default();
        let mut regions = FxHashMap::default();
        let mut constellations = FxHashMap::default();
        let mut localized_names = FxHashMap::default();
        let mut system_name_list = Vec::new();
        let mut system_positions = Vec::new();

        // Check if database needs updating and seed if necessary
        if database.needs_update(data_dir).await? {
            info!("Database is empty or outdated, seeding from JSON files...");
            database.seed_from_json(data_dir).await?;
        }

        info!("Loading spatial data from database...");

        // Load systems from database
        let db_systems = database.load_all_systems().await?;
        info!("Loaded {} systems from database", db_systems.len());

        for (system_id, system, name) in db_systems {
            // Add to KD-tree
            kdtree.add(&system.center, system_positions.len());
            system_positions.push((system.center, system_id));
            
            // Store system data
            systems.insert(system_id, system);
            system_names.insert(name.clone(), system_id);
            localized_names.insert(system_id, name.clone());
            system_name_list.push((name, system_id));
        }

        // Load regions from database
        let db_regions = database.load_all_regions().await?;
        info!("Loaded {} regions from database", db_regions.len());

        for (region_id, region_name) in db_regions {
            // Create a minimal region object - we might need to expand this
            let region = Region {
                solar_system_ids: Vec::new(), // We don't store this in DB currently
                neighbours: Vec::new(),       // We don't store this in DB currently
                center: [0.0, 0.0, 0.0],     // We don't store this in DB currently
                constellation_ids: Vec::new(), // We don't store this in DB currently
            };
            regions.insert(region_id, region);
            localized_names.insert(region_id, region_name);
        }

        // Load constellations from database
        let db_constellations = database.load_all_constellations().await?;
        info!("Loaded {} constellations from database", db_constellations.len());

        for (constellation_id, constellation_name, region_id) in db_constellations {
            // Create a minimal constellation object
            let constellation = Constellation {
                solar_system_ids: Vec::new(), // We don't store this in DB currently
                neighbours: Vec::new(),       // We don't store this in DB currently
                region_id,
                center: [0.0, 0.0, 0.0],     // We don't store this in DB currently
            };
            constellations.insert(constellation_id, constellation);
            localized_names.insert(constellation_id, constellation_name);
        }

        // Sort system names for better autocomplete performance
        system_name_list.sort_by(|a, b| a.0.cmp(&b.0));

        info!("Spatial index loaded successfully: {} systems, {} regions, {} constellations", 
              systems.len(), regions.len(), constellations.len());

        Ok(Self {
            kdtree,
            systems,
            system_names,
            regions,
            constellations,
            localized_names,
            system_name_list,
            system_positions,
        })
    }



    pub fn get_system_name(&self, id: SystemId) -> Option<&String> {
        self.localized_names.get(&id)
    }

    pub fn find_systems_within_radius(&self, center: Point3D, radius: f64) -> Vec<(SystemId, f64)> {
        self.kdtree
            .within_unsorted::<SquaredEuclidean>(&center, radius * radius) // kiddo uses squared distance
            .iter()
            .map(|result| {
                let index = result.item;
                let distance_sq = result.distance;
                let (_, system_id) = self.system_positions[index];
                (system_id, distance_sq.sqrt())
            })
            .collect()
    }

    pub fn find_nearest_systems(&self, center: Point3D, k: usize) -> Vec<(SystemId, f64)> {
        self.kdtree
            .nearest_n::<SquaredEuclidean>(&center, k)
            .iter()
            .map(|result| {
                let index = result.item;
                let distance_sq = result.distance;
                let (_, system_id) = self.system_positions[index];
                (system_id, distance_sq.sqrt())
            })
            .collect()
    }

    pub fn find_system_by_name(&self, name: &str) -> Option<SystemId> {
        self.system_names.get(name).copied()
    }

    pub fn get_system(&self, id: SystemId) -> Option<&SolarSystem> {
        self.systems.get(&id)
    }

    #[allow(dead_code)]
    pub fn get_region(&self, id: u32) -> Option<&Region> {
        self.regions.get(&id)
    }

    #[allow(dead_code)]
    pub fn get_constellation(&self, id: u32) -> Option<&Constellation> {
        self.constellations.get(&id)
    }

    pub fn autocomplete_systems(&self, query: &str, limit: usize) -> Vec<(String, SystemId)> {
        let query_lower = query.to_lowercase();
        self.system_name_list
            .iter()
            .filter(|(name, _)| name.to_lowercase().contains(&query_lower))
            .take(limit)
            .cloned()
            .collect()
    }

    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    pub fn get_all_system_ids(&self) -> Vec<SystemId> {
        self.systems.keys().copied().collect()
    }

    pub async fn save_to_binary(&self, file_path: &str, data_dir: &str) -> Result<()> {
        info!("Saving spatial index to binary file: {}", file_path);
        
        let data_fingerprint = Self::compute_data_fingerprint(data_dir).await?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        
        let serializable_data = SerializableSpatialData {
            data_fingerprint,
            version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: now,
            kdtree: self.kdtree.clone(),
            systems: self.systems.clone(),
            system_names: self.system_names.clone(),
            regions: self.regions.clone(),
            constellations: self.constellations.clone(),
            localized_names: self.localized_names.clone(),
            system_name_list: self.system_name_list.clone(),
            system_positions: self.system_positions.clone(),
        };

        let binary_data = bincode::serialize(&serializable_data)?;
        
        // Ensure the directory exists
        if let Some(parent) = Path::new(file_path).parent() {
            fs::create_dir_all(parent).await?;
        }
        
        fs::write(file_path, binary_data).await?;
        info!("Spatial index saved to binary file (size: {} bytes)", 
              fs::metadata(file_path).await?.len());
        
        Ok(())
    }

    pub async fn load_from_binary(file_path: &str, data_dir: &str) -> Result<Self> {
        info!("Loading spatial index from binary file: {}", file_path);
        
        let binary_data = fs::read(file_path).await?;
        let serializable_data: SerializableSpatialData = bincode::deserialize(&binary_data)?;
        
        // Verify data fingerprint
        let current_fingerprint = Self::compute_data_fingerprint(data_dir).await?;
        if serializable_data.data_fingerprint != current_fingerprint {
            return Err(anyhow::anyhow!("Data fingerprint mismatch. Cache is outdated (cached: {}, current: {})", 
                      serializable_data.data_fingerprint, current_fingerprint));
        }
        
        info!("Loaded spatial index from binary file: {} systems, {} regions, {} constellations (version: {}, created: {})",
              serializable_data.systems.len(), 
              serializable_data.regions.len(), 
              serializable_data.constellations.len(),
              serializable_data.version,
              serializable_data.created_at);

        Ok(Self {
            kdtree: serializable_data.kdtree,
            systems: serializable_data.systems,
            system_names: serializable_data.system_names,
            regions: serializable_data.regions,
            constellations: serializable_data.constellations,
            localized_names: serializable_data.localized_names,
            system_name_list: serializable_data.system_name_list,
            system_positions: serializable_data.system_positions,
        })
    }

    pub async fn load_with_cache(database: &Database, data_dir: &str, cache_path: &str) -> Result<Self> {
        info!("Loading spatial index with cache support...");
        
        // Try to load from cache first
        if Path::new(cache_path).exists() {
            match Self::load_from_binary(cache_path, data_dir).await {
                Ok(index) => {
                    info!("Successfully loaded spatial index from cache");
                    return Ok(index);
                }
                Err(e) => {
                    warn!("Failed to load from cache: {}, rebuilding from database", e);
                }
            }
        }

        // Load from database and save to cache
        info!("Building spatial index from database...");
        let index = Self::load_from_database(database, data_dir).await?;
        
        // Save to cache for next time
        if let Err(e) = index.save_to_binary(cache_path, data_dir).await {
            warn!("Failed to save spatial index cache: {}", e);
        } else {
            info!("Spatial index cache saved successfully");
        }
        
        Ok(index)
    }
} 