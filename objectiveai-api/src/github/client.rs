use crate::{ctx, functions};
use futures::FutureExt;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Client {
    pub http_client: reqwest::Client,
    /// Token used for fetching functions/profiles from GitHub (read-only).
    pub fetch_github_token: Option<String>,
    /// Token used for publishing inventions to GitHub (requires repo scope).
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

    /// Resolves the fetch token: checks the context first (per-request header
    /// → BYOK ext), then falls back to `self.fetch_github_token`.
    /// Returns `None` if neither is available.
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

    /// Resolves the publish token: checks the context first (per-request header
    /// → BYOK ext), then falls back to `self.publish_github_token`.
    /// Returns `Err(MissingPublishToken)` if none is available.
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

    pub async fn fetch_function<CTXEXT>(
        self: Arc<Self>,
        ctx: ctx::Context<CTXEXT>,
        owner: &str,
        repository: &str,
        commit: Option<&str>,
    ) -> Result<
        Option<functions::function_fetcher::FullGetFunction>,
        objectiveai::error::ResponseError,
    >
    where
        CTXEXT: Send + Sync + 'static,
    {
        let commit = if let Some(c) = commit {
            c.to_owned()
        } else {
            match self
                .clone()
                .fetch_latest_commit(ctx.clone(), owner, repository)
                .await?
            {
                Some(sha) => sha,
                None => return Ok(None),
            }
        };
        let shared = ctx
            .function_cache
            .entry((objectiveai::functions::Remote::Github, owner.to_owned(), repository.to_owned(), commit.clone()))
            .or_insert_with(|| {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let client = self.clone();
                let owner = owner.to_owned();
                let repository = repository.to_owned();
                let commit = commit.clone();
                tokio::spawn(async move {
                    let result = client
                        .fetch_function_uncached(&owner, &repository, &commit)
                        .await
                        .map_err(|e| {
                            objectiveai::error::ResponseError::from(&e)
                        });
                    let _ = tx.send(result);
                });
                rx.shared()
            })
            .clone();
        match shared.await.unwrap() {
            Ok(Some(inner)) => {
                Ok(Some(functions::function_fetcher::FullGetFunction {
                    remote: objectiveai::functions::Remote::Github,
                    owner: owner.to_owned(),
                    repository: repository.to_owned(),
                    commit: commit.to_owned(),
                    inner,
                }))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn fetch_function_uncached(
        &self,
        owner: &str,
        repository: &str,
        commit: &str,
    ) -> Result<Option<objectiveai::functions::FullRemoteFunction>, super::Error>
    {
        self.fetch_file::<objectiveai::functions::FullRemoteFunction>(
            owner,
            repository,
            commit,
            "function.json",
        )
        .await
    }

    pub async fn fetch_profile<CTXEXT>(
        self: Arc<Self>,
        ctx: ctx::Context<CTXEXT>,
        owner: &str,
        repository: &str,
        commit: Option<&str>,
    ) -> Result<
        Option<objectiveai::functions::profiles::response::GetProfile>,
        objectiveai::error::ResponseError,
    >
    where
        CTXEXT: Send + Sync + 'static,
    {
        let commit = if let Some(c) = commit {
            c.to_owned()
        } else {
            match self
                .clone()
                .fetch_latest_commit(ctx.clone(), owner, repository)
                .await?
            {
                Some(sha) => sha,
                None => return Ok(None),
            }
        };
        let shared = ctx
            .profile_cache
            .entry((objectiveai::functions::Remote::Github, owner.to_owned(), repository.to_owned(), commit.clone()))
            .or_insert_with(|| {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let client = self.clone();
                let owner = owner.to_owned();
                let repository = repository.to_owned();
                let commit = commit.clone();
                tokio::spawn(async move {
                    let result = client
                        .fetch_profile_uncached(&owner, &repository, &commit)
                        .await
                        .map_err(|e| {
                            objectiveai::error::ResponseError::from(&e)
                        });
                    let _ = tx.send(result);
                });
                rx.shared()
            })
            .clone();
        match shared.await.unwrap() {
            Ok(Some(inner)) => Ok(Some(
                objectiveai::functions::profiles::response::GetProfile {
                    remote: objectiveai::functions::Remote::Github,
                    owner: owner.to_owned(),
                    repository: repository.to_owned(),
                    commit: commit.to_owned(),
                    inner,
                },
            )),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn fetch_profile_uncached(
        &self,
        owner: &str,
        repository: &str,
        commit: &str,
    ) -> Result<Option<objectiveai::functions::RemoteProfile>, super::Error>
    {
        match self
            .fetch_file::<objectiveai::functions::RemoteProfile>(
                owner,
                repository,
                commit,
                "profile.json",
            )
            .await
        {
            Ok(Some(profile)) => {
                let valid = match &profile {
                    objectiveai::functions::RemoteProfile::Tasks(tasks_profile) => {
                        tasks_profile.tasks.iter().all(|t| t.validate_commit_required())
                    }
                    objectiveai::functions::RemoteProfile::Auto(_) => true,
                };
                if !valid {
                    Err(super::Error::ProfileCommitShaRequired)
                } else {
                    Ok(Some(profile))
                }
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn fetch_latest_commit<CTXEXT>(
        self: Arc<Self>,
        ctx: ctx::Context<CTXEXT>,
        owner: &str,
        repository: &str,
    ) -> Result<Option<String>, objectiveai::error::ResponseError>
    where
        CTXEXT: Send + Sync + 'static,
    {
        let shared = ctx
            .latest_commit_cache
            .entry((objectiveai::functions::Remote::Github, owner.to_owned(), repository.to_owned()))
            .or_insert_with(|| {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let owner = owner.to_owned();
                let repository = repository.to_owned();
                tokio::spawn(async move {
                    let result = self
                        .fetch_latest_commit_uncached(&owner, &repository)
                        .await
                        .map_err(|e| {
                            objectiveai::error::ResponseError::from(&e)
                        });
                    let _ = tx.send(result);
                });
                rx.shared()
            })
            .clone();
        shared.await.unwrap()
    }

    async fn fetch_latest_commit_uncached(
        &self,
        owner: &str,
        repository: &str,
    ) -> Result<Option<String>, super::Error> {
        #[derive(serde::Deserialize)]
        struct Commit {
            sha: String,
        }
        let http_request = self.request_headers(
            self.http_client
                .get(format!(
                    "https://api.github.com/repos/{}/{}/commits",
                    owner, repository,
                ))
                .header("accept", "application/vnd.github+json"),
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
                match serde_path_to_error::deserialize::<_, Vec<Commit>>(
                    &mut de,
                ) {
                    Ok(commits) => Ok(commits.first().map(|c| c.sha.clone())),
                    Err(e) => Err(backoff::Error::transient(
                        super::Error::DeserializationError(e),
                    )),
                }
            } else if code == reqwest::StatusCode::NOT_FOUND {
                Ok(None)
            } else {
                match response.text().await {
                    Ok(text) => Err(backoff::Error::transient(
                        super::Error::BadStatus {
                            code,
                            body: match serde_json::from_str::<
                                serde_json::Value,
                            >(&text) {
                                Ok(json) => json,
                                Err(_) => serde_json::Value::String(text),
                            },
                        },
                    )),
                    Err(_) => Err(backoff::Error::transient(
                        super::Error::BadStatus {
                            code,
                            body: serde_json::Value::Null,
                        },
                    )),
                }
            }
        })
        .await
    }

    async fn fetch_file<T>(
        &self,
        owner: &str,
        repository: &str,
        commit: &str,
        path: &str,
    ) -> Result<Option<T>, super::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        backoff::future::retry(self.backoff(), || async {
            match self.fetch_file_raw(owner, repository, commit, path).await {
                Ok(opt) => Ok(opt),
                Err(e1) => match self
                    .fetch_file_api(owner, repository, commit, path)
                    .await
                {
                    Ok(opt) => Ok(opt),
                    Err(e2) => Err(backoff::Error::transient(
                        super::Error::MultipleErrors(
                            Box::new(e1),
                            Box::new(e2),
                        ),
                    )),
                },
            }
        })
        .await
    }

    async fn fetch_file_raw<T>(
        &self,
        owner: &str,
        repository: &str,
        commit: &str,
        path: &str,
    ) -> Result<Option<T>, super::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let http_request = self.request_headers(self.http_client.get(format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            owner, repository, commit, path,
        )));
        let response = http_request
            .send()
            .await
            .map_err(super::Error::RequestError)?;
        let code = response.status();
        if code.is_success() {
            let text =
                response.text().await.map_err(super::Error::ResponseError)?;
            let mut de = serde_json::Deserializer::from_str(&text);
            match serde_path_to_error::deserialize::<_, T>(&mut de) {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(super::Error::DeserializationError(e)),
            }
        } else if code == reqwest::StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            match response.text().await {
                Ok(text) => Err(super::Error::BadStatus {
                    code,
                    body: match serde_json::from_str::<serde_json::Value>(&text)
                    {
                        Ok(json) => json,
                        Err(_) => serde_json::Value::String(text),
                    },
                }),
                Err(_) => Err(super::Error::BadStatus {
                    code,
                    body: serde_json::Value::Null,
                }),
            }
        }
    }

    async fn fetch_file_api<T>(
        &self,
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
        );
        let response = http_request
            .send()
            .await
            .map_err(super::Error::RequestError)?;
        let code = response.status();
        if code.is_success() {
            let text =
                response.text().await.map_err(super::Error::ResponseError)?;
            let mut de = serde_json::Deserializer::from_str(&text);
            match serde_path_to_error::deserialize::<_, T>(&mut de) {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(super::Error::DeserializationError(e)),
            }
        } else if code == reqwest::StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            match response.text().await {
                Ok(text) => Err(super::Error::BadStatus {
                    code,
                    body: match serde_json::from_str::<serde_json::Value>(&text)
                    {
                        Ok(json) => json,
                        Err(_) => serde_json::Value::String(text),
                    },
                }),
                Err(_) => Err(super::Error::BadStatus {
                    code,
                    body: serde_json::Value::Null,
                }),
            }
        }
    }

    /// Checks whether a GitHub repository exists.
    pub async fn repository_exists<CTXEXT: ctx::ContextExt>(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        owner: &str,
        repository: &str,
    ) -> Result<bool, super::Error> {
        let token = self.resolve_publish_token(ctx).await?;
        let bearer = ensure_bearer(&token);
        backoff::future::retry(self.backoff(), || async {
            let response = self
                .http_client
                .get(format!(
                    "https://api.github.com/repos/{}/{}",
                    owner, repository,
                ))
                .header(reqwest::header::AUTHORIZATION, &*bearer)
                .header("accept", "application/vnd.github+json")
                .header("user-agent", "objectiveai")
                .send()
                .await
                .map_err(|e| backoff::Error::transient(super::Error::RequestError(e)))?;
            let code = response.status();
            if code.is_success() {
                Ok(true)
            } else if code == reqwest::StatusCode::NOT_FOUND {
                Ok(false)
            } else {
                match response.text().await {
                    Ok(text) => Err(backoff::Error::transient(super::Error::BadStatus {
                        code,
                        body: serde_json::from_str::<serde_json::Value>(&text)
                            .unwrap_or(serde_json::Value::String(text)),
                    })),
                    Err(_) => Err(backoff::Error::transient(super::Error::BadStatus {
                        code,
                        body: serde_json::Value::Null,
                    })),
                }
            }
        })
        .await
    }

    /// Validates that a GitHub token is valid and has the required permissions
    /// for repository operations (create, push, edit descriptions).
    ///
    /// Returns the scopes the token has. Errors if the token is invalid.
    /// The caller should check for the `repo` scope.
    pub async fn validate_token<CTXEXT: ctx::ContextExt>(
        &self,
        ctx: &ctx::Context<CTXEXT>,
    ) -> Result<Vec<String>, super::Error> {
        let token = self.resolve_publish_token(ctx).await?;
        let bearer = ensure_bearer(&token);
        backoff::future::retry(self.backoff(), || async {
            let response = self
                .http_client
                .get("https://api.github.com/user")
                .header(reqwest::header::AUTHORIZATION, &*bearer)
                .header("accept", "application/vnd.github+json")
                .header("user-agent", "objectiveai")
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
                match response.text().await {
                    Ok(text) => Err(backoff::Error::permanent(super::Error::BadStatus {
                        code,
                        body: serde_json::from_str::<serde_json::Value>(&text)
                            .unwrap_or(serde_json::Value::String(text)),
                    })),
                    Err(_) => Err(backoff::Error::permanent(super::Error::BadStatus {
                        code,
                        body: serde_json::Value::Null,
                    })),
                }
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
        let bearer = ensure_bearer(&token);
        backoff::future::retry(self.backoff(), || async {
            let response = self
                .http_client
                .get("https://api.github.com/user")
                .header(reqwest::header::AUTHORIZATION, &*bearer)
                .header("accept", "application/vnd.github+json")
                .header("user-agent", "objectiveai")
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
                    .ok_or_else(|| backoff::Error::permanent(super::Error::BadStatus {
                        code,
                        body: body.clone(),
                    }))?
                    .to_string();
                Ok(login)
            } else {
                match response.text().await {
                    Ok(text) => Err(backoff::Error::permanent(super::Error::BadStatus {
                        code,
                        body: serde_json::from_str::<serde_json::Value>(&text)
                            .unwrap_or(serde_json::Value::String(text)),
                    })),
                    Err(_) => Err(backoff::Error::permanent(super::Error::BadStatus {
                        code,
                        body: serde_json::Value::Null,
                    })),
                }
            }
        })
        .await
    }

    /// Creates a new GitHub repository under the authenticated user.
    ///
    /// Returns the clone URL (e.g. `https://github.com/owner/repo.git`).
    pub async fn create_repository<CTXEXT: ctx::ContextExt>(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        name: &str,
        description: &str,
    ) -> Result<String, super::Error> {
        let token = self.resolve_publish_token(ctx).await?;
        let bearer = ensure_bearer(&token);
        backoff::future::retry(self.backoff(), || async {
            let response = self
                .http_client
                .post("https://api.github.com/user/repos")
                .header(reqwest::header::AUTHORIZATION, &*bearer)
                .header("accept", "application/vnd.github+json")
                .header("user-agent", "objectiveai")
                .json(&serde_json::json!({
                    "name": name,
                    "description": description,
                    "auto_init": false,
                }))
                .send()
                .await
                .map_err(|e| backoff::Error::transient(super::Error::RequestError(e)))?;
            let code = response.status();
            if code.is_success() {
                let body: serde_json::Value = response
                    .json()
                    .await
                    .map_err(|e| backoff::Error::transient(super::Error::ResponseError(e)))?;
                let clone_url = body["clone_url"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                Ok(clone_url)
            } else {
                match response.text().await {
                    Ok(text) => Err(backoff::Error::transient(super::Error::BadStatus {
                        code,
                        body: serde_json::from_str::<serde_json::Value>(&text)
                            .unwrap_or(serde_json::Value::String(text)),
                    })),
                    Err(_) => Err(backoff::Error::transient(super::Error::BadStatus {
                        code,
                        body: serde_json::Value::Null,
                    })),
                }
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
        let bearer = ensure_bearer(&token);
        backoff::future::retry(self.backoff(), || async {
            let response = self
                .http_client
                .patch(format!(
                    "https://api.github.com/repos/{}/{}",
                    owner, repository,
                ))
                .header(reqwest::header::AUTHORIZATION, &*bearer)
                .header("accept", "application/vnd.github+json")
                .header("user-agent", "objectiveai")
                .json(&serde_json::json!({
                    "description": description,
                }))
                .send()
                .await
                .map_err(|e| backoff::Error::transient(super::Error::RequestError(e)))?;
            let code = response.status();
            if code.is_success() {
                Ok(())
            } else {
                match response.text().await {
                    Ok(text) => Err(backoff::Error::transient(super::Error::BadStatus {
                        code,
                        body: serde_json::from_str::<serde_json::Value>(&text)
                            .unwrap_or(serde_json::Value::String(text)),
                    })),
                    Err(_) => Err(backoff::Error::transient(super::Error::BadStatus {
                        code,
                        body: serde_json::Value::Null,
                    })),
                }
            }
        })
        .await
    }

    /// Publishes files to a GitHub repository.
    ///
    /// Creates the repository if needed, writes files locally, commits, pushes,
    /// and updates the repository description.
    ///
    /// Returns the `RemoteFunctionPath` on success.
    pub async fn publish<CTXEXT: ctx::ContextExt + Send + Sync>(
        &self,
        filesystem_client: &crate::filesystem::Client,
        ctx: &ctx::Context<CTXEXT>,
        name: &str,
        description: &str,
        files: &[(&str, &str)],
    ) -> Result<objectiveai::functions::RemoteFunctionPath, super::Error> {
        // Parse owner/repo or just repo name.
        let (owner, repo) = if let Some((o, r)) = name.split_once('/') {
            (o.to_string(), r.to_string())
        } else {
            let user = self.get_authenticated_user(ctx).await?;
            (user, name.to_string())
        };

        // Create repository if it doesn't exist.
        let exists = self.repository_exists(ctx, &owner, &repo).await?;
        if !exists {
            self.create_repository(ctx, &repo, description).await?;
        }

        // Resolve the token for git2 operations (strip Bearer prefix if present).
        let token = self.resolve_publish_token(ctx).await?;
        let bare_token = strip_bearer(&token).to_string();
        let remote_url = format!("https://github.com/{}/{}.git", owner, repo);
        let commit_message = format!("Publish {}", name);

        let fs = filesystem_client.clone();
        let owner_clone = owner.clone();
        let repo_clone = repo.clone();
        let files_owned: Vec<(String, String)> = files.iter()
            .map(|(n, c)| (n.to_string(), c.to_string()))
            .collect();

        let commit_sha = tokio::task::spawn_blocking(move || -> Result<String, crate::filesystem::Error> {
            let file_refs: Vec<(&str, &str)> = files_owned.iter()
                .map(|(n, c)| (n.as_str(), c.as_str()))
                .collect();
            fs.publish_and_push(
                crate::filesystem::Kind::Functions, &owner_clone, &repo_clone, &file_refs, &commit_message,
                &remote_url, &bare_token,
            )
        })
        .await
        .map_err(super::Error::Join)?
        .map_err(super::Error::Filesystem)?;

        // Update repository description (best-effort).
        let _ = self.update_description(ctx, &owner, &repo, description).await;

        Ok(objectiveai::functions::RemoteFunctionPath {
            remote: objectiveai::functions::Remote::Github,
            owner,
            repository: repo,
            commit: commit_sha,
        })
    }

    fn request_headers(
        &self,
        mut http_request: reqwest::RequestBuilder,
    ) -> reqwest::RequestBuilder {
        if let Some(token) = &self.fetch_github_token {
            http_request = http_request.header(
                reqwest::header::AUTHORIZATION,
                ensure_bearer(token),
            );
        }
        if let Some(user_agent) = &self.user_agent {
            http_request = http_request.header("user-agent", user_agent);
        }
        if let Some(x_title) = &self.x_title {
            http_request = http_request.header("x-title", x_title);
        }
        if let Some(referer) = &self.referer {
            http_request = http_request
                .header("referer", referer)
                .header("http-referer", referer);
        }
        http_request
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
