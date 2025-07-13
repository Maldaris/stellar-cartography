use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, IntoParams};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SolarSystem {
    pub id: u32,
    pub name: String,
    pub center: [f64; 3],
    #[serde(rename = "regionId")]
    pub region_id: Option<u32>,
    #[serde(rename = "constellationId")]
    pub constellation_id: Option<u32>,
    pub security: SecurityInfo,
    pub celestials: CelestialInfo,
    pub navigation: NavigationInfo,
    pub metadata: SystemMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SecurityInfo {
    pub class: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CelestialInfo {
    #[serde(rename = "starId")]
    pub star_id: Option<u32>,
    #[serde(rename = "planetIds")]
    pub planet_ids: Vec<u32>,
    #[serde(rename = "planetCountByType")]
    pub planet_count_by_type: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NavigationInfo {
    pub neighbours: Vec<u32>,
    pub stargates: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SystemMetadata {
    #[serde(rename = "factionId")]
    pub faction_id: Option<u32>,
    pub sovereignty: Option<String>,
    #[serde(rename = "disallowedAnchorCategories")]
    pub disallowed_anchor_categories: Vec<String>,
    #[serde(rename = "disallowedAnchorGroups")]
    pub disallowed_anchor_groups: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Constellation {
    pub id: u32,
    pub name: String,
    #[serde(rename = "regionId")]
    pub region_id: u32,
    #[serde(rename = "solarSystemIds")]
    pub solar_system_ids: Vec<u32>,
    pub metadata: ConstellationMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConstellationMetadata {
    #[serde(rename = "factionId")]
    pub faction_id: Option<u32>,
    pub sovereignty: Option<String>,
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
    /// Coordinates in meters from galactic center [x, y, z]
    pub center: [f64; 3],
    pub region_id: Option<u32>,
    pub constellation_id: Option<u32>,
    pub faction_id: Option<u32>,
    /// Distance from query center in light-years
    pub distance: Option<f64>,
}

// New detailed models for hierarchical data
#[derive(Debug, Serialize, ToSchema)]
pub struct RegionInfo {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConstellationInfo {
    pub id: u32,
    pub name: String,
    pub region_id: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SystemHierarchy {
    pub system: SystemInfo,
    pub constellation: Option<ConstellationInfo>,
    pub region: Option<RegionInfo>,
}

// Expanded hierarchy models
#[derive(Debug, Serialize, ToSchema)]
pub struct ConstellationWithSystems {
    pub id: u32,
    pub name: String,
    pub region_id: u32,
    pub systems: Vec<SystemInfo>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RegionWithConstellations {
    pub id: u32,
    pub name: String,
    pub constellations: Vec<ConstellationWithSystems>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CompleteSystemHierarchy {
    pub target_system: SystemInfo,
    pub target_constellation: Option<ConstellationWithSystems>,
    pub target_region: Option<RegionWithConstellations>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GateConnection {
    pub id: u32,
    pub from_system_id: u32,
    pub to_system_id: u32,
    pub connection_type: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SystemConnections {
    pub system_id: u32,
    pub connections: Vec<GateConnection>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BulkConnectionsResponse {
    pub connections: Vec<GateConnection>,
    pub total_count: usize,
    pub offset: usize,
    pub limit: usize,
}

// Simplified system data for bulk map requests
#[derive(Debug, Serialize, ToSchema)]
pub struct SystemMapData {
    pub id: u32,
    pub name: String,
    pub center: [f64; 3],
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BulkSystemsResponse {
    pub systems: Vec<SystemMapData>,
    pub total_count: usize,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NearbySystemsResponse {
    pub center_system: SystemInfo,
    pub nearby_systems: Vec<SystemInfo>,
    /// Search radius in light-years
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

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct SystemLookupQuery {
    /// System ID to look up
    pub id: u32,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct BulkSystemsQuery {
    /// Maximum number of systems to return (default: 1000, max: 5000)
    pub limit: Option<usize>,
    /// Offset for pagination (default: 0)
    pub offset: Option<usize>,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct SystemHierarchyQuery {
    /// System ID to get hierarchy for
    pub id: u32,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct BulkConnectionsQuery {
    /// Maximum number of connections to return (default: 1000, max: 10000)
    pub limit: Option<usize>,
    /// Offset for pagination (default: 0)
    pub offset: Option<usize>,
    /// Connection type filter (optional): stargate, jump_bridge, wormhole
    pub connection_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TypeName {
    pub type_id: u32,
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct TypeNameQuery {
    /// Search query for type names
    pub q: String,
    /// Maximum number of results (default: 50, max: 100)
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TypeNameResponse {
    pub type_names: Vec<TypeName>,
    pub query: String,
    pub total_found: usize,
} 