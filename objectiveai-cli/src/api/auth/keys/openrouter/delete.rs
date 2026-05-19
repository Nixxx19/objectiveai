/// `DELETE /auth/keys/openrouter`
pub async fn handle(cli_config: &crate::Config, handle: &objectiveai_sdk::cli::output::Handle) -> Result<(), crate::error::Error> {
    crate::api::call::call_unary::<(), serde_json::Value>(
        cli_config, handle, reqwest::Method::DELETE, "auth/keys/openrouter", None,
    ).await
}
