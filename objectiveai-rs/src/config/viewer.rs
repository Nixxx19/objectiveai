use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewerMode {
    Remote,
    #[default]
    Local,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ViewerConfig {
    #[serde(default)]
    pub mode: ViewerMode,
    #[serde(skip_serializing_if = "ViewerLocalConfig::is_none")]
    pub local: Option<ViewerLocalConfig>,
}

impl ViewerConfig {
    pub fn is_empty(&self) -> bool {
        matches!(self.mode, ViewerMode::Local)
            && self.local.as_ref().is_none_or(|cfg| cfg.is_empty())
    }

    pub fn is_none(this: &Option<Self>) -> bool {
        this.as_ref().is_none_or(|cfg| cfg.is_empty())
    }

    pub fn local(&mut self) -> &mut ViewerLocalConfig {
        self.local.get_or_insert_with(ViewerLocalConfig::default)
    }

    pub fn get_mode(&self) -> ViewerMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: ViewerMode) {
        self.mode = mode;
    }

    pub fn jq(&self, filter: &str) -> Result<Vec<serde_json::Value>, super::ConfigError> {
        super::jq::run_jq(self, filter)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ViewerLocalConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl ViewerLocalConfig {
    pub fn is_empty(&self) -> bool {
        self.secret.is_none() && self.signature.is_none()
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

    pub fn get_signature(&self) -> Option<&str> {
        self.signature.as_deref()
    }

    pub fn set_signature(&mut self, value: impl Into<String>) {
        self.signature = Some(value.into());
    }

    pub fn jq(&self, filter: &str) -> Result<Vec<serde_json::Value>, super::ConfigError> {
        super::jq::run_jq(self, filter)
    }
}
