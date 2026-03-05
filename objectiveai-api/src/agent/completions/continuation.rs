/// The upstream client state for an agent completion, parameterized by client type.
#[derive(Debug, Clone)]
pub enum State<OPENROUTER, CLAUDEAGENTSDK, MOCK> {
    Openrouter(OPENROUTER),
    ClaudeAgentSdk(CLAUDEAGENTSDK),
    Mock(MOCK),
}

#[derive(Debug, Clone)]
pub enum Continuation<OPENROUTER, CLAUDEAGENTSDK, MOCK> {
    Openrouter(Vec<ContinuationItem<OPENROUTER>>),
    ClaudeAgentSdk(Vec<ContinuationItem<CLAUDEAGENTSDK>>),
    Mock(Vec<ContinuationItem<MOCK>>),
}

#[derive(Debug, Clone)]
pub enum ContinuationItem<STATE> {
    State(STATE),
    ToolMessage(objectiveai::agent::completions::message::ToolMessage),
    UserMessage(objectiveai::agent::completions::message::UserMessage),
}
