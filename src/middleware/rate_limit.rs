use std::sync::Arc;
use tower_governor::{
    governor::{GovernorConfigBuilder, middleware::NoOpMiddleware},
    GovernorLayer,
    key_extractor::SmartIpKeyExtractor,
};
use std::time::Duration;

/// Rate limiting configuration
pub struct RateLimitConfig {
    /// Requests per second for general endpoints
    pub general_rps: u32,
    /// Requests per second for search/autocomplete endpoints
    pub search_rps: u32,
    /// Burst size (max requests in a burst)
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            general_rps: 100,    // 100 requests per second for general endpoints
            search_rps: 20,      // 20 requests per second for search endpoints
            burst_size: 10,      // Allow bursts of up to 10 requests
        }
    }
}

/// Create a rate limiting layer for general endpoints
pub fn general_rate_limit_layer(config: &RateLimitConfig) -> GovernorLayer<'static, SmartIpKeyExtractor, NoOpMiddleware> {
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(config.general_rps as u64)
            .burst_size(config.burst_size)
            .finish()
            .expect("Failed to create rate limiter config")
    );

    GovernorLayer::from(governor_conf)
}

/// Create a rate limiting layer for search endpoints (more restrictive)
pub fn search_rate_limit_layer(config: &RateLimitConfig) -> GovernorLayer<'static, SmartIpKeyExtractor, NoOpMiddleware> {
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(config.search_rps as u64)
            .burst_size(config.burst_size / 2) // Smaller burst for search
            .finish()
            .expect("Failed to create rate limiter config")
    );

    GovernorLayer::from(governor_conf)
}

/// Create a custom rate limiting layer with specific parameters
pub fn custom_rate_limit_layer(
    requests_per_second: u64,
    burst_size: u32,
    period: Duration,
) -> GovernorLayer<'static, SmartIpKeyExtractor, NoOpMiddleware> {
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(requests_per_second)
            .burst_size(burst_size)
            .period(period)
            .finish()
            .expect("Failed to create rate limiter config")
    );

    GovernorLayer::from(governor_conf)
}

/// Rate limit error response
pub mod error_response {
    use axum::{
        response::{IntoResponse, Response},
        Json,
    };
    use axum::http::StatusCode;
    use serde_json::json;

    pub fn rate_limit_exceeded() -> Response {
        (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({
                "error": "rate_limit_exceeded",
                "message": "Too many requests. Please slow down.",
                "retry_after_seconds": 1
            }))
        ).into_response()
    }
} 