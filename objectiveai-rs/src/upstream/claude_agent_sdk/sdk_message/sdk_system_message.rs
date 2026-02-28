use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKSystemMessageType {
    System,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKSystemMessageSubtype {
    Init,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct McpServerInfo {
    pub name: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PluginInfo {
    pub name: String,
    pub path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SDKSystemMessage {
    pub r#type: SDKSystemMessageType,
    pub subtype: SDKSystemMessageSubtype,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agents: Option<Vec<String>>,
    #[serde(rename = "apiKeySource")]
    pub api_key_source: super::ApiKeySource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub betas: Option<Vec<String>>,
    pub claude_code_version: String,
    pub cwd: String,
    pub tools: Vec<String>,
    pub mcp_servers: Vec<McpServerInfo>,
    pub model: String,
    #[serde(rename = "permissionMode")]
    pub permission_mode: super::PermissionMode,
    pub slash_commands: Vec<String>,
    pub output_style: String,
    pub skills: Vec<String>,
    pub plugins: Vec<PluginInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fast_mode_state: Option<super::FastModeState>,
    pub uuid: String,
    pub session_id: String,
}
