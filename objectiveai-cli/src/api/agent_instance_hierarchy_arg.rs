use clap::Args;

/// Optional `--agent-id` flag flattened into every api endpoint
/// leaf's `Args` struct. Sets the request's `X-OBJECTIVEAI-AGENT-INSTANCE-HIERARCHY`
/// header only when `Handle.agent_instance_hierarchy` (env-derived) is `None`;
/// otherwise the env value wins.
#[derive(Args, Debug, Clone)]
pub struct AgentIdArg {
    /// Optional agent ID for the request's `X-OBJECTIVEAI-AGENT-INSTANCE-HIERARCHY`
    /// header. Ignored when `OBJECTIVEAI_AGENT_INSTANCE_HIERARCHY` is set in the
    /// environment — the env value (which also feeds
    /// `Handle.agent_instance_hierarchy`) always wins.
    #[arg(long)]
    pub agent_instance_hierarchy: Option<String>,
}
