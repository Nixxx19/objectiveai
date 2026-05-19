/// `GET /auth/credits`
pub async fn handle(cli_config: &crate::Config, handle: &objectiveai_sdk::cli::output::Handle) -> Result<(), crate::error::Error> {
    crate::api::call::call_unary::<(), serde_json::Value>(
        cli_config, handle, reqwest::Method::GET, "auth/credits", None,
    ).await
}
