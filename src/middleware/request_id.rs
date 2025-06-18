use axum::{
    body::Body,
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;
use tracing::{info_span, Instrument};

/// Header name for request ID
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Middleware that adds a unique request ID to each request
pub async fn request_id_middleware(
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check if request already has an ID (from client or proxy)
    let request_id = req
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Add request ID to headers if not present
    req.headers_mut().insert(
        REQUEST_ID_HEADER,
        HeaderValue::from_str(&request_id).unwrap(),
    );

    // Create a tracing span with the request ID
    let span = info_span!(
        "request",
        request_id = %request_id,
        method = %req.method(),
        path = %req.uri().path(),
    );

    // Process the request with the span
    let mut response = next.run(req).instrument(span).await;

    // Add request ID to response headers
    response.headers_mut().insert(
        REQUEST_ID_HEADER,
        HeaderValue::from_str(&request_id).unwrap(),
    );

    Ok(response)
}

/// Extension to extract request ID from request
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

impl RequestId {
    /// Extract request ID from headers
    pub fn from_headers(headers: &axum::http::HeaderMap) -> Option<Self> {
        headers
            .get(REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(|s| Self(s.to_string()))
    }
}

/// Extractor for request ID
#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for RequestId
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        RequestId::from_headers(&parts.headers)
            .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Missing request ID"))
    }
} 