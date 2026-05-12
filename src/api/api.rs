use axum::Router;

/// Empty API router - webhook handled in router module
pub fn router() -> Router {
    Router::new()
}
