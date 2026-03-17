use crate::ctx;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Client {
    pub http_client: reqwest::Client,
    /// Token used for fetching from GitHub (read-only).
    pub fetch_github_token: Option<String>,
    /// Token used for publishing to GitHub (requires repo scope).
    pub publish_github_token: Option<String>,
    pub user_agent: Option<String>,
    pub x_title: Option<String>,
    pub referer: Option<String>,
    pub backoff_current_interval: Duration,
    pub backoff_initial_interval: Duration,
    pub backoff_randomization_factor: f64,
    pub backoff_multiplier: f64,
    pub backoff_max_interval: Duration,
    pub backoff_max_elapsed_time: Duration,
}

impl Client {
    pub fn new(
        http_client: reqwest::Client,
        fetch_github_token: Option<String>,
        publish_github_token: Option<String>,
        user_agent: Option<String>,
        x_title: Option<String>,
        referer: Option<String>,
        backoff_current_interval: Duration,
        backoff_initial_interval: Duration,
        backoff_randomization_factor: f64,
        backoff_multiplier: f64,
        backoff_max_interval: Duration,
        backoff_max_elapsed_time: Duration,
    ) -> Self {
        Self {
            http_client,
            fetch_github_token,
            publish_github_token,
            user_agent,
            x_title,
            referer,
            backoff_current_interval,
            backoff_initial_interval,
            backoff_randomization_factor,
            backoff_multiplier,
            backoff_max_interval,
            backoff_max_elapsed_time,
        }
    }

    fn backoff(&self) -> backoff::ExponentialBackoff {
        backoff::ExponentialBackoff {
            current_interval: self.backoff_current_interval,
            initial_interval: self.backoff_initial_interval,
            randomization_factor: self.backoff_randomization_factor,
            multiplier: self.backoff_multiplier,
            max_interval: self.backoff_max_interval,
            max_elapsed_time: Some(self.backoff_max_elapsed_time),
            ..Default::default()
        }
    }

    /// Resolves the fetch token: per-request header → BYOK → self.fetch_github_token.
    async fn resolve_fetch_token<CTXEXT: ctx::ContextExt>(
        &self,
        ctx: &ctx::Context<CTXEXT>,
    ) -> Option<Arc<String>> {
        if let Some(token) = ctx.github_authorization().await {
            return Some(token);
        }
        self.fetch_github_token
            .as_ref()
            .map(|t| Arc::new(t.clone()))
    }

    /// Resolves the publish token: per-request header → BYOK → self.publish_github_token.
    async fn resolve_publish_token<CTXEXT: ctx::ContextExt>(
        &self,
        ctx: &ctx::Context<CTXEXT>,
    ) -> Result<Arc<String>, super::Error> {
        if let Some(token) = ctx.github_authorization().await {
            return Ok(token);
        }
        self.publish_github_token
            .as_ref()
            .map(|t| Arc::new(t.clone()))
            .ok_or(super::Error::MissingPublishToken)
    }

    /// Adds authorization + standard headers to a request.
    fn request_headers(
        &self,
        mut req: reqwest::RequestBuilder,
        token: Option<&str>,
    ) -> reqwest::RequestBuilder {
        if let Some(token) = token {
            req = req.header(
                reqwest::header::AUTHORIZATION,
                ensure_bearer(token),
            );
        }
        if let Some(user_agent) = &self.user_agent {
            req = req.header("user-agent", user_agent);
        }
        if let Some(x_title) = &self.x_title {
            req = req.header("x-title", x_title);
        }
        if let Some(referer) = &self.referer {
            req = req
                .header("referer", referer)
                .header("http-referer", referer);
        }
        req
    }

    // ── Public fetch methods ───────────────────────────────────────

