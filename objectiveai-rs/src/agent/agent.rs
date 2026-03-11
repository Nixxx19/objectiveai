//! Core Agent types — unified enum dispatching to per-upstream implementations.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// The base configuration for an Agent (without computed ID).
///
/// This is an untagged enum that dispatches to the per-upstream AgentBase.
/// Deserialization tries each variant in order until one matches.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(rename = "agent.AgentBase")]
pub enum AgentBase {
    Openrouter(super::openrouter::AgentBase),
    ClaudeAgentSdk(super::claude_agent_sdk::AgentBase),
    Mock(super::mock::AgentBase),
}

impl AgentBase {
    pub fn as_ref(&self) -> AgentBaseRef<'_> {
        match self {
            AgentBase::Openrouter(b) => AgentBaseRef::Openrouter(b),
            AgentBase::ClaudeAgentSdk(b) => AgentBaseRef::ClaudeAgentSdk(b),
            AgentBase::Mock(b) => AgentBaseRef::Mock(b),
        }
    }

    pub fn model(&self) -> &str {
        self.as_ref().model()
    }

    pub fn upstream(&self) -> super::Upstream {
        self.as_ref().upstream()
    }

    pub fn output_mode(&self) -> super::OutputMode {
        self.as_ref().output_mode()
    }

    pub fn mcp_servers(&self) -> Option<&super::McpServers> {
        self.as_ref().mcp_servers()
    }

    pub fn prepare(&mut self) {
        match self {
            AgentBase::Openrouter(b) => b.prepare(),
            AgentBase::ClaudeAgentSdk(b) => b.prepare(),
            AgentBase::Mock(b) => b.prepare(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        match self {
            AgentBase::Openrouter(b) => b.validate(),
            AgentBase::ClaudeAgentSdk(b) => b.validate(),
            AgentBase::Mock(b) => b.validate(),
        }
    }

    pub fn id(&self) -> String {
        match self {
            AgentBase::Openrouter(b) => b.id(),
            AgentBase::ClaudeAgentSdk(b) => b.id(),
            AgentBase::Mock(b) => b.id(),
        }
    }

}

/// A borrowed reference into an [`AgentBase`] variant.
#[derive(Clone, Copy, Debug)]
pub enum AgentBaseRef<'a> {
    Openrouter(&'a super::openrouter::AgentBase),
    ClaudeAgentSdk(&'a super::claude_agent_sdk::AgentBase),
    Mock(&'a super::mock::AgentBase),
}

impl<'a> AgentBaseRef<'a> {
    pub fn to_owned(self) -> AgentBase {
        match self {
            AgentBaseRef::Openrouter(b) => AgentBase::Openrouter(b.clone()),
            AgentBaseRef::ClaudeAgentSdk(b) => {
                AgentBase::ClaudeAgentSdk(b.clone())
            }
            AgentBaseRef::Mock(b) => AgentBase::Mock(b.clone()),
        }
    }

    pub fn model(&self) -> &'a str {
        match self {
            AgentBaseRef::Openrouter(b) => &b.model,
            AgentBaseRef::ClaudeAgentSdk(b) => &b.model,
            AgentBaseRef::Mock(_) => super::mock::AgentBase::model(),
        }
    }

    pub fn upstream(&self) -> super::Upstream {
        match self {
            AgentBaseRef::Openrouter(_) => super::Upstream::Openrouter,
            AgentBaseRef::ClaudeAgentSdk(_) => super::Upstream::ClaudeAgentSdk,
            AgentBaseRef::Mock(_) => super::Upstream::Mock,
        }
    }

    pub fn output_mode(&self) -> super::OutputMode {
        match self {
            AgentBaseRef::Openrouter(b) => b.output_mode.into(),
            AgentBaseRef::ClaudeAgentSdk(b) => b.output_mode.into(),
            AgentBaseRef::Mock(b) => b.output_mode.into(),
        }
    }

    pub fn mcp_servers(&self) -> Option<&'a super::McpServers> {
        match self {
            AgentBaseRef::Openrouter(b) => b.mcp_servers.as_ref(),
            AgentBaseRef::ClaudeAgentSdk(b) => b.mcp_servers.as_ref(),
            AgentBaseRef::Mock(_) => None,
        }
    }

    pub fn top_logprobs(&self) -> Option<u64> {
        match self {
            AgentBaseRef::Openrouter(b) => b.top_logprobs,
            AgentBaseRef::ClaudeAgentSdk(_) => None,
            AgentBaseRef::Mock(b) => b.top_logprobs,
        }
    }

    pub fn merged_messages(
        &self,
        messages: Vec<super::completions::message::Message>,
    ) -> Vec<super::completions::message::Message> {
        match self {
            AgentBaseRef::Openrouter(b) => b.merged_messages(messages),
            AgentBaseRef::ClaudeAgentSdk(b) => b.merged_messages(messages),
            AgentBaseRef::Mock(b) => b.merged_messages(messages),
        }
    }
}

/// A validated Agent with its computed content-addressed ID.
///
/// This is an untagged enum that dispatches to the per-upstream Agent.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(rename = "agent.Agent")]
pub enum Agent {
    Openrouter(super::openrouter::Agent),
    ClaudeAgentSdk(super::claude_agent_sdk::Agent),
    Mock(super::mock::Agent),
}

impl Agent {
    pub fn id(&self) -> &str {
        match self {
            Agent::Openrouter(a) => &a.id,
            Agent::ClaudeAgentSdk(a) => &a.id,
            Agent::Mock(a) => &a.id,
        }
    }

    pub fn base(&self) -> AgentBaseRef<'_> {
        match self {
            Agent::Openrouter(a) => AgentBaseRef::Openrouter(&a.base),
            Agent::ClaudeAgentSdk(a) => AgentBaseRef::ClaudeAgentSdk(&a.base),
            Agent::Mock(a) => AgentBaseRef::Mock(&a.base),
        }
    }

    pub fn into_base(self) -> AgentBase {
        match self {
            Agent::Openrouter(a) => AgentBase::Openrouter(a.base),
            Agent::ClaudeAgentSdk(a) => AgentBase::ClaudeAgentSdk(a.base),
            Agent::Mock(a) => AgentBase::Mock(a.base),
        }
    }

    pub fn top_logprobs(&self) -> Option<u64> {
        self.base().top_logprobs()
    }
}

