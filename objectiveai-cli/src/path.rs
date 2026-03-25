use clap::Subcommand;

/// CLI subcommand for specifying a remote path with an optional commit.
#[derive(Subcommand)]
pub enum RemotePathCommitOptional {
    /// GitHub repository
    Github {
        #[arg(long)]
        owner: String,
        #[arg(long)]
        repository: String,
        #[arg(long)]
        commit: Option<String>,
    },
    /// Local filesystem repository
    Filesystem {
        #[arg(long)]
        owner: String,
        #[arg(long)]
        repository: String,
        #[arg(long)]
        commit: Option<String>,
    },
}

impl From<RemotePathCommitOptional> for objectiveai::RemotePathCommitOptional {
    fn from(path: RemotePathCommitOptional) -> Self {
        match path {
            RemotePathCommitOptional::Github { owner, repository, commit } => {
                Self::Github { owner, repository, commit }
            }
            RemotePathCommitOptional::Filesystem { owner, repository, commit } => {
                Self::Filesystem { owner, repository, commit }
            }
        }
    }
}
