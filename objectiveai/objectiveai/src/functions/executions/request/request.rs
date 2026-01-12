use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionRequest {
    FunctionInlineProfileInline {
        body: super::FunctionInlineProfileInlineRequestBody,
    },
    FunctionInlineProfileRemote {
        path: super::FunctionInlineProfileRemoteRequestPath,
        body: super::FunctionInlineProfileRemoteRequestBody,
    },
    FunctionRemoteProfileInline {
        path: super::FunctionRemoteProfileInlineRequestPath,
        body: super::FunctionRemoteProfileInlineRequestBody,
    },
    FunctionRemoteProfileRemote {
        path: super::FunctionRemoteProfileRemoteRequestPath,
        body: super::FunctionRemoteProfileRemoteRequestBody,
    },
}
