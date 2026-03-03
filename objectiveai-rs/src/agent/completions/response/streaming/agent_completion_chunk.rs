//! Streaming agent completion chunk type.

use crate::agent::completions::response;
use serde::{Deserialize, Serialize};

/// A chunk of a streaming agent completion response.
///
/// Multiple chunks are received via Server-Sent Events and can be
/// accumulated into a complete [`AgentCompletion`](response::unary::AgentCompletion)
/// using the [`push`](Self::push) method.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AgentCompletionChunk {
    pub id: String,
    pub created: u64,
    pub messages: Vec<super::MessageChunk>,
    /// The object type (always "agent.completion.chunk").
    pub object: super::Object,
    /// Token usage (only present in the final chunk).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<response::Usage>,
    /// Upstream provider
    pub upstream: crate::agent::Upstream,
    /// Error details if this completion failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<crate::error::ResponseError>,
}

impl AgentCompletionChunk {
    /// Accumulates another chunk into this one.
    ///
    /// This is used to build up a complete response from streaming chunks.
    pub fn push(
        &mut self,
        AgentCompletionChunk {
            messages, usage, error, ..
        }: &AgentCompletionChunk,
    ) {
        self.push_messages(messages);
        match (&mut self.usage, usage) {
            (Some(self_usage), Some(other_usage)) => {
                self_usage.push(other_usage);
            }
            (None, Some(other_usage)) => {
                self.usage = Some(other_usage.clone());
            }
            _ => {}
        }
        if self.error.is_none() {
            if let Some(other_error) = error {
                self.error = Some(other_error.clone());
            }
        }
    }

    fn push_messages(&mut self, other_choices: &[super::MessageChunk]) {
        fn push_message(
            messages: &mut Vec<super::MessageChunk>,
            other: &super::MessageChunk,
        ) {
            fn find_message(
                messages: &mut Vec<super::MessageChunk>,
                index: u64,
            ) -> Option<&mut super::MessageChunk> {
                for message in messages {
                    if message.index() == index {
                        return Some(message);
                    }
                }
                None
            }
            if let Some(message) = find_message(messages, other.index()) {
                message.push(other);
            } else {
                messages.push(other.clone());
            }
        }
        for other_message in other_choices {
            push_message(&mut self.messages, other_message);
        }
    }
}
