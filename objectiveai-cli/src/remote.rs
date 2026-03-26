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
    pub remote: Option<Remote>,
    /// Owner
    #[arg(long)]
    pub owner: Option<String>,
    /// Repository
    #[arg(long)]
    pub repository: Option<String>,
    /// Commit (optional)
    #[arg(long)]
    pub commit: Option<String>,
}

impl RemotePathCommitOptional {
    /// Converts to SDK type. Returns None if remote/owner/repository are not set.
    pub fn into_path(self) -> Option<objectiveai::RemotePathCommitOptional> {
        match (self.remote, self.owner, self.repository) {
            (Some(remote), Some(owner), Some(repository)) => {
                Some(remote.into_path(owner, repository, self.commit))
            }
            _ => None,
        }
    }
}

/// CLI args for specifying a function-profile pair by remote paths.
#[derive(Args)]
pub struct PairRemotePathCommitOptional {
    /// Function remote source
    #[arg(long, value_enum)]
    pub function_remote: Option<Remote>,
    /// Function owner
    #[arg(long)]
    pub function_owner: Option<String>,
    /// Function repository
    #[arg(long)]
    pub function_repository: Option<String>,
    /// Function commit (optional)
    #[arg(long)]
    pub function_commit: Option<String>,
    /// Profile remote source
    #[arg(long, value_enum)]
    pub profile_remote: Option<Remote>,
    /// Profile owner
    #[arg(long)]
    pub profile_owner: Option<String>,
    /// Profile repository
    #[arg(long)]
    pub profile_repository: Option<String>,
    /// Profile commit (optional)
    #[arg(long)]
    pub profile_commit: Option<String>,
}

impl PairRemotePathCommitOptional {
    /// Converts to SDK types. Returns None if required fields are not set.
    pub fn into_paths(self) -> Option<(objectiveai::RemotePathCommitOptional, objectiveai::RemotePathCommitOptional)> {
        match (self.function_remote, self.function_owner, self.function_repository,
               self.profile_remote, self.profile_owner, self.profile_repository) {
            (Some(fr), Some(fo), Some(fre), Some(pr), Some(po), Some(pre)) => {
                let function = fr.into_path(fo, fre, self.function_commit);
                let profile = pr.into_path(po, pre, self.profile_commit);
                Some((function, profile))
            }
            _ => None,
        }
    }
}
