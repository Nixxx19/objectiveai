use serde::{Deserialize, Serialize};

/// Output of a config getter. One variant per logical config family;
/// the payload is the typed value as it lives in the on-disk config
/// (no `Option<&str>` smuggling, no stringification).
///
/// `Jq` is the one untyped escape hatch: a user-supplied jq filter
/// over the config can return literally any JSON shape, including
/// mixed-type arrays, so it must accept `Vec<serde_json::Value>`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "subkind", rename_all = "snake_case")]
pub enum Config {
    /// Emitted by `api mode config get`.
    ApiMode {
        value: objectiveai::filesystem::config::ApiMode,
    },
    /// Emitted by `api remote config get`.
    ApiRemote {
        value: objectiveai::filesystem::config::ApiRemoteConfig,
    },
    /// Emitted by `api headers config get` (the full headers bag).
    ApiHeaders {
        value: objectiveai::filesystem::config::ApiHeadersConfig,
    },
    /// Emitted by `api headers x_objectiveai_authorization config get`.
    ApiHeaderXObjectiveaiAuthorization { value: Option<String> },
    /// Emitted by `api headers x_openrouter_authorization config get`.
    ApiHeaderXOpenrouterAuthorization { value: Option<String> },
    /// Emitted by `api headers x_github_authorization config get`.
    ApiHeaderXGithubAuthorization { value: Option<String> },
    /// Emitted by `api headers x_mcp_authorization config get`. MCP
    /// authorization is keyed per server, hence the map.
    ApiHeaderXMcpAuthorization {
        value: Option<indexmap::IndexMap<String, String>>,
    },
    /// Emitted by `api headers x_viewer_signature config get`.
    ApiHeaderXViewerSignature { value: Option<String> },
    /// Emitted by `api headers x_viewer_address config get`.
    ApiHeaderXViewerAddress { value: Option<String> },
    /// Emitted by `api headers user_agent config get`.
    ApiHeaderUserAgent { value: Option<String> },
    /// Emitted by `api headers http_referer config get`.
    ApiHeaderHttpReferer { value: Option<String> },
    /// Emitted by `api headers x_title config get`.
    ApiHeaderXTitle { value: Option<String> },
    /// Emitted by `api headers x_commit_author_name config get`.
    ApiHeaderXCommitAuthorName { value: Option<String> },
    /// Emitted by `api headers x_commit_author_email config get`.
    ApiHeaderXCommitAuthorEmail { value: Option<String> },
    /// Emitted by `agents config get`.
    Agents {
        value: objectiveai::filesystem::config::AgentsConfig,
    },
    /// Emitted by `swarms config get`.
    Swarms {
        value: objectiveai::filesystem::config::SwarmsConfig,
    },
    /// Emitted by `functions config get`.
    Functions {
        value: objectiveai::filesystem::config::FunctionsConfig,
    },
    /// Emitted by `functions profiles config get`.
    FunctionsProfiles {
        value: objectiveai::filesystem::config::FunctionsProfilesConfig,
    },
    /// Emitted by `functions profiles pairs config get`.
    FunctionsProfilesPairs {
        value: objectiveai::filesystem::config::FunctionsProfilesPairsConfig,
    },
    /// Emitted by `viewer mode config get`.
    ViewerMode {
        value: objectiveai::filesystem::config::ViewerMode,
    },
    /// Emitted by any config getter when the user passes a `--filter`
    /// jq expression. The filter is user-supplied and may return any
    /// JSON shape, so this is the one place untyped values appear.
    Jq { results: Vec<serde_json::Value> },
}
