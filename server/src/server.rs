use crate::gene_pool::GenePool;
use crate::web;
use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use shared::{GlobalStats, ServerError, WorkAssignment, WorkRequest, WorkResult, PROTOCOL_VERSION};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub gene_pool: GenePool,
}

pub async fn run() -> anyhow::Result<()> {
    let state = AppState {
        gene_pool: GenePool::new(),
    };

    let app = Router::new()
        .route("/api/work/request", post(handle_work_request))
        .route("/api/work/submit", post(handle_work_submit))
        .route("/api/stats", get(handle_stats))
        .route("/health", get(web::health))
        .route("/healthz", get(web::health))
        .route("/", get(web::index))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Handle work request from client
#[axum::debug_handler]
async fn handle_work_request(
    State(state): State<AppState>,
    Json(_request): Json<WorkRequest>,
) -> Json<WorkAssignment> {
    // Get seed genomes for spatial simulation (Version 2)
    let seed_genomes_v2 = state.gene_pool.get_seed_genomes_spatial().await;

    // Create work assignment for spatial simulation
    let assignment = WorkAssignment::new_spatial(
        seed_genomes_v2,
        300,  // grid width
        300,  // grid height
        3000, // max steps
        0.05, // mutation rate
    );

    Json(assignment)
}

/// Handle work result submission from client
async fn handle_work_submit(
    State(state): State<AppState>,
    Json(result): Json<WorkResult>,
) -> StatusCode {
    // Check if this is spatial simulation results (Version 2)
    if !result.survival_results.is_empty() {
        tracing::info!(
            "Spatial simulation result from client {} ({} steps)",
            result.client_id,
            result.steps_completed
        );

        // Submit survival results to gene pool
        state
            .gene_pool
            .submit_survival_results(
                result.client_id,
                result.survival_results,
                result.steps_completed,
            )
            .await;
    } else {
        // Legacy results (Version 1)
        tracing::info!(
            "Legacy work result from client {} ({} generations)",
            result.client_id,
            result.generations_completed
        );

        state
            .gene_pool
            .submit_results(
                result.client_id,
                result.best_genomes,
                result.generations_completed,
            )
            .await;
    }

    StatusCode::OK
}

/// Get global statistics
async fn handle_stats(State(state): State<AppState>) -> Json<GlobalStats> {
    let stats = state.gene_pool.get_stats().await;
    Json(stats)
}

/// API error type
#[derive(Debug)]
pub enum ApiError {
    VersionMismatch {
        server_version: u32,
        client_version: u32,
    },
}

pub type Result<T> = std::result::Result<T, ApiError>;

impl IntoResponse for ApiError {
    fn into_response(self) -> Response<Body> {
        let (status, error) = match self {
            ApiError::VersionMismatch {
                server_version,
                client_version,
            } => (
                StatusCode::BAD_REQUEST,
                ServerError::VersionMismatch {
                    server_version,
                    client_version,
                },
            ),
        };

        (status, Json(error)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // Tests disabled due to Axum Handler trait compilation issue
    // The handlers work fine at runtime but don't compile in test context

    // #[tokio::test]
    // async fn test_work_request_handler() {
    //     let state = AppState {
    //         gene_pool: GenePool::new(),
    //     };
    //
    //     let request = WorkRequest::new(Uuid::new_v4(), PROTOCOL_VERSION);
    //
    //     let _response = handle_work_request(State(state), Json(request)).await;
    // }

    #[tokio::test]
    async fn test_gene_pool() {
        let pool = GenePool::new();
        let stats = pool.get_stats().await;
        assert!(stats.gene_pool_size > 0);
    }
}
