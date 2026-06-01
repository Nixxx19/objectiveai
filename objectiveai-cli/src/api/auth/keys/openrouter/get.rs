use clap::Args as ClapArgs;

/// `GET /auth/keys/openrouter`
#[derive(ClapArgs)]
pub struct Args {
    #[command(flatten)]
    pub agent_instance_hierarchy: crate::api::agent_instance_hierarchy_arg::AgentIdArg,
}

pub async fn handle(
    args: Args,
    cli_config: &crate::Config,
    handle: &objectiveai_sdk::cli::output::Handle,
) -> Result<(), crate::error::Error> {
    crate::api::call::call_unary::<(), serde_json::Value>(
        cli_config,
        handle,
        reqwest::Method::GET,
        "auth/keys/openrouter",
        None,
        args.agent_instance_hierarchy.agent_instance_hierarchy,
    )
    .await
}
