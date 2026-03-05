use std::collections::HashMap;
use std::sync::Arc;

/// The first stream item must never be an error chunk. If the upstream
/// would fail before producing any non-error chunk, it must return
/// `Err(...)` from `create` instead of yielding an error chunk into
/// the stream.
pub trait UpstreamClient<AGENT> {
    type State: Send + Sync + 'static;
    type Stream: futures::Stream<Item = StreamItem<Self::State>>
        + Send
        + 'static;
    fn create(
        &self,
        // unique identifier for this completion
        id: &str,
        // unix timestamp when the completion was created
        created: u64,
        // the agent that the upstream client uses
        agent: &AGENT,
        // the original request params for the agent completion
        params: &objectiveai::agent::completions::request::AgentCompletionCreateParams,
        // contains the full prompt, including from the params and from the agent
        // upstream clients do not handle merging params and agent messages
        messages: &[objectiveai::agent::completions::message::Message],
        // optionally used by some upstreams to handle MCP internally
        // but may be safely ignored by others that want to use continuation for that instead
        mcp_connections: &[Arc<crate::mcp::Connection>],
        // optionally used by some upstreams to handle invention tools internally
        // but may be safely ignored by others that want to use continuation for that instead
        invention_tools: Option<
            &[objectiveai::functions::inventions::InventionTool],
        >,
        // resolved tool names (in order) from tool::resolve_tools
        tool_names: &[String],
        // map from resolved tool name to its origin
        tool_map: &HashMap<String, super::tool::ResolvedTool>,
        // a continuation from a previous agent completion
        // the upstream client can continue conversations from previous state
        // the agent may change
        continuation: Option<&[super::ContinuationItem<Self::State>]>,
        // optional user-provided API key (BYOK) — used as authorization if provided
        byok: Option<&str>,
        // cost multiplier for usage reporting
        cost_multiplier: rust_decimal::Decimal,
    ) -> impl Future<
        Output = Result<
            (Self::Stream, Self::State),
            objectiveai::error::ResponseError,
        >,
    > + Send
    + 'static;
}

#[derive(Debug, Clone)]
pub enum StreamItem<STATE> {
    Chunk(objectiveai::agent::completions::response::streaming::AgentCompletionChunk),
    State(STATE),
}
