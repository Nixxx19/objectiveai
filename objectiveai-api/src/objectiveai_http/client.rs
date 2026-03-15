//! Wrapper around [`objectiveai::HttpClient`] that injects per-request
//! authorization headers from the request context.

use crate::ctx;

/// ObjectiveAI HTTP client that injects per-request authorization from context.
///
/// Stores the same configuration as [`objectiveai::HttpClient`] minus the
/// three authorization fields, which are populated per-request via
/// [`with_authorization`](Self::with_authorization).
#[derive(Debug, Clone)]
pub struct Client {
    pub http_client: reqwest::Client,
    pub api_base: String,
    pub api_key: Option<String>,
    pub user_agent: Option<String>,
    pub x_title: Option<String>,
    pub referer: Option<String>,
}

impl Client {
    pub fn new(
        http_client: reqwest::Client,
        api_base: Option<impl Into<String>>,
        api_key: Option<impl Into<String>>,
        user_agent: Option<impl Into<String>>,
        x_title: Option<impl Into<String>>,
        referer: Option<impl Into<String>>,
    ) -> Self {
        Self {
            http_client,
            api_base: match api_base {
                Some(base) => base.into(),
                None => "https://api.objective-ai.io".to_string(),
            },
            api_key: api_key.map(Into::into),
            user_agent: user_agent.map(Into::into),
            x_title: x_title.map(Into::into),
            referer: referer.map(Into::into),
        }
    }

    /// Creates an [`objectiveai::HttpClient`] with authorization headers
    /// populated from the request context.
    pub async fn with_authorization<CTXEXT: ctx::ContextExt>(
        &self,
        ctx: &ctx::Context<CTXEXT>,
    ) -> objectiveai::HttpClient {
        objectiveai::HttpClient {
            http_client: self.http_client.clone(),
            api_base: self.api_base.clone(),
            api_key: self.api_key.clone(),
            user_agent: self.user_agent.clone(),
            x_title: self.x_title.clone(),
            referer: self.referer.clone(),
            x_github_authorization: ctx.github_authorization().await,
            x_openrouter_authorization: ctx.get_upstream_byok(objectiveai::agent::Upstream::Openrouter).await,
            x_mcp_authorization: ctx.mcp_authorization().await,
        }
    }
}
