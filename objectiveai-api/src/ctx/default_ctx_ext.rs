/// Default context extension
#[derive(Clone)]
pub struct DefaultContextExt;

#[async_trait::async_trait]
impl super::ContextExt for DefaultContextExt {
    async fn get_openrouter_byok(&self) -> Option<std::sync::Arc<String>> {
        None
    }

    async fn get_github_byok(&self) -> Option<std::sync::Arc<String>> {
        None
    }

    async fn get_mcp_byok(
        &self,
    ) -> Option<std::sync::Arc<std::collections::HashMap<String, String>>> {
        None
    }
}