    /// Fetches the latest commit SHA for a repository.
    pub async fn fetch_latest_commit<CTXEXT: ctx::ContextExt>(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        owner: &str,
        repository: &str,
    ) -> Result<Option<String>, super::Error> {
        #[derive(serde::Deserialize)]
        struct Commit {
            sha: String,
        }
        let token = self.resolve_fetch_token(ctx).await;
        let http_request = self.request_headers(
            self.http_client
                .get(format!(
                    "https://api.github.com/repos/{}/{}/commits",
                    owner, repository,
                ))
                .header("accept", "application/vnd.github+json"),
            token.as_deref(),
        );
        backoff::future::retry(self.backoff(), || async {
            let response = http_request
                .try_clone()
                .unwrap()
                .send()
                .await
                .map_err(super::Error::RequestError)?;
            let code = response.status();
            if code.is_success() {
                let text = response
                    .text()
                    .await
                    .map_err(super::Error::ResponseError)?;
                let mut de = serde_json::Deserializer::from_str(&text);
                match serde_path_to_error::deserialize::<_, Vec<Commit>>(&mut de) {
                    Ok(commits) => Ok(commits.first().map(|c| c.sha.clone())),
                    Err(e) => Err(backoff::Error::transient(
                        super::Error::DeserializationError(e),
                    )),
                }
            } else if code == reqwest::StatusCode::NOT_FOUND {
                Ok(None)
            } else {
                Err(backoff::Error::transient(bad_status(response).await))
            }
        })
        .await
    }

