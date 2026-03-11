use crate::agent;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[schemars(rename = "vector.completions.cache.CacheVoteRequest")]
pub enum CacheVoteRequest<'a> {
    Ref(CacheVoteRequestRef<'a>),
    Owned(CacheVoteRequestOwned),
}

impl<'de> serde::de::Deserialize<'de> for CacheVoteRequest<'static> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let owned = CacheVoteRequestOwned::deserialize(deserializer)?;
        Ok(CacheVoteRequest::Owned(owned))
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[schemars(rename = "vector.completions.cache.CacheVoteRequestRef")]
pub struct CacheVoteRequestRef<'a> {
    pub agent: &'a agent::completions::request::Agent,
    pub agents: Option<&'a [agent::completions::request::Agent]>,
    pub messages: &'a [agent::completions::message::Message],
    pub responses: &'a [agent::completions::message::RichContent],
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "vector.completions.cache.CacheVoteRequestOwned")]
pub struct CacheVoteRequestOwned {
    pub agent: agent::completions::request::Agent,
    pub agents: Option<Vec<agent::completions::request::Agent>>,
    pub messages: Vec<agent::completions::message::Message>,
    pub responses: Vec<agent::completions::message::RichContent>,
}
