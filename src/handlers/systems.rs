use axum::{
    extract::{Query, State},
    Json,
};
use tracing::info;

use crate::{
    error::{ApiError, ApiResult},
    middleware::RequestId,
    models::{
        NearbyQuery, NearestQuery, AutocompleteQuery,
        NearbySystemsResponse, NearestSystemsResponse, AutocompleteResponse,
        SystemInfo, SystemSuggestion,
    },
    AppState,
};

pub async fn systems_near(
    Query(params): Query<NearbyQuery>,
    State(state): State<AppState>,
    request_id: Option<RequestId>,
) -> ApiResult<Json<NearbySystemsResponse>> {
    // Log with request ID if available
    if let Some(RequestId(id)) = &request_id {
        info!(request_id = %id, "Finding systems near '{}' within radius {}", params.name, params.radius);
    } else {
        info!("Finding systems near '{}' within radius {}", params.name, params.radius);
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

    // Find nearby systems
    let nearby = state
        .spatial_index
        .find_systems_within_radius(center_system_data.center, params.radius);

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
        .filter_map(|(id, distance)| {
            state.spatial_index.get_system(id).map(|sys| SystemInfo {
                id,
                name: state.spatial_index.get_system_name(id).cloned(),
                center: sys.center,
                region_id: sys.region_id,
                constellation_id: sys.constellation_id,
                faction_id: sys.faction_id,
                distance: Some(distance),
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

pub async fn systems_nearest(
    Query(params): Query<NearestQuery>,
    State(state): State<AppState>,
) -> ApiResult<Json<NearestSystemsResponse>> {
    info!("Finding {} nearest systems to '{}'", params.k, params.name);

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
        .filter_map(|(id, distance)| {
            state.spatial_index.get_system(id).map(|sys| SystemInfo {
                id,
                name: state.spatial_index.get_system_name(id).cloned(),
                center: sys.center,
                region_id: sys.region_id,
                constellation_id: sys.constellation_id,
                faction_id: sys.faction_id,
                distance: Some(distance),
            })
        })
        .collect();

    Ok(Json(NearestSystemsResponse {
        center_system,
        nearest_systems,
        k: params.k,
    }))
}

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