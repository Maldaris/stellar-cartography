use axum::{
    routing::get,
    Router,
    middleware as axum_middleware,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber;

mod spatial;
mod models;
mod handlers;
mod database;
mod error;
mod middleware;

use handlers::{health, systems};
use spatial::SpatialIndex;
use database::Database;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting stellar cartography API server");

    // Initialize database
    let db = Database::new("data/stellar.db").await?;
    
    // Load spatial index with binary cache support
    info!("Loading spatial index with cache support...");
    let spatial_index = Arc::new(SpatialIndex::load_with_cache(&db, "data/json", "data/cache/spatial_index.bin").await?);
    info!("Loaded {} systems into spatial index", spatial_index.system_count());

    // Build our application with routes
    let mut app = Router::new()
        // Health check - no rate limit
        .route("/health", get(health::health_check))
        // System routes
        .route("/systems/near", get(systems::systems_near))
        .route("/systems/nearest", get(systems::systems_nearest))
        // Autocomplete
        .route("/systems/autocomplete", get(systems::systems_autocomplete))
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

    // Run it with hyper on localhost:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("API server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
struct AppState {
    database: Database,
    spatial_index: Arc<SpatialIndex>,
} 