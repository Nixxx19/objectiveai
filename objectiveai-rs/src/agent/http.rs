//! HTTP client functions for Agent endpoints.

use crate::{HttpClient, HttpError};

/// Lists all Agents that have been used.
pub async fn list_agents(
    client: &HttpClient,
) -> Result<super::response::ListAgent, HttpError> {
    client
        .send_unary(reqwest::Method::GET, "agents", None::<String>)
        .await
}

/// Retrieves a specific Agent by its content-addressed ID.
pub async fn get_agent(
    client: &HttpClient,
    agent_id: &str,
) -> Result<super::response::GetAgent, HttpError> {
    client
        .send_unary(
            reqwest::Method::GET,
            &format!("agents/{}", agent_id),
            None::<String>,
        )
        .await
}

/// Retrieves usage statistics for a specific Agent.
pub async fn get_agent_usage(
    client: &HttpClient,
    agent_id: &str,
) -> Result<super::response::UsageAgent, HttpError> {
    client
        .send_unary(
            reqwest::Method::GET,
            &format!("agents/{}/usage", agent_id),
            None::<String>,
        )
        .await
}
