//! Prompt construction for OpenRouter requests.
//!
//! Handles merging prefix/suffix messages from Agent configurations
//! with the request messages.

/// Constructs the message array for a agent completion.
///
/// Concatenates the Agent's prefix messages, the request messages,
/// and the Agent's suffix messages.
pub fn new(
    agent_prefix: Option<&[crate::agent::completions::message::Message]>,
    request: &[crate::agent::completions::message::Message],
    agent_suffix: Option<&[crate::agent::completions::message::Message]>,
) -> Vec<crate::agent::completions::message::Message> {
    match (agent_prefix, agent_suffix) {
        (Some(agent_prefix), Some(agent_suffix)) => {
            let mut messages = Vec::with_capacity(
                agent_prefix.len() + request.len() + agent_suffix.len(),
            );
            messages.extend_from_slice(agent_prefix);
            messages.extend_from_slice(request);
            messages.extend_from_slice(agent_suffix);
            messages
        }
        (Some(agent_prefix), None) => {
            let mut messages = Vec::with_capacity(agent_prefix.len() + request.len());
            messages.extend_from_slice(agent_prefix);
            messages.extend_from_slice(request);
            messages
        }
        (None, Some(agent_suffix)) => {
            let mut messages = Vec::with_capacity(request.len() + agent_suffix.len());
            messages.extend_from_slice(request);
            messages.extend_from_slice(agent_suffix);
            messages
        }
        (None, None) => request.to_vec(),
    }
}
