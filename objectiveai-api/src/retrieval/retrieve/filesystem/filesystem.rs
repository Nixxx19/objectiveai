//! Filesystem fetch source implementation.

use crate::ctx;
use objectiveai::error::ResponseError;
use std::sync::Arc;

pub struct FilesystemClient {
    pub client: Arc<crate::filesystem::Client>,
}

impl FilesystemClient {
    pub fn new(client: Arc<crate::filesystem::Client>) -> Self {
        Self { client }
    }
}

/// Extracts (owner, repository, commit) from a RemotePath.
/// Panics on Mock variant (mock paths should not reach the filesystem client).
fn fs_fields(path: &objectiveai::RemotePath) -> (&str, &str, &str) {
    match path {
        objectiveai::RemotePath::Github { owner, repository, commit }
        | objectiveai::RemotePath::Filesystem { owner, repository, commit } => {
            (owner, repository, commit)
        }
        objectiveai::RemotePath::Mock { .. } => {
            unreachable!("mock paths should not reach the filesystem client")
        }
    }
}

#[async_trait::async_trait]
impl<CTXEXT> super::super::Client<CTXEXT> for FilesystemClient
where
    CTXEXT: Send + Sync + 'static,
{
    async fn get_agent<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        _ctx: &ctx::Context<CTXEXT, PC>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::agent::RemoteAgentBaseWithFallbacks>, ResponseError> {
        let (owner, repository, commit) = fs_fields(path);
        match self
            .client
            .read_json::<objectiveai::agent::RemoteAgentBaseWithFallbacks>(
                crate::retrieval::Kind::Agents,
                owner,
                repository,
                Some(commit),
                "agent.json",
            )
            .await
        {
            Ok(Some((agent, _resolved_commit))) => Ok(Some(agent)),
            Ok(None) => Ok(None),
            Err(e) => Err(ResponseError::from(&e)),
        }
    }

    async fn get_swarm<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        _ctx: &ctx::Context<CTXEXT, PC>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::swarm::RemoteSwarmBase>, ResponseError> {
        let (owner, repository, commit) = fs_fields(path);
        match self
            .client
            .read_json::<objectiveai::swarm::RemoteSwarmBase>(
                crate::retrieval::Kind::Swarms,
                owner,
                repository,
                Some(commit),
                "swarm.json",
            )
            .await
        {
            Ok(Some((swarm, _resolved_commit))) => Ok(Some(swarm)),
            Ok(None) => Ok(None),
            Err(e) => Err(ResponseError::from(&e)),
        }
    }

    async fn get_function<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        _ctx: &ctx::Context<CTXEXT, PC>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::functions::FullRemoteFunction>, ResponseError> {
        let (owner, repository, commit) = fs_fields(path);
        match self
            .client
            .read_json::<objectiveai::functions::FullRemoteFunction>(
                crate::retrieval::Kind::Functions,
                owner,
                repository,
                Some(commit),
                "function.json",
            )
            .await
        {
            Ok(Some((function, _resolved_commit))) => Ok(Some(function)),
            Ok(None) => Ok(None),
            Err(e) => Err(ResponseError::from(&e)),
        }
    }

    async fn get_profile<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        _ctx: &ctx::Context<CTXEXT, PC>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::functions::RemoteProfile>, ResponseError> {
        let (owner, repository, commit) = fs_fields(path);
        match self
            .client
            .read_json::<objectiveai::functions::RemoteProfile>(
                crate::retrieval::Kind::Profiles,
                owner,
                repository,
                Some(commit),
                "profile.json",
            )
            .await
        {
            Ok(Some((profile, _resolved_commit))) => Ok(Some(profile)),
            Ok(None) => Ok(None),
            Err(e) => Err(ResponseError::from(&e)),
        }
    }

    async fn resolve_latest<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        _ctx: &ctx::Context<CTXEXT, PC>,
        kind: crate::retrieval::Kind,
        path: &objectiveai::RemotePathCommitOptional,
    ) -> Result<Option<objectiveai::RemotePath>, ResponseError> {
        match path {
            objectiveai::RemotePathCommitOptional::Filesystem { owner, repository, commit } => {
                let resolved_commit = match commit {
                    Some(c) => c.clone(),
                    None => match self.client.resolve_head(kind, owner, repository) {
                        Ok(c) => c,
                        Err(_) => return Ok(None),
                    },
                };
                Ok(Some(objectiveai::RemotePath::Filesystem {
                    owner: owner.clone(),
                    repository: repository.clone(),
                    commit: resolved_commit,
                }))
            }
            _ => Ok(None),
        }
    }
}
