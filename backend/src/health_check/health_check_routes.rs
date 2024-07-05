use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;

pub fn health_check_routes() -> Router<()> {
    Router::new().route("/health", get(health))
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, "Server up and running")
}
