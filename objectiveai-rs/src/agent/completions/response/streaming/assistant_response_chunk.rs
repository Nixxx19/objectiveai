//! Streaming agent completion chunk type.

use crate::agent::completions::{request, response};
use serde::{Deserialize, Serialize};

/// A chunk of a streaming agent completion response.
///
/// Multiple chunks are received via Server-Sent Events and can be
/// accumulated into a complete [`AgentCompletion`](response::unary::AgentCompletion)
/// using the [`push`](Self::push) method.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AssistantResponseChunk {
    pub role: response::AssistantRole,
    pub index: u64,
    pub created: u64,
    pub agent: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<crate::agent::completions::request::AssistantToolCallDelta>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<request::RichContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    pub finish_reason: Option<response::FinishReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<response::Logprobs>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

impl AssistantResponseChunk {
    /// Accumulates another chunk into this one.
    ///
    /// This is used to build up a complete response from streaming chunks.
    pub fn push(
        &mut self,
        AssistantResponseChunk {
            reasoning,
            tool_calls,
            content,
            refusal,
            finish_reason,
            logprobs,
            upstream_id,
            service_tier,
            system_fingerprint,
            provider,
            ..
        }: &AssistantResponseChunk,
    ) {
        response::util::push_option_string(&mut self.reasoning, reasoning);
        self.push_tool_calls(tool_calls);
        match (&mut self.content, content) {
            (Some(self_content), Some(other_content)) => {
                self_content.push(other_content);
            }
            (None, Some(other_content)) => {
                self.content = Some(other_content.clone());
            }
            _ => {}
        }
        response::util::push_option_string(&mut self.refusal, refusal);
        if self.finish_reason.is_none() {
            self.finish_reason = finish_reason.clone();
        }
        match (&mut self.logprobs, logprobs) {
            (Some(self_logprobs), Some(other_logprobs)) => {
                self_logprobs.push(other_logprobs);
            }
            (None, Some(other_logprobs)) => {
                self.logprobs = Some(other_logprobs.clone());
            }
            _ => {}
        }
        if self.upstream_id.is_none() {
            self.upstream_id = upstream_id.clone();
        }
        if self.service_tier.is_none() {
            self.service_tier = service_tier.clone();
        }
        if self.system_fingerprint.is_none() {
            self.system_fingerprint = system_fingerprint.clone();
        }
        if self.provider.is_none() {
            self.provider = provider.clone();
        }
    }

    fn push_tool_calls(
        &mut self,
        other_tool_calls: &Option<Vec<crate::agent::completions::request::AssistantToolCallDelta>>,
    ) {
        fn push_tool_call(
            tool_calls: &mut Vec<crate::agent::completions::request::AssistantToolCallDelta>,
            other: &crate::agent::completions::request::AssistantToolCallDelta,
        ) {
            fn find_tool_call(
                tool_calls: &mut Vec<crate::agent::completions::request::AssistantToolCallDelta>,
                index: u64,
            ) -> Option<&mut crate::agent::completions::request::AssistantToolCallDelta> {
                for tool_call in tool_calls {
                    if tool_call.index == index {
                        return Some(tool_call);
                    }
                }
                None
            }
            if let Some(tool_call) = find_tool_call(tool_calls, other.index) {
                tool_call.push(other);
            } else {
                tool_calls.push(other.clone());
            }
        }
        match (self.tool_calls.as_mut(), other_tool_calls) {
            (Some(self_tool_calls), Some(other_tool_calls)) => {
                for other_tool_call in other_tool_calls {
                    push_tool_call(self_tool_calls, other_tool_call);
                }
            }
            (None, Some(other_tool_calls)) => {
                self.tool_calls = Some(other_tool_calls.clone());
            }
            _ => {}
        }
    }
}
