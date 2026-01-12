use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    pub expires: Option<chrono::DateTime<chrono::Utc>>,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisableApiKeyRequest {
    pub api_key: super::ApiKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOpenRouterByokApiKeyRequest {
    pub api_key: String,
}
