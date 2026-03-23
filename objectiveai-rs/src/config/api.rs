use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiConfig {
    #[serde(default)]
    pub mode: ApiMode,
    #[serde(skip_serializing_if = "RemoteApiConfig::is_none")]
    pub remote: Option<RemoteApiConfig>,
    #[serde(skip_serializing_if = "LocalApiConfig::is_none")]
    pub local: Option<LocalApiConfig>,
    #[serde(skip_serializing_if = "ApiAuthorizationConfig::is_none")]
    pub authorization: Option<ApiAuthorizationConfig>,
}

impl ApiConfig {
    pub fn is_empty(&self) -> bool {
        false
    }

    pub fn is_none(this: &Option<Self>) -> bool {
        this.as_ref().is_none_or(|cfg| cfg.is_empty())
    }

    pub fn remote(&mut self) -> &mut RemoteApiConfig {
        self.remote.get_or_insert_with(RemoteApiConfig::default)
    }

    pub fn local(&mut self) -> &mut LocalApiConfig {
        self.local.get_or_insert_with(LocalApiConfig::default)
    }

    pub fn authorization(&mut self) -> &mut ApiAuthorizationConfig {
        self.authorization.get_or_insert_with(ApiAuthorizationConfig::default)
    }

    pub fn get_mode(&self) -> ApiMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: ApiMode) {
        self.mode = mode;
    }

    pub fn jq(&self, filter: &str) -> Result<Vec<serde_json::Value>, super::ConfigError> {
        super::jq::run_jq(self, filter)
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiMode {
    Remote,
    #[default]
    Local,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RemoteApiConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objectiveai_api_base: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objectiveai_api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_referer: Option<String>,
}

impl RemoteApiConfig {
    pub fn is_empty(&self) -> bool {
        self.objectiveai_api_base.is_none()
            && self.objectiveai_api_key.is_none()
            && self.user_agent.is_none()
            && self.x_title.is_none()
            && self.http_referer.is_none()
    }

    pub fn is_none(this: &Option<Self>) -> bool {
        this.as_ref().is_none_or(|cfg| cfg.is_empty())
    }

    pub fn get_objectiveai_api_base(&self) -> Option<&str> {
        self.objectiveai_api_base.as_deref()
    }

    pub fn set_objectiveai_api_base(&mut self, value: impl Into<String>) {
        self.objectiveai_api_base = Some(value.into());
    }

    pub fn get_objectiveai_api_key(&self) -> Option<&str> {
        self.objectiveai_api_key.as_deref()
    }

    pub fn set_objectiveai_api_key(&mut self, value: impl Into<String>) {
        self.objectiveai_api_key = Some(value.into());
    }

    pub fn get_user_agent(&self) -> Option<&str> {
        self.user_agent.as_deref()
    }

    pub fn set_user_agent(&mut self, value: impl Into<String>) {
        self.user_agent = Some(value.into());
    }

    pub fn get_x_title(&self) -> Option<&str> {
        self.x_title.as_deref()
    }

    pub fn set_x_title(&mut self, value: impl Into<String>) {
        self.x_title = Some(value.into());
    }

    pub fn get_http_referer(&self) -> Option<&str> {
        self.http_referer.as_deref()
    }

    pub fn set_http_referer(&mut self, value: impl Into<String>) {
        self.http_referer = Some(value.into());
    }

    pub fn jq(&self, filter: &str) -> Result<Vec<serde_json::Value>, super::ConfigError> {
        super::jq::run_jq(self, filter)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocalApiConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objectiveai_api_base: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objectiveai_api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openrouter_api_base: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openrouter_api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claude_agent_sdk: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_referer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_author_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_author_email: Option<String>,
}

impl LocalApiConfig {
    pub fn is_empty(&self) -> bool {
        self.objectiveai_api_base.is_none()
            && self.objectiveai_api_key.is_none()
            && self.openrouter_api_base.is_none()
            && self.openrouter_api_key.is_none()
            && self.claude_agent_sdk.is_none()
            && self.user_agent.is_none()
            && self.http_referer.is_none()
            && self.x_title.is_none()
            && self.commit_author_name.is_none()
            && self.commit_author_email.is_none()
    }

    pub fn is_none(this: &Option<Self>) -> bool {
        this.as_ref().is_none_or(|cfg| cfg.is_empty())
    }

    pub fn get_objectiveai_api_base(&self) -> Option<&str> {
        self.objectiveai_api_base.as_deref()
    }

    pub fn set_objectiveai_api_base(&mut self, value: impl Into<String>) {
        self.objectiveai_api_base = Some(value.into());
    }

    pub fn get_objectiveai_api_key(&self) -> Option<&str> {
        self.objectiveai_api_key.as_deref()
    }

    pub fn set_objectiveai_api_key(&mut self, value: impl Into<String>) {
        self.objectiveai_api_key = Some(value.into());
    }

    pub fn get_openrouter_api_base(&self) -> Option<&str> {
        self.openrouter_api_base.as_deref()
    }

    pub fn set_openrouter_api_base(&mut self, value: impl Into<String>) {
        self.openrouter_api_base = Some(value.into());
    }

    pub fn get_openrouter_api_key(&self) -> Option<&str> {
        self.openrouter_api_key.as_deref()
    }

    pub fn set_openrouter_api_key(&mut self, value: impl Into<String>) {
        self.openrouter_api_key = Some(value.into());
    }

    pub fn get_claude_agent_sdk(&self) -> Option<bool> {
        self.claude_agent_sdk
    }

    pub fn set_claude_agent_sdk(&mut self, value: bool) {
        self.claude_agent_sdk = Some(value);
    }

    pub fn get_user_agent(&self) -> Option<&str> {
        self.user_agent.as_deref()
    }

    pub fn set_user_agent(&mut self, value: impl Into<String>) {
        self.user_agent = Some(value.into());
    }

    pub fn get_http_referer(&self) -> Option<&str> {
        self.http_referer.as_deref()
    }

    pub fn set_http_referer(&mut self, value: impl Into<String>) {
        self.http_referer = Some(value.into());
    }

    pub fn get_x_title(&self) -> Option<&str> {
        self.x_title.as_deref()
    }

    pub fn set_x_title(&mut self, value: impl Into<String>) {
        self.x_title = Some(value.into());
    }

    pub fn get_commit_author_name(&self) -> Option<&str> {
        self.commit_author_name.as_deref()
    }

    pub fn set_commit_author_name(&mut self, value: impl Into<String>) {
        self.commit_author_name = Some(value.into());
    }

    pub fn get_commit_author_email(&self) -> Option<&str> {
        self.commit_author_email.as_deref()
    }

    pub fn set_commit_author_email(&mut self, value: impl Into<String>) {
        self.commit_author_email = Some(value.into());
    }

    pub fn jq(&self, filter: &str) -> Result<Vec<serde_json::Value>, super::ConfigError> {
        super::jq::run_jq(self, filter)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiAuthorizationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objectiveai: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openrouter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp: Option<indexmap::IndexMap<String, String>>,
}

impl ApiAuthorizationConfig {
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
        self.mcp.get_or_insert_with(indexmap::IndexMap::new).insert(key.into(), value.into());
    }

    pub fn get_mcp(&self) -> Option<&indexmap::IndexMap<String, String>> {
        self.mcp.as_ref()
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
