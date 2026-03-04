use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CacheControlEphemeralType {
    Ephemeral,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CacheControlTtl {
    #[serde(rename = "5m")]
    FiveMinutes,
    #[serde(rename = "1h")]
    OneHour,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheControlEphemeral {
    pub r#type: CacheControlEphemeralType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<CacheControlTtl>,
}
