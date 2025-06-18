use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, IntoParams};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SolarSystem {
    pub center: [f64; 3],
    #[serde(rename = "regionID")]
    pub region_id: Option<u32>,
    #[serde(rename = "planetItemIDs")]
    pub planet_item_ids: Vec<u32>,
    #[serde(rename = "planetCountByType")]
    pub planet_count_by_type: HashMap<String, u32>,
    pub neighbours: Vec<u32>,
    #[serde(rename = "factionID")]
    pub faction_id: Option<u32>,
    #[serde(rename = "constellationID")]
    pub constellation_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Constellation {
    #[serde(rename = "solarSystemIDs")]
    pub solar_system_ids: Vec<u32>,
    pub neighbours: Vec<u32>,
    #[serde(rename = "regionID")]
    pub region_id: u32,
    pub center: [f64; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Region {
    #[serde(rename = "solarSystemIDs")]
    pub solar_system_ids: Vec<u32>,
    pub neighbours: Vec<u32>,
    pub center: [f64; 3],
    #[serde(rename = "constellationIDs")]
    pub constellation_ids: Vec<u32>,
}

// API response types
#[derive(Debug, Serialize, ToSchema)]
pub struct SystemInfo {
    pub id: u32,
    pub name: Option<String>,
    pub center: [f64; 3],
    pub region_id: Option<u32>,
    pub constellation_id: Option<u32>,
    pub faction_id: Option<u32>,
    pub distance: Option<f64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NearbySystemsResponse {
    pub center_system: SystemInfo,
    pub nearby_systems: Vec<SystemInfo>,
    pub radius: f64,
    pub total_found: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NearestSystemsResponse {
    pub center_system: SystemInfo,
    pub nearest_systems: Vec<SystemInfo>,
    pub k: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AutocompleteResponse {
    pub suggestions: Vec<SystemSuggestion>,
    pub query: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SystemSuggestion {
    pub id: u32,
    pub name: String,
    pub region_name: Option<String>,
    pub constellation_name: Option<String>,
}

// Query parameters
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct NearbyQuery {
    /// System name to search around
    pub name: String,
    /// Search radius in light years
    pub radius: f64,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct NearestQuery {
    /// System name to search around
    pub name: String,
    /// Number of nearest systems to return
    pub k: usize,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct AutocompleteQuery {
    /// Search query for system names
    pub q: String,
    /// Maximum number of suggestions (max 50)
    pub limit: Option<usize>,
} 