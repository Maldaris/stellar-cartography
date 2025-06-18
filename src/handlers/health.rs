use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service health status", body = HealthResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "health"
)]
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        service: "stellar-cartography".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
} 