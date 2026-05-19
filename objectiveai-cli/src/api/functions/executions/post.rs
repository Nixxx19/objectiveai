use clap::Args as ClapArgs;

/// `POST /functions/executions` — dual-mode.
#[derive(ClapArgs)]
pub struct Args {
    #[command(flatten)]
    pub body: crate::api::body::BodySource,
}

pub async fn handle(args: Args, cli_config: &crate::Config, handle: &objectiveai_sdk::cli::output::Handle) -> Result<(), crate::error::Error> {
    let params: objectiveai_sdk::functions::executions::request::FunctionExecutionCreateParams = args.body.resolve()?;
    if params.stream.unwrap_or(false) {
        crate::api::call::call_streaming::<_, objectiveai_sdk::functions::executions::response::streaming::FunctionExecutionChunk>(
            cli_config, handle, reqwest::Method::POST, "functions/executions", Some(params),
        ).await
    } else {
        crate::api::call::call_unary::<_, serde_json::Value>(
            cli_config, handle, reqwest::Method::POST, "functions/executions", Some(params),
        ).await
    }
}
