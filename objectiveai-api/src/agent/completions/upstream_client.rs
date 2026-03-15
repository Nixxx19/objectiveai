use std::collections::HashMap;
use std::sync::Arc;

pub trait UpstreamError:
    std::error::Error + objectiveai::error::StatusError + Send + Sync + 'static
{
}

impl<T> UpstreamError for T where
    T: std::error::Error + objectiveai::error::StatusError + Send + Sync + 'static
{
}

/// The first stream item must never be an error chunk. If the upstream
/// would fail before producing any non-error chunk, it must return
/// `Err(...)` from `create` instead of yielding an error chunk into
/// the stream.
///
/// The stream must never be empty. If the upstream produces no chunks
/// at all, it must return `Err(...)` from `create` instead of an
/// empty stream.
pub trait UpstreamClient<AGENT> {
    type State: Send + Sync + 'static;
    type Stream: futures::Stream<Item = StreamItem<Self::State>>
        + Send
        + 'static;
    type Error: UpstreamError;
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
        // when false, the model should not be allowed to call tools
        tools_enabled: bool,
    ) -> impl Future<
        Output = Result<
            Self::Stream,
            Self::Error,
        >,
    > + Send
    + 'static;
}

pub struct UnimplementedUpstreamClient;

impl<AGENT> UpstreamClient<AGENT> for UnimplementedUpstreamClient {
    type State = ();
    type Stream = futures::stream::Empty<StreamItem<Self::State>>;
    type Error = objectiveai::error::ResponseError;
    fn create(
        &self,
        _id: &str,
        _created: u64,
        _agent: &AGENT,
        _params: &objectiveai::agent::completions::request::AgentCompletionCreateParams,
        _messages: &[objectiveai::agent::completions::message::Message],
        _mcp_connections: &[Arc<crate::mcp::Connection>],
        _invention_tools: Option<
            &[objectiveai::functions::inventions::InventionTool],
        >,
        _tool_names: &[String],
        _tool_map: &HashMap<String, super::tool::ResolvedTool>,
        _continuation: Option<&[super::ContinuationItem<Self::State>]>,
        _byok: Option<&str>,
        _cost_multiplier: rust_decimal::Decimal,
        _tools_enabled: bool,
    ) -> impl Future<
        Output = Result<
            Self::Stream,
            Self::Error,
        >,
    > + Send
    + 'static {
        async {
            Err(
                objectiveai::error::ResponseError {
                    code: 501,
                    message: serde_json::Value::Null,
                }
            )
        }
    }
}

#[derive(Debug, Clone)]
pub enum StreamItem<STATE> {
    Chunk(objectiveai::agent::completions::response::streaming::AgentCompletionChunk),
    State(STATE),
}
