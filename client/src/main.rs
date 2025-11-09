mod client;
mod tui;

use anyhow::Result;
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const DEFAULT_SERVER_URL: &str = "https://evo-islands.rackspace.koski.co";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "client=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Get server URL from environment or use default
    let server_url = env::var("SERVER_URL").unwrap_or_else(|_| DEFAULT_SERVER_URL.to_string());

    tracing::info!("Starting EvoIslands client");
    tracing::info!("Server URL: {}", server_url);

    // Run the client
    client::run(&server_url).await
}