impl TryFrom<AgentBase> for Agent {
    type Error = String;
    fn try_from(base: AgentBase) -> Result<Self, Self::Error> {
        match base {
            AgentBase::Openrouter(b) => Ok(Agent::Openrouter(b.try_into()?)),
            AgentBase::ClaudeAgentSdk(b) => {
                Ok(Agent::ClaudeAgentSdk(b.try_into()?))
            }
            AgentBase::Mock(b) => Ok(Agent::Mock(b.try_into()?)),
        }
    }
}

/// Wrapper that adds fallback agents and a count to any agent type.
///
/// Used to specify how many instances of an agent to include in an ensemble,
/// along with fallback agents to try if the primary fails.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "agent.WithFallbacksAndCount.{T}")]
pub struct WithFallbacksAndCount<T> {
    /// Number of instances of this agent in the ensemble. Defaults to 1.
    #[serde(default = "WithFallbacksAndCount::<T>::default_count")]
    pub count: u64,
    /// The primary agent configuration.
    #[serde(flatten)]
    pub inner: T,
    /// Fallback agents to try if the primary fails.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallbacks: Option<Vec<T>>,
}

impl<T> WithFallbacksAndCount<T> {
    fn default_count() -> u64 {
        1
    }
}

/// An [`AgentBase`] with optional fallbacks and count (pre-validation).
pub type AgentBaseWithFallbacksAndCount = WithFallbacksAndCount<AgentBase>;

/// A validated [`Agent`] with optional fallbacks and count.
pub type AgentWithFallbacksAndCount = WithFallbacksAndCount<Agent>;

impl AgentWithFallbacksAndCount {
    /// Returns the concatenated IDs of the primary agent and all fallbacks.
    ///
    /// Used by [`Ensemble`](crate::ensemble::Ensemble) to compute its own
    /// content-addressed ID.
    pub fn full_id(&self) -> String {
        match &self.fallbacks {
            Some(fallbacks) => {
                let id = self.inner.id();
                let mut full_id =
                    String::with_capacity(id.len() + fallbacks.len() * 22);
                full_id.push_str(id);
                for fallback in fallbacks {
                    full_id.push_str(fallback.id());
                }
                full_id
            }
            None => self.inner.id().to_owned(),
        }
    }

    /// Converts to a base variant, stripping the computed IDs.
    pub fn into_base(self) -> AgentBaseWithFallbacksAndCount {
        AgentBaseWithFallbacksAndCount {
            count: self.count,
            inner: self.inner.into_base(),
            fallbacks: self.fallbacks.map(|fallbacks| {
                fallbacks
                    .into_iter()
                    .map(|fallback| fallback.into_base())
                    .collect()
            }),
        }
    }

    /// Returns an iterator over the IDs of the primary agent and all fallbacks.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.inner.id()).chain(
            self.fallbacks.as_ref().into_iter().flat_map(|fallbacks| {
                fallbacks.iter().map(|fallback| fallback.id())
            }),
        )
    }
}

impl TryFrom<AgentBaseWithFallbacksAndCount> for AgentWithFallbacksAndCount {
    type Error = String;
    fn try_from(
        AgentBaseWithFallbacksAndCount {
            count,
            inner: base_inner,
            fallbacks: base_fallbacks,
        }: AgentBaseWithFallbacksAndCount,
    ) -> Result<Self, Self::Error> {
        let inner = base_inner.try_into()?;
        let fallbacks = match base_fallbacks {
            Some(base_fallbacks) if base_fallbacks.len() > 0 => {
                let mut fallbacks = Vec::with_capacity(base_fallbacks.len());
                for base_fallback in base_fallbacks {
                    fallbacks.push(base_fallback.try_into()?);
                }
                Some(fallbacks)
            }
            _ => None,
        };
        Ok(AgentWithFallbacksAndCount {
            count,
            inner,
            fallbacks,
        })
    }
}
