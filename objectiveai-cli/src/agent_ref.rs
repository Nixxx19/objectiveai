use crate::path_ref::PathRef;

/// Agent reference — a [`PathRef`] that resolves to an agent-specific type.
///
/// Parsed from `key=value,key=value` format. See [`PathRef`] for supported keys.
#[derive(Clone, Debug)]
pub struct AgentRef(pub PathRef);

impl std::str::FromStr for AgentRef {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<PathRef>().map(AgentRef)
    }
}

impl std::fmt::Display for AgentRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AgentRef {
    pub fn resolve(
        self,
    ) -> Result<
        objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional,
        crate::error::Error,
    > {
        Ok(
            objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::Remote(
                self.0.resolve()?,
            ),
        )
    }
}
