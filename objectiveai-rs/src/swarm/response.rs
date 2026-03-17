//! Response types for Swarm API endpoints.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Response containing a list of Swarms.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "swarm.ListSwarm")]
pub struct ListSwarm {
    /// The list of Swarm summaries.
    pub data: Vec<ListSwarmItem>,
}

/// Summary information for a listed Swarm.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "swarm.ListSwarmItem")]
pub struct ListSwarmItem {
    /// The unique content-addressed ID of the Swarm.
    pub id: String,
}

/// Response containing a single Swarm with creation timestamp.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "swarm.GetSwarm")]
pub struct GetSwarm {
    /// Unix timestamp when this Swarm was first used.
    pub created: u64,
    /// The Swarm definition.
    #[serde(flatten)]
    #[schemars(schema_with = "crate::flatten_schema::<super::Swarm>")]
    pub inner: super::Swarm,
}

/// Usage statistics for an Swarm.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "swarm.UsageSwarm")]
pub struct UsageSwarm {
    /// Total number of requests made with this Swarm.
    pub requests: u64,
    /// Total completion tokens generated across all agents.
    pub completion_tokens: u64,
    /// Total prompt tokens processed across all agents.
    pub prompt_tokens: u64,
    /// Total cost incurred.
    #[schemars(with = "f64")]
    pub total_cost: rust_decimal::Decimal,
}
