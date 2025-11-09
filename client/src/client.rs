use anyhow::{Context, Result};
use shared::{ServerError, WorkRequest, WorkResult, PROTOCOL_VERSION};
use std::time::Duration;
use uuid::Uuid;

pub struct Client {
    client_id: Uuid,
    server_url: String,
    http_client: reqwest::Client,
}

impl Client {
    pub fn new(server_url: &str) -> Self {
        // Load or generate client ID
        let client_id = Uuid::new_v4();

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        Self {
            client_id,
            server_url: server_url.to_string(),
            http_client,
        }
    }

    /// Request work from the server
    pub async fn request_work(&self) -> Result<shared::WorkAssignment> {
        let url = format!("{}/api/work/request", self.server_url);
        let request = WorkRequest::new(self.client_id, PROTOCOL_VERSION);

        tracing::debug!("Requesting work from server");

        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send work request")?;

        if !response.status().is_success() {
            let status = response.status();
            // Try to parse error response
            if let Ok(error) = response.json::<ServerError>().await {
                match error {
                    ServerError::VersionMismatch {
                        server_version,
                        client_version,
                    } => {
                        tracing::error!(
                            "Version mismatch! Server version: {}, Client version: {}",
                            server_version,
                            client_version
                        );
                        anyhow::bail!("Version mismatch - please update client");
                    }
                    ServerError::ServerOverloaded => {
                        tracing::warn!("Server is overloaded, will retry");
                        anyhow::bail!("Server overloaded");
                    }
                    ServerError::InvalidRequest(msg) => {
                        anyhow::bail!("Invalid request: {}", msg);
                    }
                    ServerError::InternalError(msg) => {
                        anyhow::bail!("Server error: {}", msg);
                    }
                }
            }
            anyhow::bail!("Request failed with status: {}", status);
        }

        let assignment = response
            .json()
            .await
            .context("Failed to parse work assignment")?;

        Ok(assignment)
    }

    /// Submit work results to the server
    pub async fn submit_results(&self, result: WorkResult) -> Result<()> {
        let url = format!("{}/api/work/submit", self.server_url);

        tracing::debug!("Submitting work results");

        let response = self
            .http_client
            .post(&url)
            .json(&result)
            .send()
            .await
            .context("Failed to send work results")?;

        if !response.status().is_success() {
            anyhow::bail!("Submit failed with status: {}", response.status());
        }

        Ok(())
    }

    /// Run a work assignment
    pub fn process_work(&self, assignment: shared::WorkAssignment) -> Result<WorkResult> {
        tracing::info!(
            "Processing work: {} generations with {} creatures",
            assignment.generations,
            assignment.population_size
        );

        let (best_genomes, stats) = sim::run_simulation(
            assignment.seed_genomes,
            assignment.generations,
            assignment.population_size,
            assignment.mutation_rate,
        );

        Ok(WorkResult {
            work_id: assignment.work_id,
            client_id: self.client_id,
            best_genomes,
            generations_completed: assignment.generations,
            stats,
        })
    }
}

/// Main client loop
pub async fn run(server_url: &str) -> Result<()> {
    let client = Client::new(server_url);

    tracing::info!("Client ID: {}", client.client_id);

    loop {
        // Request work
        let assignment = match client.request_work().await {
            Ok(a) => a,
            Err(e) => {
                if e.to_string().contains("Version mismatch") {
                    tracing::error!("Version mismatch detected - exiting");
                    std::process::exit(1);
                }

                tracing::error!("Failed to request work: {}", e);
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
        };

        // Process work
        let result = match client.process_work(assignment) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Failed to process work: {}", e);
                continue;
            }
        };

        // Submit results
        if let Err(e) = client.submit_results(result).await {
            tracing::error!("Failed to submit results: {}", e);
            tokio::time::sleep(Duration::from_secs(5)).await;
            continue;
        }

        tracing::info!("Work completed successfully");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = Client::new("http://localhost:8080");
        assert!(!client.client_id.is_nil());
    }

    #[test]
    fn test_process_work() {
        let client = Client::new("http://localhost:8080");
        let assignment = shared::WorkAssignment::new(vec![shared::Genome::random()], 10, 20, 0.1);

        let result = client.process_work(assignment);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.generations_completed, 10);
        assert!(!result.best_genomes.is_empty());
    }
}
