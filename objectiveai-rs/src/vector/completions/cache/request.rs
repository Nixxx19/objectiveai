use crate::agent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct CacheVoteRequestRef<'a> {
    pub model: &'a agent::completions::request::Model,
    pub models: Option<&'a [agent::completions::request::Model]>,
    pub messages: &'a [agent::completions::request::Message],
    pub tools: Option<&'a [agent::completions::request::Tool]>,
    pub responses: &'a [agent::completions::request::RichContent],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheVoteRequestOwned {
    pub model: agent::completions::request::Model,
    pub models: Option<Vec<agent::completions::request::Model>>,
    pub messages: Vec<agent::completions::request::Message>,
    pub tools: Option<Vec<agent::completions::request::Tool>>,
    pub responses: Vec<agent::completions::request::RichContent>,
}