    /// Fetches a JSON file from a GitHub repository and deserializes it.
    pub async fn read_json<T, CTXEXT: ctx::ContextExt>(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        owner: &str,
        repository: &str,
        commit: &str,
        path: &str,
    ) -> Result<Option<T>, super::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let token = self.resolve_fetch_token(ctx).await;
        let token_str = token.as_deref();
        backoff::future::retry(self.backoff(), || async {
            match self.fetch_file_raw::<T>(token_str, owner, repository, commit, path).await {
                Ok(opt) => Ok(opt),
                Err(e1) => match self
                    .fetch_file_api::<T>(token_str, owner, repository, commit, path)
                    .await
                {
                    Ok(opt) => Ok(opt),
                    Err(e2) => Err(backoff::Error::transient(
                        super::Error::MultipleErrors(Box::new(e1), Box::new(e2)),
                    )),
                },
            }
        })
        .await
    }

    // ── Private fetch helpers ──────────────────────────────────────

    async fn fetch_file_raw<T>(
        &self,
        token: Option<&str>,
        owner: &str,
        repository: &str,
        commit: &str,
        path: &str,
    ) -> Result<Option<T>, super::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let http_request = self.request_headers(
            self.http_client.get(format!(
                "https://raw.githubusercontent.com/{}/{}/{}/{}",
                owner, repository, commit, path,
            )),
            token,
        );
        let response = http_request
            .send()
            .await
            .map_err(super::Error::RequestError)?;
        let code = response.status();
        if code.is_success() {
            let text = response.text().await.map_err(super::Error::ResponseError)?;
            let mut de = serde_json::Deserializer::from_str(&text);
            match serde_path_to_error::deserialize::<_, T>(&mut de) {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(super::Error::DeserializationError(e)),
            }
        } else if code == reqwest::StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            Err(bad_status(response).await)
        }
    }

    async fn fetch_file_api<T>(
        &self,
        token: Option<&str>,
        owner: &str,
        repository: &str,
        commit: &str,
        path: &str,
    ) -> Result<Option<T>, super::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let http_request = self.request_headers(
            self.http_client
                .get(format!(
                    "https://api.github.com/repos/{}/{}/contents/{}?ref={}",
                    owner, repository, path, commit,
                ))
                .header("accept", "application/vnd.github+json"),
            token,
        );
        let response = http_request
            .send()
            .await
            .map_err(super::Error::RequestError)?;
        let code = response.status();
        if code.is_success() {
            let text = response.text().await.map_err(super::Error::ResponseError)?;
            let mut de = serde_json::Deserializer::from_str(&text);
            match serde_path_to_error::deserialize::<_, T>(&mut de) {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(super::Error::DeserializationError(e)),
            }
        } else if code == reqwest::StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            Err(bad_status(response).await)
        }
    }

    // ── Publish methods ────────────────────────────────────────────

    /// Checks whether a GitHub repository exists.
    pub async fn repository_exists<CTXEXT: ctx::ContextExt>(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        owner: &str,
        repository: &str,
    ) -> Result<bool, super::Error> {
        let token = self.resolve_publish_token(ctx).await?;
        let req = self.request_headers(
            self.http_client
                .get(format!("https://api.github.com/repos/{}/{}", owner, repository))
                .header("accept", "application/vnd.github+json"),
            Some(&token),
        );
        backoff::future::retry(self.backoff(), || async {
            let response = req
                .try_clone()
                .unwrap()
                .send()
                .await
                .map_err(|e| backoff::Error::transient(super::Error::RequestError(e)))?;
            let code = response.status();
            if code.is_success() {
                Ok(true)
            } else if code == reqwest::StatusCode::NOT_FOUND {
                Ok(false)
            } else {
                Err(backoff::Error::transient(bad_status(response).await))
            }
        })
        .await
    }

    /// Validates that a GitHub token is valid. Returns the scopes.
    pub async fn validate_token<CTXEXT: ctx::ContextExt>(
        &self,
        ctx: &ctx::Context<CTXEXT>,
    ) -> Result<Vec<String>, super::Error> {
        let token = self.resolve_publish_token(ctx).await?;
        let req = self.request_headers(
            self.http_client
                .get("https://api.github.com/user")
                .header("accept", "application/vnd.github+json"),
            Some(&token),
        );
        backoff::future::retry(self.backoff(), || async {
            let response = req
                .try_clone()
                .unwrap()
                .send()
                .await
                .map_err(|e| backoff::Error::transient(super::Error::RequestError(e)))?;
            let code = response.status();
            if code.is_success() {
                let scopes = response
                    .headers()
                    .get("x-oauth-scopes")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                Ok(scopes)
            } else {
                Err(backoff::Error::permanent(bad_status(response).await))
            }
        })
        .await
    }

    /// Returns the authenticated user's login name.
    pub async fn get_authenticated_user<CTXEXT: ctx::ContextExt>(
        &self,
        ctx: &ctx::Context<CTXEXT>,
    ) -> Result<String, super::Error> {
        let token = self.resolve_publish_token(ctx).await?;
        let req = self.request_headers(
            self.http_client
                .get("https://api.github.com/user")
                .header("accept", "application/vnd.github+json"),
            Some(&token),
        );
        backoff::future::retry(self.backoff(), || async {
            let response = req
                .try_clone()
                .unwrap()
                .send()
                .await
                .map_err(|e| backoff::Error::transient(super::Error::RequestError(e)))?;
            let code = response.status();
            if code.is_success() {
                let body: serde_json::Value = response
                    .json()
                    .await
                    .map_err(|e| backoff::Error::transient(super::Error::ResponseError(e)))?;
                let login = body["login"]
                    .as_str()
                    .ok_or_else(|| {
                        backoff::Error::permanent(super::Error::BadStatus {
                            code,
                            body: body.clone(),
                        })
                    })?
                    .to_string();
                Ok(login)
            } else {
                Err(backoff::Error::permanent(bad_status(response).await))
            }
        })
        .await
    }

    /// Creates a new GitHub repository under the authenticated user.
    pub async fn create_repository<CTXEXT: ctx::ContextExt>(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        name: &str,
        description: &str,
    ) -> Result<String, super::Error> {
        let token = self.resolve_publish_token(ctx).await?;
        let req = self.request_headers(
            self.http_client
                .post("https://api.github.com/user/repos")
                .header("accept", "application/vnd.github+json")
                .json(&serde_json::json!({
                    "name": name,
                    "description": description,
                    "auto_init": false,
                })),
            Some(&token),
        );
        backoff::future::retry(self.backoff(), || async {
            let response = req
                .try_clone()
                .unwrap()
                .send()
                .await
                .map_err(|e| backoff::Error::transient(super::Error::RequestError(e)))?;
            let code = response.status();
            if code.is_success() {
                let body: serde_json::Value = response
                    .json()
                    .await
                    .map_err(|e| backoff::Error::transient(super::Error::ResponseError(e)))?;
                Ok(body["clone_url"].as_str().unwrap_or("").to_string())
            } else {
                Err(backoff::Error::transient(bad_status(response).await))
            }
        })
        .await
    }

    /// Updates the description of a GitHub repository.
    pub async fn update_description<CTXEXT: ctx::ContextExt>(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        owner: &str,
        repository: &str,
        description: &str,
    ) -> Result<(), super::Error> {
        let token = self.resolve_publish_token(ctx).await?;
        let req = self.request_headers(
            self.http_client
                .patch(format!("https://api.github.com/repos/{}/{}", owner, repository))
                .header("accept", "application/vnd.github+json")
                .json(&serde_json::json!({ "description": description })),
            Some(&token),
        );
        backoff::future::retry(self.backoff(), || async {
            let response = req
                .try_clone()
                .unwrap()
                .send()
                .await
                .map_err(|e| backoff::Error::transient(super::Error::RequestError(e)))?;
            let code = response.status();
            if code.is_success() {
                Ok(())
            } else {
                Err(backoff::Error::transient(bad_status(response).await))
            }
        })
        .await
    }

    /// Publishes files to a GitHub repository.
    pub async fn publish<CTXEXT: ctx::ContextExt + Send + Sync>(
        &self,
        filesystem_client: &crate::filesystem::Client,
        ctx: &ctx::Context<CTXEXT>,
        name: &str,
        description: &str,
        files: &[(&str, &str)],
    ) -> Result<objectiveai::RemotePath, super::Error> {
        let (owner, repo) = if let Some((o, r)) = name.split_once('/') {
            (o.to_string(), r.to_string())
        } else {
            let user = self.get_authenticated_user(ctx).await?;
            (user, name.to_string())
        };

        let exists = self.repository_exists(ctx, &owner, &repo).await?;
        if !exists {
            self.create_repository(ctx, &repo, description).await?;
        }

        let token = self.resolve_publish_token(ctx).await?;
        let bare_token = strip_bearer(&token).to_string();
        let remote_url = format!("https://github.com/{}/{}.git", owner, repo);
        let commit_message = format!("Publish {}", name);

        let fs = filesystem_client.clone();
        let owner_clone = owner.clone();
        let repo_clone = repo.clone();
        let files_owned: Vec<(String, String)> = files
            .iter()
            .map(|(n, c)| (n.to_string(), c.to_string()))
            .collect();

        let commit_sha = tokio::task::spawn_blocking(move || -> Result<String, crate::filesystem::Error> {
            let file_refs: Vec<(&str, &str)> = files_owned
                .iter()
                .map(|(n, c)| (n.as_str(), c.as_str()))
                .collect();
            fs.publish_and_push(
                crate::retrieval::Kind::Functions,
                &owner_clone,
                &repo_clone,
                &file_refs,
                &commit_message,
                &remote_url,
                &bare_token,
            )
        })
        .await
        .map_err(super::Error::Join)?
        .map_err(super::Error::Filesystem)?;

        let _ = self.update_description(ctx, &owner, &repo, description).await;

        Ok(objectiveai::RemotePath {
            remote: objectiveai::Remote::Github,
            owner,
            repository: repo,
            commit: commit_sha,
        })
    }
}

/// Extracts a bad status error from a response.
async fn bad_status(response: reqwest::Response) -> super::Error {
    let code = response.status();
    match response.text().await {
        Ok(text) => super::Error::BadStatus {
            code,
            body: serde_json::from_str::<serde_json::Value>(&text)
                .unwrap_or(serde_json::Value::String(text)),
        },
        Err(_) => super::Error::BadStatus {
            code,
            body: serde_json::Value::Null,
        },
    }
}

/// Ensures a token has the "Bearer " prefix.
fn ensure_bearer(token: &str) -> String {
    if token.starts_with("Bearer ") {
        token.to_string()
    } else {
        format!("Bearer {}", token)
    }
}

/// Strips the "Bearer " prefix from a token if present.
fn strip_bearer(token: &str) -> &str {
    token.strip_prefix("Bearer ").unwrap_or(token)
}
