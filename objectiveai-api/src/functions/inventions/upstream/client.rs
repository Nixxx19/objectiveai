use std::sync::Arc;

/// Client that manages connections to upstream providers for function inventions.
///
/// Dispatches to provider-specific clients that stream invention CompletionChunk
/// types (Chat with index, Tool with index).
#[derive(Debug, Clone)]
pub struct Client {
    // provider-specific clients go here (e.g. openrouter, claude_agent_sdk)
}

impl Client {
    /// Creates a new upstream client.
    pub fn new() -> Self {
        Self {}
    }

    /// Creates a streaming invention completion.
    ///
    /// Returns a stream of `CompletionChunk`s and the upstream state as JSON
    /// (for passing back to the upstream on subsequent calls within the same step).
    pub async fn create_streaming(
        &self,
        _request: Arc<
            objectiveai::functions::inventions::request::FunctionInventionCreateParams,
        >,
        _prompt: String,
        _tools: Vec<objectiveai::upstream::Tool>,
        _upstream_state: Option<serde_json::Value>,
    ) -> Result<
        (
            futures::stream::Empty<
                objectiveai::functions::inventions::response::streaming::CompletionChunk,
            >,
            serde_json::Value,
        ),
        super::Error,
    >{
        Err(super::Error::NoUpstreamAvailable)
    }
}
