use clap::Args;

#[derive(Clone, clap::ValueEnum)]
pub enum Remote {
    Github,
    Filesystem,
}

impl Remote {
    pub fn into_path(self, owner: String, repository: String, commit: Option<String>) -> objectiveai::RemotePathCommitOptional {
        match self {
            Remote::Github => objectiveai::RemotePathCommitOptional::Github { owner, repository, commit },
            Remote::Filesystem => objectiveai::RemotePathCommitOptional::Filesystem { owner, repository, commit },
        }
    }
}

/// CLI args for specifying a remote path with an optional commit.
#[derive(Args)]
pub struct RemotePathCommitOptional {
    /// Remote source
    #[arg(long, value_enum)]
    pub remote: Remote,
    /// Owner
    #[arg(long)]
    pub owner: String,
    /// Repository
    #[arg(long)]
    pub repository: String,
    /// Commit (optional)
    #[arg(long)]
    pub commit: Option<String>,
}

impl From<RemotePathCommitOptional> for objectiveai::RemotePathCommitOptional {
    fn from(path: RemotePathCommitOptional) -> Self {
        path.remote.into_path(path.owner, path.repository, path.commit)
    }
}

/// CLI args for specifying a function-profile pair by remote paths.
#[derive(Args)]
pub struct PairRemotePathCommitOptional {
    /// Function remote source
    #[arg(long, value_enum)]
    pub function_remote: Remote,
    /// Function owner
    #[arg(long)]
    pub function_owner: String,
    /// Function repository
    #[arg(long)]
    pub function_repository: String,
    /// Function commit (optional)
    #[arg(long)]
    pub function_commit: Option<String>,
    /// Profile remote source
    #[arg(long, value_enum)]
    pub profile_remote: Remote,
    /// Profile owner
    #[arg(long)]
    pub profile_owner: String,
    /// Profile repository
    #[arg(long)]
    pub profile_repository: String,
    /// Profile commit (optional)
    #[arg(long)]
    pub profile_commit: Option<String>,
}

impl PairRemotePathCommitOptional {
    pub fn into_paths(self) -> (objectiveai::RemotePathCommitOptional, objectiveai::RemotePathCommitOptional) {
        let function = self.function_remote.into_path(self.function_owner, self.function_repository, self.function_commit);
        let profile = self.profile_remote.into_path(self.profile_owner, self.profile_repository, self.profile_commit);
        (function, profile)
    }
}
