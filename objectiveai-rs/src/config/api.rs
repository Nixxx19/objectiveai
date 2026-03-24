use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiConfig {
    #[serde(default)]
    pub mode: ApiMode,
    #[serde(skip_serializing_if = "RemoteApiConfig::is_none")]
    pub remote: Option<RemoteApiConfig>,
    #[serde(skip_serializing_if = "LocalApiConfig::is_none")]
    pub local: Option<LocalApiConfig>,
    #[serde(skip_serializing_if = "ApiHeadersConfig::is_none")]
    pub headers: Option<ApiHeadersConfig>,
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

    pub fn headers(&mut self) -> &mut ApiHeadersConfig {
        self.headers.get_or_insert_with(ApiHeadersConfig::default)
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
}

impl RemoteApiConfig {
    pub fn is_empty(&self) -> bool {
        self.objectiveai_api_base.is_none()
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

    pub fn jq(&self, filter: &str) -> Result<Vec<serde_json::Value>, super::ConfigError> {
        super::jq::run_jq(self, filter)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocalApiConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claude_agent_sdk: Option<bool>,
}

impl LocalApiConfig {
    pub fn is_empty(&self) -> bool {
        self.claude_agent_sdk.is_none()
    }

    pub fn is_none(this: &Option<Self>) -> bool {
        this.as_ref().is_none_or(|cfg| cfg.is_empty())
    }

    pub fn get_claude_agent_sdk(&self) -> Option<bool> {
        self.claude_agent_sdk
    }

    pub fn set_claude_agent_sdk(&mut self, value: bool) {
        self.claude_agent_sdk = Some(value);
    }

    pub fn jq(&self, filter: &str) -> Result<Vec<serde_json::Value>, super::ConfigError> {
        super::jq::run_jq(self, filter)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiHeadersConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objectiveai_authorization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openrouter_authorization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_authorization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_authorization: Option<indexmap::IndexMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer_signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewer_address: Option<String>,
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

impl ApiHeadersConfig {
    pub fn is_empty(&self) -> bool {
        self.objectiveai_authorization.is_none()
            && self.openrouter_authorization.is_none()
            && self.github_authorization.is_none()
            && self.mcp_authorization.as_ref().is_none_or(|m| m.is_empty())
            && self.viewer_signature.is_none()
            && self.viewer_address.is_none()
            && self.user_agent.is_none()
            && self.http_referer.is_none()
            && self.x_title.is_none()
            && self.commit_author_name.is_none()
            && self.commit_author_email.is_none()
    }

    pub fn is_none(this: &Option<Self>) -> bool {
        this.as_ref().is_none_or(|cfg| cfg.is_empty())
    }

    pub fn get_objectiveai_authorization(&self) -> Option<&str> {
        self.objectiveai_authorization.as_deref()
    }

    pub fn set_objectiveai_authorization(&mut self, value: impl Into<String>) {
        self.objectiveai_authorization = Some(value.into());
    }

    pub fn get_openrouter_authorization(&self) -> Option<&str> {
        self.openrouter_authorization.as_deref()
    }

    pub fn set_openrouter_authorization(&mut self, value: impl Into<String>) {
        self.openrouter_authorization = Some(value.into());
    }

    pub fn get_github_authorization(&self) -> Option<&str> {
        self.github_authorization.as_deref()
    }

    pub fn set_github_authorization(&mut self, value: impl Into<String>) {
        self.github_authorization = Some(value.into());
    }

    pub fn get_mcp_authorization(&self) -> Option<&indexmap::IndexMap<String, String>> {
        self.mcp_authorization.as_ref()
    }

    pub fn add_mcp_authorization(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.mcp_authorization.get_or_insert_with(indexmap::IndexMap::new).insert(key.into(), value.into());
    }

    pub fn del_mcp_authorization(&mut self, key: &str) {
        if let Some(mcp) = &mut self.mcp_authorization {
            mcp.shift_remove(key);
        }
    }

    pub fn get_viewer_signature(&self) -> Option<&str> {
        self.viewer_signature.as_deref()
    }

    pub fn set_viewer_signature(&mut self, value: impl Into<String>) {
        self.viewer_signature = Some(value.into());
    }

    pub fn get_viewer_address(&self) -> Option<&str> {
        self.viewer_address.as_deref()
    }

    pub fn set_viewer_address(&mut self, value: impl Into<String>) {
        self.viewer_address = Some(value.into());
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
