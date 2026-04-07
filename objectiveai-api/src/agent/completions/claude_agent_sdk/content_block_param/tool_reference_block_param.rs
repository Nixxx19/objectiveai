use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolReferenceBlockParamType {
    ToolReference,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ToolReferenceBlockParam {
    pub tool_name: String,
    pub r#type: ToolReferenceBlockParamType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<super::CacheControlEphemeral>,
}
