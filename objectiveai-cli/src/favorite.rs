use clap::Args;

#[derive(Args)]
pub struct AddFavorite {
    /// Remote source
    #[arg(long, value_enum)]
    pub remote: crate::remote::Remote,
    /// Owner
    #[arg(long)]
    pub owner: String,
    /// Repository
    #[arg(long)]
    pub repository: String,
    /// Commit (optional)
    #[arg(long)]
    pub commit: Option<String>,
    /// Note
    #[arg(long)]
    pub note: String,
}

impl AddFavorite {
    pub fn apply(self, favorites: &mut Vec<objectiveai::config::Favorite>) {
        favorites.push(self.into());
    }
}

impl From<AddFavorite> for objectiveai::config::Favorite {
    fn from(add: AddFavorite) -> Self {
        Self {
            path: add.remote.into_path(add.owner, add.repository, add.commit),
            note: add.note,
        }
    }
}

#[derive(Args)]
pub struct EditFavorite {
    /// Index of the favorite to edit
    pub index: usize,
    /// Set the note
    #[arg(long)]
    pub note: Option<String>,
    /// Set the commit
    #[arg(long, conflicts_with = "remove_commit")]
    pub commit: Option<String>,
    /// Remove the commit
    #[arg(long, conflicts_with = "commit")]
    pub remove_commit: bool,
}

impl EditFavorite {
    pub fn apply(self, favorite: &mut objectiveai::config::Favorite) {
        if let Some(note) = self.note {
            favorite.note = note;
        }
        if let Some(commit) = self.commit {
            match &mut favorite.path {
                objectiveai::RemotePathCommitOptional::Github { commit: c, .. } => *c = Some(commit),
                objectiveai::RemotePathCommitOptional::Filesystem { commit: c, .. } => *c = Some(commit),
                objectiveai::RemotePathCommitOptional::Mock { .. } => {}
            }
        } else if self.remove_commit {
            match &mut favorite.path {
                objectiveai::RemotePathCommitOptional::Github { commit, .. } => *commit = None,
                objectiveai::RemotePathCommitOptional::Filesystem { commit, .. } => *commit = None,
                objectiveai::RemotePathCommitOptional::Mock { .. } => {}
            }
        }
    }
}

#[derive(Args)]
pub struct AddPairFavorite {
    /// Function remote source
    #[arg(long, value_enum)]
    pub function_remote: crate::remote::Remote,
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
    pub profile_remote: crate::remote::Remote,
    /// Profile owner
    #[arg(long)]
    pub profile_owner: String,
    /// Profile repository
    #[arg(long)]
    pub profile_repository: String,
    /// Profile commit (optional)
    #[arg(long)]
    pub profile_commit: Option<String>,
    /// Note for this favorite
    #[arg(long)]
    pub note: String,
}

impl From<AddPairFavorite> for objectiveai::config::PairFavorite {
    fn from(add: AddPairFavorite) -> Self {
        Self {
            function: add.function_remote.into_path(add.function_owner, add.function_repository, add.function_commit),
            profile: add.profile_remote.into_path(add.profile_owner, add.profile_repository, add.profile_commit),
            note: add.note,
        }
    }
}

#[derive(Args)]
pub struct EditPairFavorite {
    /// Index of the favorite to edit
    pub index: usize,
    /// Set the note
    #[arg(long)]
    pub note: Option<String>,
    /// Set the function commit
    #[arg(long, conflicts_with = "remove_function_commit")]
    pub function_commit: Option<String>,
    /// Remove the function commit
    #[arg(long, conflicts_with = "function_commit")]
    pub remove_function_commit: bool,
    /// Set the profile commit
    #[arg(long, conflicts_with = "remove_profile_commit")]
    pub profile_commit: Option<String>,
    /// Remove the profile commit
    #[arg(long, conflicts_with = "profile_commit")]
    pub remove_profile_commit: bool,
}

impl EditPairFavorite {
    pub fn apply(self, favorite: &mut objectiveai::config::PairFavorite) {
        if let Some(note) = self.note {
            favorite.note = note;
        }
        if let Some(commit) = self.function_commit {
            match &mut favorite.function {
                objectiveai::RemotePathCommitOptional::Github { commit: c, .. } => *c = Some(commit),
                objectiveai::RemotePathCommitOptional::Filesystem { commit: c, .. } => *c = Some(commit),
                objectiveai::RemotePathCommitOptional::Mock { .. } => {}
            }
        } else if self.remove_function_commit {
            match &mut favorite.function {
                objectiveai::RemotePathCommitOptional::Github { commit, .. } => *commit = None,
                objectiveai::RemotePathCommitOptional::Filesystem { commit, .. } => *commit = None,
                objectiveai::RemotePathCommitOptional::Mock { .. } => {}
            }
        }
        if let Some(commit) = self.profile_commit {
            match &mut favorite.profile {
                objectiveai::RemotePathCommitOptional::Github { commit: c, .. } => *c = Some(commit),
                objectiveai::RemotePathCommitOptional::Filesystem { commit: c, .. } => *c = Some(commit),
                objectiveai::RemotePathCommitOptional::Mock { .. } => {}
            }
        } else if self.remove_profile_commit {
            match &mut favorite.profile {
                objectiveai::RemotePathCommitOptional::Github { commit, .. } => *commit = None,
                objectiveai::RemotePathCommitOptional::Filesystem { commit, .. } => *commit = None,
                objectiveai::RemotePathCommitOptional::Mock { .. } => {}
            }
        }
    }
}
