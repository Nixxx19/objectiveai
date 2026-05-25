//! `objectiveai-cli-stream` — per-stream subprocess runner.
//!
//! Hello-world skeleton. Owns the full clap argument surface that the
//! existing `objectiveai-cli` resolves from env vars + on-disk config,
//! so any caller can spawn this binary with everything it needs on
//! the command line — no env, no config file.
//!
//! Layout mirrors `objectiveai-cli/src/api/{client,conduit}.rs`:
//!   `--api-address` is required (it's the only thing every endpoint
//!   needs); every per-request header is optional; `--mcp-address`
//!   is optional and will feed the MCP conduit handler once endpoint
//!   dispatch lands. The conduit type itself lives in `objectiveai-cli`
//!   today and will be moved out before this crate consumes it.
//!
//! The `Args::build_http_client` method exists so the next step
//! (endpoint dispatch + body input) is a pure addition.

use std::collections::HashMap;

use clap::Parser;

/// Per-stream subprocess runner for the ObjectiveAI CLI.
#[derive(Parser, Debug)]
#[command(
    name = "objectiveai-cli-stream",
    version,
    about = "Per-stream subprocess runner for the ObjectiveAI CLI",
)]
struct Args {
    // ── API endpoint ─────────────────────────────────────────────
    /// Full base URL of the ObjectiveAI API to dial (e.g. `http://127.0.0.1:8080`).
    /// Maps to the existing CLI's `OBJECTIVEAI_ADDRESS` env var.
    #[arg(long)]
    api_address: String,

    // ── Per-request headers (all optional) ───────────────────────
    /// `Authorization` header. Maps to `OBJECTIVEAI_AUTHORIZATION`.
    #[arg(long)]
    objectiveai_authorization: Option<String>,

    /// `User-Agent` header. Maps to `USER_AGENT`.
    #[arg(long)]
    user_agent: Option<String>,

    /// `X-Title` header. Maps to `X_TITLE`.
    #[arg(long)]
    x_title: Option<String>,

    /// `HTTP-Referer` header. Maps to `HTTP_REFERER`.
    #[arg(long)]
    http_referer: Option<String>,

    /// `X-GITHUB-AUTHORIZATION` header. Maps to `GITHUB_AUTHORIZATION`.
    #[arg(long)]
    github_authorization: Option<String>,

    /// `X-OPENROUTER-AUTHORIZATION` header. Maps to `OPENROUTER_AUTHORIZATION`.
    #[arg(long)]
    openrouter_authorization: Option<String>,

    /// `X-MCP-AUTHORIZATION` header — JSON-encoded `HashMap<String,String>`
    /// e.g. `'{"server.example.com":"bearer-token"}'`. Maps to `MCP_AUTHORIZATION`.
    #[arg(long)]
    mcp_authorization: Option<String>,

    /// `X-VIEWER-SIGNATURE` header. Maps to `VIEWER_SIGNATURE`.
    #[arg(long)]
    viewer_signature: Option<String>,

    /// Viewer base URL (full, e.g. `http://127.0.0.1:8081`).
    /// Maps to `VIEWER_ADDRESS`.
    #[arg(long)]
    viewer_address: Option<String>,

    /// `X-COMMIT-AUTHOR-NAME` header. Maps to `COMMIT_AUTHOR_NAME`.
    #[arg(long)]
    commit_author_name: Option<String>,

    /// `X-COMMIT-AUTHOR-EMAIL` header. Maps to `COMMIT_AUTHOR_EMAIL`.
    #[arg(long)]
    commit_author_email: Option<String>,

    /// `X-OBJECTIVEAI-AGENT-ID` header. Maps to `OBJECTIVEAI_AGENT_ID`.
    #[arg(long)]
    objectiveai_agent_id: Option<String>,

    // ── MCP conduit (optional) ───────────────────────────────────
    /// Full base URL of the local `objectiveai-mcp` server the conduit
    /// dials when the API forwards an inbound `server_request`. When
    /// unset, the conduit 501s every inbound request (matches today's
    /// behavior when neither env var nor config provides one).
    /// Maps to `OBJECTIVEAI_MCP_ADDRESS`.
    #[arg(long)]
    mcp_address: Option<String>,
}

impl Args {
    /// Build the SDK `HttpClient` from the parsed args. Mirrors
    /// `objectiveai_cli::api::client::build_http_client` 1:1 except that
    /// every value comes from clap instead of env-var-or-config.
    fn build_http_client(&self) -> Result<objectiveai_sdk::HttpClient, String> {
        let x_mcp_authorization: Option<HashMap<String, String>> = self
            .mcp_authorization
            .as_deref()
            .map(|s| {
                serde_json::from_str(s).map_err(|e| {
                    format!("--mcp-authorization is not valid JSON for HashMap<String,String>: {e}")
                })
            })
            .transpose()?;

        Ok(objectiveai_sdk::HttpClient::new(
            reqwest::Client::new(),
            Some(self.api_address.clone()),
            self.objectiveai_authorization.clone(),
            self.user_agent.clone(),
            self.x_title.clone(),
            self.http_referer.clone(),
            self.github_authorization.clone(),
            self.openrouter_authorization.clone(),
            x_mcp_authorization,
            self.viewer_signature.clone(),
            self.viewer_address.clone(),
            self.commit_author_name.clone(),
            self.commit_author_email.clone(),
            self.objectiveai_agent_id.clone(),
        ))
    }

}

#[tokio::main]
async fn main() -> Result<(), String> {
    let args = Args::parse();

    // Construct the HttpClient — proves the wiring end-to-end without
    // yet dispatching to any streaming endpoint. Endpoint selection +
    // body parsing + the MCP conduit handler all land in follow-ups.
    let _http_client = args.build_http_client()?;

    println!(
        "objectiveai-cli-stream: resolved api_address={}, mcp={}, agent_id={}",
        args.api_address,
        args.mcp_address.as_deref().unwrap_or("<none>"),
        args.objectiveai_agent_id.as_deref().unwrap_or("<none>"),
    );
    Ok(())
}
