use std::pin::Pin;
use std::sync::Arc;

use futures::Stream;

use super::super::{ContinuationItem, StreamItem, UpstreamClient};

pub struct Client {}

impl Client {
    /// Validates that tool messages only appear before a State item.
    ///
    /// After the final State item in the continuation, no ToolMessage
    /// items are allowed.
    fn validate_continuation(
        continuation: &[ContinuationItem<super::State>],
    ) -> Result<(), super::Error> {
        let last_state_pos = continuation
            .iter()
            .rposition(|item| matches!(item, ContinuationItem::State(_)));

        if let Some(pos) = last_state_pos {
            for (i, item) in continuation.iter().enumerate() {
                if i > pos && matches!(item, ContinuationItem::ToolMessage(_)) {
                    return Err(super::Error::InvalidContinuation(
                        "tool messages must precede a state item".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }
}

impl UpstreamClient<objectiveai::agent::claude_agent_sdk::Agent> for Client {
    type State = super::State;
    type Stream = Pin<
        Box<dyn Stream<Item = StreamItem<Self::State>> + Send + 'static>,
    >;

    #[allow(unused_variables)]
    fn create(
        &self,
        id: &str,
        created: u64,
        agent: &objectiveai::agent::claude_agent_sdk::Agent,
        params: &objectiveai::agent::completions::request::AgentCompletionCreateParams,
        messages: &[objectiveai::agent::completions::message::Message],
        mcp_connections: &[Arc<crate::mcp::Connection>],
        invention_tools: Option<
            &[objectiveai::functions::inventions::InventionTool],
        >,
        continuation: Option<&[ContinuationItem<Self::State>]>,
        byok: Option<&str>,
        cost_multiplier: rust_decimal::Decimal,
    ) -> impl Future<
        Output = Result<
            (Self::Stream, Self::State),
            objectiveai::error::ResponseError,
        >,
    > + Send
    + 'static {
        let byok = byok.is_some();
        let continuation_result = continuation
            .map(|c| Self::validate_continuation(c))
            .unwrap_or(Ok(()));

        async move {
            if byok {
                return Err(objectiveai::error::ResponseError::from(
                    &super::Error::InvalidByok,
                ));
            }

            continuation_result.map_err(|e| {
                objectiveai::error::ResponseError::from(&e)
            })?;

            unimplemented!()
        }
    }
}
