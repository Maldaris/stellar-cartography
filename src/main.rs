use axum::{
    routing::get,
    Router,
    middleware as axum_middleware,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use serde_json;

mod spatial;
mod models;
mod handlers;
mod database;
mod error;
mod middleware;
pub mod coordinates;

use handlers::{health, systems, type_names};
use spatial::SpatialIndex;
use database::Database;

#[derive(OpenApi)]
#[openapi(
    paths(
        // System endpoints
        systems::systems_near,
        systems::systems_nearest,
        systems::systems_autocomplete,
        systems::systems_lookup,
        systems::systems_bulk,
        systems::system_hierarchy,
        systems::complete_system_hierarchy,
        systems::systems_connections_bulk,
        
        // Type names endpoints
        type_names::search_type_names,
        type_names::get_type_name,
        
        // Health endpoint
        health::health_check,
    ),
    components(
        schemas(
            // Response models
            models::NearbySystemsResponse,
            models::NearestSystemsResponse,
            models::AutocompleteResponse,
            models::BulkSystemsResponse,
            models::SystemInfo,
            models::SystemSuggestion,
            models::SystemMapData,
            models::SystemHierarchy,
            models::RegionInfo,
            models::ConstellationInfo,
            models::CompleteSystemHierarchy,
            models::ConstellationWithSystems,
            models::RegionWithConstellations,
            models::GateConnection,
            models::SystemConnections,
            models::BulkConnectionsResponse,

            // Type names models
            models::TypeName,
            models::TypeNameResponse,
            
            // Query models
            models::NearbyQuery,
            models::NearestQuery,
            models::AutocompleteQuery,
            models::SystemLookupQuery,
            models::BulkSystemsQuery,
            models::SystemHierarchyQuery,
            models::BulkConnectionsQuery,
            models::TypeNameQuery,
            
            // Health response
            health::HealthResponse,
        )
    ),
    tags(
        (name = "systems", description = "Solar system spatial queries and search"),
        (name = "type-names", description = "EVE type ID to name lookup functionality"),
        (name = "health", description = "Service health monitoring")
    ),
    info(
        title = "Stellar Cartography API",
        version = "0.1.0",
        description = "A high-performance spatial search engine for EVE Frontier solar systems, providing nearest neighbor queries, radius-based searches, and autocomplete functionality.",
        contact(
            name = "VULTUR Project",
            url = "https://github.com/Maldaris/stellar-cartography"
        )
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting stellar cartography API server");

    // Generate OpenAPI JSON file for reference
    let openapi_json = serde_json::to_string_pretty(&ApiDoc::openapi()).unwrap();
    std::fs::write("openapi.json", &openapi_json)?;
    info!("OpenAPI specification written to openapi.json");

    // Initialize database
    let db = Database::new("data/stellar.db").await?;
    
    // Load spatial index with binary cache support
    info!("Loading spatial index with cache support...");
    let data_dir = std::env::var("EVE_FRONTIER_DATA_DIR").unwrap_or_else(|_| "../eve-frontier-tools/data/extracted".to_string());
    let spatial_index = Arc::new(SpatialIndex::load_with_cache(&db, &data_dir, "data/cache/spatial_index.bin").await?);
    info!("Loaded {} systems into spatial index", spatial_index.system_count());

    // Get path prefix from environment variable
    let path_prefix = std::env::var("PATH_PREFIX").unwrap_or_default();
    info!("Using path prefix: '{}'", path_prefix);
    
    // Build our application with routes
    let mut app = Router::new()
        // Health check - no rate limit
        .route(&format!("{}/health", path_prefix), get(health::health_check))
        // System routes
        .route(&format!("{}/systems/near", path_prefix), get(systems::systems_near))
        .route(&format!("{}/systems/nearest", path_prefix), get(systems::systems_nearest))
        // Autocomplete
        .route(&format!("{}/systems/autocomplete", path_prefix), get(systems::systems_autocomplete))
        .route(&format!("{}/systems/lookup", path_prefix), get(systems::systems_lookup))
        .route(&format!("{}/systems/bulk", path_prefix), get(systems::systems_bulk))
        .route(&format!("{}/systems/hierarchy", path_prefix), get(systems::system_hierarchy))
        .route(&format!("{}/systems/hierarchy/complete", path_prefix), get(systems::complete_system_hierarchy))
        .route(&format!("{}/systems/connections/bulk", path_prefix), get(systems::systems_connections_bulk))
        // Type names routes
        .route(&format!("{}/type-names/search", path_prefix), get(type_names::search_type_names))
        .route(&format!("{}/type-names/:type_id", path_prefix), get(type_names::get_type_name))
        .with_state(AppState {
            database: db,
            spatial_index,
        });

    // Apply individual middleware layers
    app = app.layer(axum_middleware::from_fn(middleware::request_id::request_id_middleware));
    
    // Apply security headers individually
    for header_layer in middleware::security::security_headers() {
        app = app.layer(header_layer);
    }
    
    app = app.layer(middleware::security::timeout_layer());
    app = app.layer(middleware::security::body_limit_layer());
    app = app.layer(middleware::security::cors_layer());

    // Add Swagger UI routes
    app = app.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

    // Run it with hyper on all interfaces:3000
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("API server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
struct AppState {
    #[allow(dead_code)]
    database: Database,
    spatial_index: Arc<SpatialIndex>,
} 