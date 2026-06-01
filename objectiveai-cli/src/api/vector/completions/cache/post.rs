use clap::Args as ClapArgs;

/// `POST /vector/completions/cache`
#[derive(ClapArgs)]
pub struct Args {
    #[command(flatten)]
    pub body: crate::api::body::BodySource,
    #[command(flatten)]
    pub agent_instance_hierarchy: crate::api::agent_instance_hierarchy_arg::AgentIdArg,
}

pub async fn handle(
    args: Args,
    cli_config: &crate::Config,
    handle: &objectiveai_sdk::cli::output::Handle,
) -> Result<(), crate::error::Error> {
    let req: objectiveai_sdk::vector::completions::cache::request::CacheVoteRequestOwned =
        args.body.resolve()?;
    crate::api::call::call_unary::<_, serde_json::Value>(
        cli_config,
        handle,
        reqwest::Method::POST,
        "vector/completions/cache",
        Some(req),
        args.agent_instance_hierarchy.agent_instance_hierarchy,
    )
    .await
}
