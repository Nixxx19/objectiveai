#[derive(Debug, Clone)]
pub struct State {
    pub session_id: String,
    /// Number of messages (assistant turns + tool responses) produced in this turn.
    pub message_count: u64,
}
