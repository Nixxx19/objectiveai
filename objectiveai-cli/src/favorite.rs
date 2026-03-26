use clap::Args;

#[derive(Args)]
pub struct AddFavorite {
    /// Name
    #[arg(long)]
    pub name: String,
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
    pub fn into_favorite(self) -> Result<objectiveai::config::Favorite, objectiveai::config::ConfigError> {
        objectiveai::config::Favorite::new(
            self.name,
            self.remote.into_path(self.owner, self.repository, self.commit),
            self.note,
        )
    }
}

#[derive(Args)]
pub struct EditFavorite {
    /// Name of the favorite to edit
    pub name: String,
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
    pub fn apply(self, favorite: &mut objectiveai::config::Favorite) -> Result<(), objectiveai::config::ConfigError> {
        if let Some(note) = self.note {
            favorite.set_note(note)?;
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
        Ok(())
    }
}

#[derive(Args)]
pub struct AddPairFavorite {
    /// Name
    #[arg(long)]
    pub name: String,
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

impl AddPairFavorite {
    pub fn into_pair_favorite(self) -> Result<objectiveai::config::PairFavorite, objectiveai::config::ConfigError> {
        objectiveai::config::PairFavorite::new(
            self.name,
            self.function_remote.into_path(self.function_owner, self.function_repository, self.function_commit),
            self.profile_remote.into_path(self.profile_owner, self.profile_repository, self.profile_commit),
            self.note,
        )
    }
}

#[derive(Args)]
pub struct EditPairFavorite {
    /// Name of the favorite to edit
    pub name: String,
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
    pub fn apply(self, favorite: &mut objectiveai::config::PairFavorite) -> Result<(), objectiveai::config::ConfigError> {
        if let Some(note) = self.note {
            favorite.set_note(note)?;
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
        Ok(())
    }
}
