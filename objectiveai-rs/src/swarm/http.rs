//! HTTP client functions for Swarm endpoints.

use crate::{HttpClient, HttpError};

/// Lists all Swarms that have been used.
pub async fn list_swarms(
    client: &HttpClient,
) -> Result<super::response::ListSwarm, HttpError> {
    client
        .send_unary(reqwest::Method::GET, "swarms", None::<String>)
        .await
}

/// Retrieves a specific Swarm by its content-addressed ID.
pub async fn get_swarm(
    client: &HttpClient,
    swarm_id: &str,
) -> Result<super::response::GetSwarm, HttpError> {
    client
        .send_unary(
            reqwest::Method::GET,
            &format!("swarms/{}", swarm_id),
            None::<String>,
        )
        .await
}

/// Retrieves usage statistics for a specific Swarm.
pub async fn get_swarm_usage(
    client: &HttpClient,
    swarm_id: &str,
) -> Result<super::response::UsageSwarm, HttpError> {
    client
        .send_unary(
            reqwest::Method::GET,
            &format!("swarms/{}/usage", swarm_id),
            None::<String>,
        )
        .await
}
