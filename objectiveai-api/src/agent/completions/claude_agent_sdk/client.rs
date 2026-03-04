use std::pin::Pin;
use std::sync::Arc;

use futures::Stream;

use super::super::{ContinuationItem, StreamItem, UpstreamClient};

pub struct Client {}

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
        async { unimplemented!() }
    }
}
