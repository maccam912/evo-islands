use axum::{http::StatusCode, response::Html, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
}

/// Serve the main web UI
pub async fn index() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

/// Health check endpoint for Kubernetes readiness/liveness probes
pub async fn health() -> (StatusCode, Json<HealthResponse>) {
    (
        StatusCode::OK,
        Json(HealthResponse { status: "healthy" }),
    )
}
