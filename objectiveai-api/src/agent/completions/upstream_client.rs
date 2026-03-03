pub trait UpstreamClient<AGENT> {
    type State: Clone + Send + Sync + 'static;
    type Stream: futures::Stream<Item = StreamItem<Self::State>>
        + Send
        + 'static;
    async fn create(
        &self,
        agent: &AGENT,
        params: &objectiveai::agent::completions::request::AgentCompletionCreateParams,
        invention_tools: Option<
            Vec<objectiveai::functions::inventions::InventionTool>,
        >,
        continuation: Option<Vec<ContinuationItem<Self::State>>>,
    ) -> Result<(Self::Stream, Self::State), objectiveai::error::ResponseError>;
}

#[derive(Debug, Clone)]
pub enum StreamItem<STATE> {
    Chunk(objectiveai::agent::completions::response::streaming::AgentCompletionChunk),
    State(STATE),
}

#[derive(Debug, Clone)]
pub enum ContinuationItem<STATE> {
    State(STATE),
    ToolMessage(objectiveai::agent::completions::message::ToolMessage),
    UserMessage(objectiveai::agent::completions::message::UserMessage),
}
