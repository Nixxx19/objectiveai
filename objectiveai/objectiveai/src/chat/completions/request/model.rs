use crate::ensemble_llm;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Model {
    Id(String),
    Provided(ensemble_llm::EnsembleLlmBase),
}
