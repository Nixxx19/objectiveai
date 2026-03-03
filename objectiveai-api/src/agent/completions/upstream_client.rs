pub trait UpstreamClient<AGENT> {
    type State: Send + Sync + 'static;
    type Stream: futures::Stream<Item = StreamItem<Self::State>>
        + Send
        + 'static;
    fn create(
        &self,
        agent: &AGENT,
        params: &objectiveai::agent::completions::request::AgentCompletionCreateParams,
        continuation: Option<&[ContinuationItem<Self::State>]>,
        invention_tools: Option<
            &[objectiveai::functions::inventions::InventionTool],
        >,
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

#[derive(Debug, Clone)]
pub enum ContinuationItem<STATE> {
    State(STATE),
    ToolMessage(objectiveai::agent::completions::message::ToolMessage),
    UserMessage(objectiveai::agent::completions::message::UserMessage),
}
