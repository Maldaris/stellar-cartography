use axum::http::{header, HeaderValue};
use std::time::Duration;
use tower_http::{
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    set_header::SetResponseHeaderLayer,
    timeout::TimeoutLayer,
    sensitive_headers::SetSensitiveHeadersLayer,
};

/// Create individual security header layers
pub fn security_headers() -> Vec<SetResponseHeaderLayer<HeaderValue>> {
    vec![
        // X-Content-Type-Options: nosniff
        SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ),
        // X-Frame-Options: DENY
        SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ),
        // X-XSS-Protection: 1; mode=block
        SetResponseHeaderLayer::if_not_present(
            header::X_XSS_PROTECTION,
            HeaderValue::from_static("1; mode=block"),
        ),
        // Strict-Transport-Security: max-age=31536000; includeSubDomains
        SetResponseHeaderLayer::if_not_present(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ),
        // Referrer-Policy: no-referrer
        SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            HeaderValue::from_static("no-referrer"),
        ),
        // Content-Security-Policy
        SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                "default-src 'self'; \
                 script-src 'self'; \
                 style-src 'self' 'unsafe-inline'; \
                 img-src 'self' data: https:; \
                 font-src 'self'; \
                 connect-src 'self'; \
                 frame-ancestors 'none'; \
                 base-uri 'self'; \
                 form-action 'self'"
            ),
        ),
        // Permissions-Policy
        SetResponseHeaderLayer::if_not_present(
            header::HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static(
                "accelerometer=(), \
                 camera=(), \
                 geolocation=(), \
                 gyroscope=(), \
                 magnetometer=(), \
                 microphone=(), \
                 payment=(), \
                 usb=()"
            ),
        ),
    ]
}

/// Create request body limit layer
pub fn body_limit_layer() -> RequestBodyLimitLayer {
    RequestBodyLimitLayer::new(1_048_576) // 1MB
}

/// Create timeout layer
pub fn timeout_layer() -> TimeoutLayer {
    TimeoutLayer::new(Duration::from_secs(30))
}

/// Create CORS layer with secure defaults
pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
        ])
        .max_age(Duration::from_secs(3600))
}

/// Additional security middleware for sensitive headers
#[allow(dead_code)]
pub fn sensitive_headers_layer() -> SetSensitiveHeadersLayer {
    SetSensitiveHeadersLayer::new([
        header::AUTHORIZATION,
        header::COOKIE,
        header::SET_COOKIE,
    ])
} 