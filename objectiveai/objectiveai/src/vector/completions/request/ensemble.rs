use crate::ensemble;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Ensemble {
    Id(String),
    Provided(ensemble::EnsembleBase),
}
