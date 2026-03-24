use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ViewerConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
}

impl ViewerConfig {
    pub fn is_empty(&self) -> bool {
        self.secret.is_none()
    }

    pub fn is_none(this: &Option<Self>) -> bool {
        this.as_ref().is_none_or(|cfg| cfg.is_empty())
    }

    pub fn get_secret(&self) -> Option<&str> {
        self.secret.as_deref()
    }

    pub fn set_secret(&mut self, value: impl Into<String>) {
        self.secret = Some(value.into());
    }

    pub fn jq(&self, filter: &str) -> Result<Vec<serde_json::Value>, super::ConfigError> {
        super::jq::run_jq(self, filter)
    }
}
