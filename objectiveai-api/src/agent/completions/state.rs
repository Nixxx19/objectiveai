/// The upstream client state for an agent completion, parameterized by client type.
#[derive(Debug, Clone)]
pub enum State<OPENROUTER, CLAUDEAGENTSDK, MOCK> {
    Openrouter(OPENROUTER),
    ClaudeAgentSdk(CLAUDEAGENTSDK),
    Mock(MOCK),
}
