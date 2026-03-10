//! Context extension trait for per-request customization.

/// Extension trait for providing per-request customization.
///
/// Implementations can provide BYOK (Bring Your Own Key) API keys
/// for upstream providers, allowing users to use their own API keys
/// instead of ObjectiveAI's pooled keys.
#[async_trait::async_trait]
pub trait ContextExt {
    /// Returns the user's BYOK OpenRouter API key.
    async fn get_openrouter_byok(&self) -> Option<std::sync::Arc<String>>;

    /// Returns the user's BYOK GitHub authorization token.
    async fn get_github_byok(&self) -> Option<std::sync::Arc<String>>;

    /// Returns the user's BYOK MCP authorization headers.
    async fn get_mcp_byok(
        &self,
    ) -> Option<std::sync::Arc<std::collections::HashMap<String, String>>>;
}
