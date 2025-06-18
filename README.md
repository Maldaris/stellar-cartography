# Stellar Cartography API

A high-performance spatial indexing API for EVE Frontier's stellar map data, capable of handling 100k+ star systems with sub-millisecond query times.

## Architecture

- **Spatial Index**: KD-tree (using `kiddo` crate) for O(log n) nearest neighbor queries
- **Web Framework**: Axum with HTTP/2 support + Tokio for async operations
- **Database**: SQLite for metadata storage with proper migrations
- **Data Pipeline**: Node.js extraction from EVE Frontier game files

## Setup

### Prerequisites

1. **Install Rust**: <https://rustup.rs/>
2. **Node.js**: For data extraction pipeline
3. **EVE Frontier**: Game must be installed with ResFiles available

### Data Extraction

```bash
# Install Node.js dependencies
npm install

# Extract EVE Frontier data (already completed if data/json exists)
npm run extract
```

### Building the API

```bash
# Build the project
cargo build --release

# Run the API server
cargo run --release
```

The API will start on `http://localhost:3000`

## API Endpoints

- `GET /health` - Health check
- `GET /systems/near?name={system_name}&radius={radius}` - Find systems within radius
- `GET /systems/nearest?name={system_name}&k={count}` - Find k-nearest systems
- `GET /systems/autocomplete?q={partial_name}` - Autocomplete system names

## Database Migrations

We use SQLx migrations for database schema management:

```bash
# Create a new migration
sqlx migrate add <migration_name>

# Run migrations (happens automatically on startup)
sqlx migrate run
```

## Performance

- **Startup**: Sub-second with binary cache, ~3-4 minutes for initial database seeding
- **Queries**: Sub-millisecond response times for spatial queries
- **Memory**: ~5MB binary cache, efficient KD-tree structure for 24k+ systems
- **Concurrency**: Fully async, handles thousands of concurrent requests
- **Data Integrity**: SHA-256 fingerprinting ensures cache validity

## Future Enhancements

- [x] Persistent spatial index serialization
- [x] System name resolution from localization files
- [ ] 3D visualization web UI
- [ ] Jump route calculations
- [ ] Regional statistics API
