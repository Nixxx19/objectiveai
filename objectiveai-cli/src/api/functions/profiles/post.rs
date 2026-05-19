use clap::Args as ClapArgs;

/// `POST /functions/profiles`
#[derive(ClapArgs)]
pub struct Args {
    #[command(flatten)]
    pub body: crate::api::body::BodySource,
}

pub async fn handle(args: Args, cli_config: &crate::Config, handle: &objectiveai_sdk::cli::output::Handle) -> Result<(), crate::error::Error> {
    let req: objectiveai_sdk::RemotePathCommitOptional = args.body.resolve()?;
    crate::api::call::call_unary::<_, serde_json::Value>(
        cli_config, handle, reqwest::Method::POST, "functions/profiles", Some(req),
    ).await
}
