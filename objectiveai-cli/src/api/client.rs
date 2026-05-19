//! Builds the SDK `HttpClient` for endpoint subcommands.
//!
//! Reads the local API server's `address` + `port` from the flat config and
//! the auth/viewer/author headers from `api.headers`. Assumes the user has an
//! `objectiveai-api` already listening on the configured (address, port); the
//! auto-spawn lifecycle is handled by the deferred runner rewrite.

use std::sync::Arc;

pub fn build_http_client(
    config: &mut objectiveai_sdk::filesystem::config::Config,
) -> objectiveai_sdk::HttpClient {
    let api = config.api();
    let address = match (api.get_address(), api.get_port()) {
        (Some(addr), Some(port)) => Some(format!("http://{addr}:{port}")),
        (Some(addr), None) => Some(addr.to_string()),
        (None, Some(port)) => Some(format!("http://127.0.0.1:{port}")),
        (None, None) => None,
    };

    let mut http_client = objectiveai_sdk::HttpClient::new(
        reqwest::Client::new(),
        address,
        None::<String>, // authorization
        None::<String>, // user_agent
        None::<String>, // x_title
        None::<String>, // http_referer
        None::<String>, // x_github_authorization
        None::<String>, // x_openrouter_authorization
        None,           // x_mcp_authorization
        None::<String>, // x_viewer_signature
        None::<String>, // x_viewer_address
        None::<String>, // x_commit_author_name
        None::<String>, // x_commit_author_email
    );

    let headers = config.api().headers();
    if http_client.authorization.is_none() {
        http_client.authorization = headers
            .get_x_objectiveai_authorization()
            .map(|s| Arc::new(s.to_string()));
    }
    if http_client.user_agent.is_none() {
        http_client.user_agent = headers.get_user_agent().map(String::from);
    }
    if http_client.x_title.is_none() {
        http_client.x_title = headers.get_x_title().map(String::from);
    }
    if http_client.http_referer.is_none() {
        http_client.http_referer = headers.get_http_referer().map(String::from);
    }
    if http_client.x_github_authorization.is_none() {
        http_client.x_github_authorization = headers
            .get_x_github_authorization()
            .map(|s| Arc::new(s.to_string()));
    }
    if http_client.x_openrouter_authorization.is_none() {
        http_client.x_openrouter_authorization = headers
            .get_x_openrouter_authorization()
            .map(|s| Arc::new(s.to_string()));
    }
    if http_client.x_mcp_authorization.is_none() {
        http_client.x_mcp_authorization = headers
            .get_x_mcp_authorization()
            .map(|m| Arc::new(m.iter().map(|(k, v)| (k.clone(), v.clone())).collect()));
    }
    if http_client.x_viewer_signature.is_none() {
        http_client.x_viewer_signature = headers
            .get_x_viewer_signature()
            .map(|s| Arc::new(s.to_string()));
    }
    if http_client.x_viewer_address.is_none() {
        http_client.x_viewer_address = headers
            .get_x_viewer_address()
            .map(|s| Arc::new(s.to_string()));
    }
    if http_client.x_commit_author_name.is_none() {
        http_client.x_commit_author_name = headers
            .get_x_commit_author_name()
            .map(|s| Arc::new(s.to_string()));
    }
    if http_client.x_commit_author_email.is_none() {
        http_client.x_commit_author_email = headers
            .get_x_commit_author_email()
            .map(|s| Arc::new(s.to_string()));
    }

    http_client
}
