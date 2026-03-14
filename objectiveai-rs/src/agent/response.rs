//! Response types for Agent API endpoints.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Response containing a list of Agents.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "agent.ListAgent")]
pub struct ListAgent {
    /// The list of Agent summaries.
    pub data: Vec<ListAgentItem>,
}

/// Summary information for a listed Agent.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "agent.ListAgentItem")]
pub struct ListAgentItem {
    /// The unique content-addressed ID of the Agent.
    pub id: String,
}

/// Response containing a single Agent with creation timestamp.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "agent.GetAgent")]
pub struct GetAgent {
    /// Unix timestamp when this Agent was first used.
    pub created: u64,
    /// The Agent definition.
    #[serde(flatten)]
    #[schemars(schema_with = "crate::flatten_schema::<super::Agent>")]
    pub inner: super::Agent,
}

/// Usage statistics for an Agent.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "agent.UsageAgent")]
pub struct UsageAgent {
    /// Total number of requests made with this Agent.
    pub requests: u64,
    /// Total completion tokens generated.
    pub completion_tokens: u64,
    /// Total prompt tokens processed.
    pub prompt_tokens: u64,
    /// Total cost incurred.
    #[schemars(with = "f64")]
    pub total_cost: rust_decimal::Decimal,
}
