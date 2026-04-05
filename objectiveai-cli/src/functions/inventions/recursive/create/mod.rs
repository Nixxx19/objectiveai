use clap::{Args, Subcommand};
use futures::StreamExt;

/// Shared params across all invention state types.
#[derive(Args)]
pub struct InventionParams {
    /// Function name
    #[arg(long)]
    pub name: String,
    /// Specification/prompt for the invention
    #[arg(long)]
    pub spec: String,
    /// Nesting depth (0 for leaf-only)
    #[arg(long, default_value = "0")]
    pub depth: u64,
    /// Minimum branch width
    #[arg(long, default_value = "2")]
    pub min_branch_width: u64,
    /// Maximum branch width
    #[arg(long, default_value = "3")]
    pub max_branch_width: u64,
    /// Minimum leaf width (tasks per leaf)
    #[arg(long, default_value = "1")]
    pub min_leaf_width: u64,
    /// Maximum leaf width (tasks per leaf)
    #[arg(long, default_value = "5")]
    pub max_leaf_width: u64,
}

impl InventionParams {
    fn into_params(self) -> objectiveai::functions::inventions::state::Params {
        objectiveai::functions::inventions::state::Params {
            depth: self.depth,
            min_branch_width: self.min_branch_width,
            max_branch_width: self.max_branch_width,
            min_leaf_width: self.min_leaf_width,
            max_leaf_width: self.max_leaf_width,
            name: self.name,
            spec: self.spec,
        }
    }
}

/// A single invention result item.
#[derive(serde::Serialize)]
pub struct InventionResultItem {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<objectiveai::RemotePath>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<objectiveai::error::ResponseError>,
}

/// Extract the name from an invention State.
fn state_name(state: &objectiveai::functions::inventions::State) -> &str {
    match state {
        objectiveai::functions::inventions::State::AlphaScalarBranch(s) => &s.params.name,
        objectiveai::functions::inventions::State::AlphaScalarLeaf(s) => &s.params.name,
        objectiveai::functions::inventions::State::AlphaVectorBranch(s) => &s.params.name,
        objectiveai::functions::inventions::State::AlphaVectorLeaf(s) => &s.params.name,
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Invent a scalar function
    AlphaScalar {
        #[command(flatten)]
        params: InventionParams,
        /// Agent reference (e.g. favorite=name or remote=github,owner=x,repository=y)
        #[arg(long)]
        agent: crate::agent_ref::AgentRef,
        #[command(flatten)]
        continuation: crate::continuation::ContinuationArgs,
        /// Seed for deterministic mock responses
        #[arg(long)]
        seed: Option<i64>,
    },
    /// Invent a vector function
    AlphaVector {
        #[command(flatten)]
        params: InventionParams,
        /// Agent reference (e.g. favorite=name or remote=github,owner=x,repository=y)
        #[arg(long)]
        agent: crate::agent_ref::AgentRef,
        #[command(flatten)]
        continuation: crate::continuation::ContinuationArgs,
        /// Seed for deterministic mock responses
        #[arg(long)]
        seed: Option<i64>,
    },
    /// Invent from a remote state (previously saved invention state files)
    Remote {
        /// State reference (e.g. remote=mock,name=inv-good-sl or remote=github,owner=x,repository=y)
        #[arg(long)]
        state: crate::path_ref::PathRef,
        /// Agent reference (e.g. favorite=name or remote=github,owner=x,repository=y)
        #[arg(long)]
        agent: crate::agent_ref::AgentRef,
        #[command(flatten)]
        continuation: crate::continuation::ContinuationArgs,
        /// Seed for deterministic mock responses
        #[arg(long)]
        seed: Option<i64>,
    },
}

impl Commands {
    pub async fn handle(self) -> Result<crate::Output, crate::error::Error> {
        let (agent_ref, continuation_args, seed, state) = match self {
            Commands::AlphaScalar { params, agent, continuation, seed } => {
                let p = params.into_params();
                let state = objectiveai::functions::inventions::ParamsStateOrRemoteCommitOptional::Inline(
                    objectiveai::functions::inventions::ParamsState::AlphaScalar(
                        objectiveai::functions::inventions::state::AlphaScalarState { params: p, input_schema: None },
                    ),
                );
                (agent, continuation, seed, state)
            }
            Commands::AlphaVector { params, agent, continuation, seed } => {
                let p = params.into_params();
                let state = objectiveai::functions::inventions::ParamsStateOrRemoteCommitOptional::Inline(
                    objectiveai::functions::inventions::ParamsState::AlphaVector(
                        objectiveai::functions::inventions::state::AlphaVectorState { params: p, input_schema: None },
                    ),
                );
                (agent, continuation, seed, state)
            }
            Commands::Remote { state, agent, continuation, seed } => {
                let remote_path = state.resolve()?;
                let state = objectiveai::functions::inventions::ParamsStateOrRemoteCommitOptional::Remote(remote_path);
                (agent, continuation, seed, state)
            }
        };

        let agent = agent_ref.resolve()?;
        let continuation = continuation_args.resolve()?;

        // Read remote from config
        let (_, mut config) = crate::config::read()?;
        let remote = config.functions().inventions().get_remote();

        let request = objectiveai::functions::inventions::recursive::request::FunctionInventionRecursiveCreateParams {
            remote,
            overwrite: None,
            state,
            provider: None,
            agent,
            seed,
            stream: Some(true),
            max_step_retries: None,
            continuation,
        };

        crate::api::run(|http_client| async move {
            let stream = objectiveai::functions::inventions::recursive::create_function_invention_recursive_streaming(
                &http_client, request,
            ).await?;
            tokio::pin!(stream);

            // Aggregate all chunks
            let mut aggregated: Option<objectiveai::functions::inventions::recursive::response::streaming::FunctionInventionRecursiveChunk> = None;
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                match &mut aggregated {
                    Some(agg) => agg.push(&chunk),
                    None => aggregated = Some(chunk),
                }
            }

            let chunk = aggregated.ok_or(crate::error::Error::EmptyStream)?;

            // Build result: one item per invention that has state
            let results: Vec<InventionResultItem> = chunk.inventions.iter()
                .filter_map(|inv| {
                    let state = inv.inner.state.as_ref()?;
                    Some(InventionResultItem {
                        name: state_name(state).to_string(),
                        path: inv.inner.path.clone(),
                        error: inv.inner.error.clone(),
                    })
                })
                .collect();

            Ok(serde_json::to_string(&results).unwrap())
        }, true).await
    }
}
