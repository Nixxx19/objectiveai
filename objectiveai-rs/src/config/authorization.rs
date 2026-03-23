use serde::{Serialize, Deserialize};
use indexmap::IndexMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthorizationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objectiveai: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openrouter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp: Option<IndexMap<String, String>>,
}

impl AuthorizationConfig {
    pub fn is_empty(&self) -> bool {
        self.objectiveai.is_none() &&
            self.openrouter.is_none() &&
            self.github.is_none() &&
            self.mcp.as_ref().is_none_or(|mcp| mcp.is_empty())
    }

    pub fn is_none(this: &Option<Self>) -> bool {
        this.as_ref().is_none_or(|cfg| cfg.is_empty())
    }

    pub fn get_objectiveai(&self) -> Option<&str> {
        self.objectiveai.as_deref()
    }

    pub fn set_objectiveai(&mut self, value: impl Into<String>) {
        self.objectiveai = Some(value.into());
    }

    pub fn get_openrouter(&self) -> Option<&str> {
        self.openrouter.as_deref()
    }

    pub fn set_openrouter(&mut self, value: impl Into<String>) {
        self.openrouter = Some(value.into());
    }

    pub fn get_github(&self) -> Option<&str> {
        self.github.as_deref()
    }

    pub fn set_github(&mut self, value: impl Into<String>) {
        self.github = Some(value.into());
    }

    pub fn add_mcp(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.mcp.get_or_insert_with(IndexMap::new).insert(key.into(), value.into());
    }

    pub fn del_mcp(&mut self, key: &str) {
        if let Some(mcp) = &mut self.mcp {
            mcp.shift_remove(key);
        }
    }

    pub fn jq(&self, filter: &str) -> Result<Vec<serde_json::Value>, super::ConfigError> {
        super::jq::run_jq(self, filter)
    }
}
