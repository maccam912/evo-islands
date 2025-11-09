use axum::response::Html;

/// Serve the main web UI
pub async fn index() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}
