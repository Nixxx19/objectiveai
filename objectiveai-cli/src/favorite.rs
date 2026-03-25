use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum AddFavorite {
    /// GitHub repository
    Github {
        #[arg(long)]
        owner: String,
        #[arg(long)]
        repository: String,
        #[arg(long)]
        commit: Option<String>,
        #[arg(long)]
        note: String,
    },
    /// Local filesystem repository
    Filesystem {
        #[arg(long)]
        owner: String,
        #[arg(long)]
        repository: String,
        #[arg(long)]
        commit: Option<String>,
        #[arg(long)]
        note: String,
    },
}

impl AddFavorite {
    pub fn apply(self, favorites: &mut Vec<objectiveai::config::Favorite>) {
        favorites.push(self.into());
    }
}

impl From<AddFavorite> for objectiveai::config::Favorite {
    fn from(add: AddFavorite) -> Self {
        match add {
            AddFavorite::Github { owner, repository, commit, note } => Self {
                path: objectiveai::RemotePathCommitOptional::Github { owner, repository, commit },
                note,
            },
            AddFavorite::Filesystem { owner, repository, commit, note } => Self {
                path: objectiveai::RemotePathCommitOptional::Filesystem { owner, repository, commit },
                note,
            },
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
