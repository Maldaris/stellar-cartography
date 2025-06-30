use axum::{
    extract::{Query, State},
    Json,
};
use tracing::info;

use crate::{
    error::{ApiError, ApiResult},
    middleware::RequestId,
    models::{
        NearbyQuery, NearestQuery, AutocompleteQuery, BulkSystemsQuery,
        NearbySystemsResponse, NearestSystemsResponse, AutocompleteResponse, BulkSystemsResponse,
        SystemInfo, SystemSuggestion, SystemMapData,
    },
    coordinates::Distance,
    AppState,
};

#[utoipa::path(
    get,
    path = "/systems/near",
    params(
        NearbyQuery
    ),
    responses(
        (status = 200, description = "Systems near the specified system (distances in light-years)", body = NearbySystemsResponse),
        (status = 404, description = "System not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "systems"
)]
pub async fn systems_near(
    Query(params): Query<NearbyQuery>,
    State(state): State<AppState>,
    request_id: Option<RequestId>,
) -> ApiResult<Json<NearbySystemsResponse>> {
    // Log with request ID if available
    if let Some(RequestId(id)) = &request_id {
        info!(request_id = %id, "Finding systems near '{}' within radius {:.2} ly", params.name, params.radius);
    } else {
        info!("Finding systems near '{}' within radius {:.2} ly", params.name, params.radius);
    }

    // Find the center system by name
    let center_system_id = state
        .spatial_index
        .find_system_by_name(&params.name)
        .ok_or_else(|| ApiError::SystemNotFound(params.name.clone()))?;

    let center_system_data = state
        .spatial_index
        .get_system(center_system_id)
        .ok_or_else(|| ApiError::InternalError(
            anyhow::anyhow!("System {} exists in name index but not in data", center_system_id)
        ))?;

    // Find nearby systems - convert radius from light-years to meters for spatial search
    let radius_meters = Distance::from_light_years(params.radius).to_meters();
    let nearby = state
        .spatial_index
        .find_systems_within_radius(center_system_data.center, radius_meters);

    let center_system = SystemInfo {
        id: center_system_id,
        name: Some(params.name.clone()),
        center: center_system_data.center,
        region_id: center_system_data.region_id,
        constellation_id: center_system_data.constellation_id,
        faction_id: center_system_data.faction_id,
        distance: Some(0.0),
    };

    let nearby_systems: Vec<SystemInfo> = nearby
        .into_iter()
        .filter(|(id, _)| *id != center_system_id) // Exclude the center system itself
        .filter_map(|(id, distance_meters)| {
            state.spatial_index.get_system(id).map(|sys| {
                // Convert distance from meters to light-years for the response
                let distance_ly = Distance::from_meters(distance_meters).to_ly();
                SystemInfo {
                    id,
                    name: state.spatial_index.get_system_name(id).cloned(),
                    center: sys.center,
                    region_id: sys.region_id,
                    constellation_id: sys.constellation_id,
                    faction_id: sys.faction_id,
                    distance: Some(distance_ly),
                }
            })
        })
        .collect();

    let total_found = nearby_systems.len();

    Ok(Json(NearbySystemsResponse {
        center_system,
        nearby_systems,
        radius: params.radius,
        total_found,
    }))
}

