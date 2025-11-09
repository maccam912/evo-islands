use crate::gene_pool::GenePool;
use crate::web;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
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
async fn handle_work_request(
    State(state): State<AppState>,
    Json(request): Json<WorkRequest>,
) -> Result<Json<WorkAssignment>, ApiError> {
    tracing::info!("Work request from client {}", request.client_id);

    // Check protocol version
    if request.protocol_version != PROTOCOL_VERSION {
        return Err(ApiError::VersionMismatch {
            server_version: PROTOCOL_VERSION,
            client_version: request.protocol_version,
        });
    }

    // Register client
    state.gene_pool.register_client(request.client_id).await;

    // Get seed genomes
    let seed_genomes = state.gene_pool.get_seed_genomes(10).await;

    // Create work assignment
    let assignment = WorkAssignment::new(
        seed_genomes,
        100,  // generations
        50,   // population size
        0.05, // mutation rate
    );

    tracing::debug!(
        "Assigned work {} to client {}",
        assignment.work_id,
        request.client_id
    );

    Ok(Json(assignment))
}

/// Handle work result submission from client
async fn handle_work_submit(
    State(state): State<AppState>,
    Json(result): Json<WorkResult>,
) -> Result<StatusCode, ApiError> {
    tracing::info!(
        "Work result from client {} ({} generations)",
        result.client_id,
        result.generations_completed
    );

    // Submit results to gene pool
    state
        .gene_pool
        .submit_results(
            result.client_id,
            result.best_genomes,
            result.generations_completed,
        )
        .await;

    Ok(StatusCode::OK)
}

/// Get global statistics
async fn handle_stats(State(state): State<AppState>) -> Json<GlobalStats> {
    let stats = state.gene_pool.get_stats().await;
    Json(stats)
}

/// API error type
#[derive(Debug)]
enum ApiError {
    VersionMismatch {
        server_version: u32,
        client_version: u32,
    },
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
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

    #[tokio::test]
    async fn test_work_request_handler() {
        let state = AppState {
            gene_pool: GenePool::new(),
        };

        let request = WorkRequest::new(Uuid::new_v4(), PROTOCOL_VERSION);

        let result = handle_work_request(State(state), Json(request)).await;

        assert!(result.is_ok());
        let assignment = result.unwrap().0;
        assert!(!assignment.seed_genomes.is_empty());
    }

    #[tokio::test]
    async fn test_version_mismatch() {
        let state = AppState {
            gene_pool: GenePool::new(),
        };

        let request = WorkRequest::new(Uuid::new_v4(), 999); // Wrong version

        let result = handle_work_request(State(state), Json(request)).await;

        assert!(result.is_err());
    }
}
