use axum::{
    extract::{Query, State},
    Json,
};
use tracing::info;

use crate::{
    error::{ApiError, ApiResult},
    models::{TypeNameQuery, TypeNameResponse},
    AppState,
};

/// Search type names by query string
#[utoipa::path(
    get,
    path = "/type-names/search",
    params(TypeNameQuery),
    responses(
        (status = 200, description = "Type names matching query", body = TypeNameResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "type-names"
)]
pub async fn search_type_names(
    State(state): State<AppState>,
    Query(params): Query<TypeNameQuery>,
) -> ApiResult<Json<TypeNameResponse>> {
    info!("Searching type names with query: {}", params.q);

    if params.q.trim().is_empty() {
        return Err(ApiError::InvalidInput("Query parameter 'q' cannot be empty".to_string()));
    }

    let limit = params.limit.unwrap_or(50).min(100);

    match state.database.search_type_names(&params.q, limit).await {
        Ok(response) => {
            info!("Found {} type names", response.type_names.len());
            Ok(Json(response))
        }
        Err(e) => {
            let error_msg = format!("Failed to search type names: {}", e);
            tracing::error!("{}", error_msg);
            Err(ApiError::InternalError(e))
        }
    }
}

/// Get a specific type name by ID
#[utoipa::path(
    get,
    path = "/type-names/{type_id}",
    params(
        ("type_id" = u32, Path, description = "Type ID to look up")
    ),
    responses(
        (status = 200, description = "Type name", body = String),
        (status = 404, description = "Type not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "type-names"
)]
pub async fn get_type_name(
    State(state): State<AppState>,
    axum::extract::Path(type_id): axum::extract::Path<u32>,
) -> ApiResult<Json<Option<String>>> {
    info!("Looking up type name for ID: {}", type_id);

    match state.database.get_type_name(type_id).await {
        Ok(name) => Ok(Json(name)),
        Err(e) => {
            let error_msg = format!("Failed to get type name: {}", e);
            tracing::error!("{}", error_msg);
            Err(ApiError::InternalError(e))
        }
    }
} 