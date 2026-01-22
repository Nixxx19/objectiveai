use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Upstream {
    OpenRouter,
}

pub fn upstreams(
    _ensemble_llm: &objectiveai::ensemble_llm::EnsembleLlm,
    _request: super::Params,
) -> impl Iterator<Item = Upstream> {
    const ALL_UPSTREAMS: [Upstream; 1] = [Upstream::OpenRouter];
    ALL_UPSTREAMS.iter().copied()
}
