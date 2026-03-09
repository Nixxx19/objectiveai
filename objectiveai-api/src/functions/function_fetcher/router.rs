//! Router that dispatches to GitHub or Filesystem fetchers based on Remote.

use crate::ctx;
use std::collections::HashMap;
use std::sync::Arc;

/// Routes Function fetch requests to the appropriate sub-fetcher based on [`Remote`].
///
/// [`Remote`]: objectiveai::functions::Remote
pub struct FetcherRouter<G, F, M> {
    /// GitHub sub-fetcher.
    pub github: Arc<G>,
    /// Filesystem sub-fetcher.
    pub filesystem: Arc<F>,
    /// Mock sub-fetcher.
    pub mock: Arc<M>,
}

impl<G, F, M> FetcherRouter<G, F, M> {
    /// Creates a new FetcherRouter with GitHub, Filesystem, and Mock sub-fetchers.
    pub fn new(github: Arc<G>, filesystem: Arc<F>, mock: Arc<M>) -> Self {
        Self { github, filesystem, mock }
    }
}

impl<G, F, M> FetcherRouter<G, F, M> {
    /// Dispatches a Function fetch to the appropriate sub-fetcher based on the remote.
    ///
    /// Alpha function types are transpiled to standard function types before returning.
    pub async fn fetch<CTXEXT>(
        &self,
        ctx: ctx::Context<CTXEXT>,
        remote: objectiveai::functions::Remote,
        owner: &str,
        repository: &str,
        commit: Option<&str>,
    ) -> Result<
        Option<objectiveai::functions::response::GetFunction>,
        objectiveai::error::ResponseError,
    >
    where
        CTXEXT: Send + Sync + 'static,
        G: super::Fetcher<CTXEXT> + Send + Sync + 'static,
        F: super::Fetcher<CTXEXT> + Send + Sync + 'static,
        M: super::Fetcher<CTXEXT> + Send + Sync + 'static,
    {
        let full = match remote {
            objectiveai::functions::Remote::Github => {
                self.github.fetch(ctx, owner, repository, commit).await?
            }
            objectiveai::functions::Remote::Filesystem => {
                self.filesystem
                    .fetch(ctx, owner, repository, commit)
                    .await?
            }
            objectiveai::functions::Remote::Mock => {
                self.mock.fetch(ctx, owner, repository, commit).await?
            }
        };
        Ok(full.map(|f| f.transpile()))
    }

    /// Recursively fetches all child functions referenced by the given function's
    /// tasks. Returns a HashMap keyed by `"{owner}/{repository}/{commit}"` containing
    /// each child's transpiled `RemoteFunction`.
    ///
    /// Each child is fetched and immediately recurses into its own children
    /// concurrently — no waiting for siblings to complete first.
    pub fn fetch_recursive<'a, CTXEXT>(
        &'a self,
        ctx: &'a ctx::Context<CTXEXT>,
        function: &'a objectiveai::functions::RemoteFunction,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<
                        HashMap<String, objectiveai::functions::RemoteFunction>,
                        objectiveai::error::ResponseError,
                    >,
                > + Send
                + 'a,
        >,
    >
    where
        CTXEXT: Send + Sync + 'static,
        G: super::Fetcher<CTXEXT> + Send + Sync + 'static,
        F: super::Fetcher<CTXEXT> + Send + Sync + 'static,
        M: super::Fetcher<CTXEXT> + Send + Sync + 'static,
    {
        Box::pin(async move {
            let refs = child_refs(function);
            if refs.is_empty() {
                return Ok(HashMap::new());
            }

            // Fetch each child + its descendants concurrently.
            let futures: Vec<_> = refs
                .into_iter()
                .map(|r| async move {
                    let get_fn = self
                        .fetch(
                            ctx.clone(),
                            r.remote,
                            &r.owner,
                            &r.repository,
                            Some(&r.commit),
                        )
                        .await?;
                    match get_fn {
                        Some(get_fn) => {
                            // Recurse immediately — don't wait for siblings.
                            let mut descendants =
                                self.fetch_recursive(ctx, &get_fn.inner).await?;
                            descendants.insert(r.key(), get_fn.inner);
                            Ok(descendants)
                        }
                        None => Ok(HashMap::new()),
                    }
                })
                .collect();

            let results: Vec<
                Result<
                    HashMap<String, objectiveai::functions::RemoteFunction>,
                    objectiveai::error::ResponseError,
                >,
            > = futures::future::join_all(futures).await;
            let mut merged = HashMap::new();
            for result in results {
                merged.extend(result?);
            }
            Ok(merged)
        })
    }
}

/// A reference to a child function extracted from a task.
struct ChildRef {
    remote: objectiveai::functions::Remote,
    owner: String,
    repository: String,
    commit: String,
}

impl ChildRef {
    fn key(&self) -> String {
        format!("{}/{}/{}", self.owner, self.repository, self.commit)
    }
}

/// Extracts child function references from a function's tasks.
fn child_refs(
    function: &objectiveai::functions::RemoteFunction,
) -> Vec<ChildRef> {
    let tasks = match function {
        objectiveai::functions::RemoteFunction::Scalar { tasks, .. } => tasks,
        objectiveai::functions::RemoteFunction::Vector { tasks, .. } => tasks,
    };
    let mut refs = Vec::new();
    for task in tasks {
        match task {
            objectiveai::functions::TaskExpression::ScalarFunction(t) => {
                refs.push(ChildRef {
                    remote: t.remote.clone(),
                    owner: t.owner.clone(),
                    repository: t.repository.clone(),
                    commit: t.commit.clone(),
                });
            }
            objectiveai::functions::TaskExpression::VectorFunction(t) => {
                refs.push(ChildRef {
                    remote: t.remote.clone(),
                    owner: t.owner.clone(),
                    repository: t.repository.clone(),
                    commit: t.commit.clone(),
                });
            }
            _ => {}
        }
    }
    refs
}