#[utoipa::path(
    get,
    path = "/systems/nearest",
    params(
        NearestQuery
    ),
    responses(
        (status = 200, description = "Nearest systems to the specified system (distances in light-years)", body = NearestSystemsResponse),
        (status = 404, description = "System not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "systems"
)]
pub async fn systems_nearest(
    Query(params): Query<NearestQuery>,
    State(state): State<AppState>,
) -> ApiResult<Json<NearestSystemsResponse>> {
    info!("Finding {} nearest systems to '{}' (distances in ly)", params.k, params.name);

    // Find the center system by name
    let center_system_id = state
        .spatial_index
        .find_system_by_name(&params.name)
        .ok_or_else(|| ApiError::SystemNotFound(params.name.clone()))?;

    let center_system_data = state
        .spatial_index
        .get_system(center_system_id)
        .ok_or_else(|| ApiError::InternalError(
            anyhow::anyhow!("System {} exists in name index but not in data", center_system_id)
        ))?;

    // Find nearest systems (k+1 to account for the center system itself)
    let nearest = state
        .spatial_index
        .find_nearest_systems(center_system_data.center, params.k + 1);

    let center_system = SystemInfo {
        id: center_system_id,
        name: Some(params.name.clone()),
        center: center_system_data.center,
        region_id: center_system_data.region_id,
        constellation_id: center_system_data.constellation_id,
        faction_id: center_system_data.faction_id,
        distance: Some(0.0),
    };

    let nearest_systems: Vec<SystemInfo> = nearest
        .into_iter()
        .filter(|(id, _)| *id != center_system_id) // Exclude the center system itself
        .take(params.k) // Take only k systems
        .filter_map(|(id, distance_meters)| {
            state.spatial_index.get_system(id).map(|sys| {
                // Convert distance from meters to light-years for the response
                let distance_ly = Distance::from_meters(distance_meters).to_ly();
                SystemInfo {
                    id,
                    name: state.spatial_index.get_system_name(id).cloned(),
                    center: sys.center,
                    region_id: sys.region_id,
                    constellation_id: sys.constellation_id,
                    faction_id: sys.faction_id,
                    distance: Some(distance_ly),
                }
            })
        })
        .collect();

    Ok(Json(NearestSystemsResponse {
        center_system,
        nearest_systems,
        k: params.k,
    }))
}

#[utoipa::path(
    get,
    path = "/systems/autocomplete",
    params(
        AutocompleteQuery
    ),
    responses(
        (status = 200, description = "System name suggestions", body = AutocompleteResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "systems"
)]
pub async fn systems_autocomplete(
    Query(params): Query<AutocompleteQuery>,
    State(state): State<AppState>,
) -> ApiResult<Json<AutocompleteResponse>> {
    let limit = params.limit.unwrap_or(10).min(50); // Cap at 50 results
    
    info!("Autocomplete search for '{}' (limit: {})", params.q, limit);

    let suggestions: Vec<SystemSuggestion> = state
        .spatial_index
        .autocomplete_systems(&params.q, limit)
        .into_iter()
        .map(|(name, id)| SystemSuggestion {
            id,
            name,
            region_name: None,     // TODO: Lookup region names
            constellation_name: None, // TODO: Lookup constellation names
        })
        .collect();

    Ok(Json(AutocompleteResponse {
        suggestions,
        query: params.q,
    }))
}

#[utoipa::path(
    get,
    path = "/systems/bulk",
    params(
        BulkSystemsQuery
    ),
    responses(
        (status = 200, description = "Bulk system data for map visualization", body = BulkSystemsResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "systems"
)]
pub async fn systems_bulk(
    Query(params): Query<BulkSystemsQuery>,
    State(state): State<AppState>,
    request_id: Option<RequestId>,
) -> ApiResult<Json<BulkSystemsResponse>> {
    let limit = params.limit.unwrap_or(1000).min(5000); // Cap at 5000 systems
    let offset = params.offset.unwrap_or(0);

    // Log with request ID if available
    if let Some(RequestId(id)) = &request_id {
        info!(request_id = %id, "Bulk systems request: limit={}, offset={}", limit, offset);
    } else {
        info!("Bulk systems request: limit={}, offset={}", limit, offset);
    }

    // Get all system IDs and apply pagination
    let all_system_ids: Vec<u32> = state.spatial_index.get_all_system_ids();
    let total_count = all_system_ids.len();
    
    let paginated_ids: Vec<u32> = all_system_ids
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    // Convert to SystemMapData
    let systems: Vec<SystemMapData> = paginated_ids
        .into_iter()
        .filter_map(|id| {
            let system_data = state.spatial_index.get_system(id)?;
            let name = state.spatial_index.get_system_name(id)?.clone();
            
            Some(SystemMapData {
                id,
                name,
                center: system_data.center,
            })
        })
        .collect();

    Ok(Json(BulkSystemsResponse {
        systems,
        total_count,
        offset,
        limit,
    }))
} 