use clap::Args;

/// CLI args for getting a resource by remote path or favorite name.
#[derive(Args)]
pub struct GetArgs {
    /// Get by favorite name (mutually exclusive with --remote)
    #[arg(long, conflicts_with_all = ["remote", "owner", "repository", "commit"])]
    pub favorite: Option<String>,
    #[command(flatten)]
    pub path: crate::remote::RemotePathCommitOptional,
}

impl GetArgs {
    /// Resolves to a RemotePathCommitOptional, either from the favorite or from the path args.
    pub fn resolve(
        self,
        get_favorites: impl FnOnce() -> Vec<objectiveai::config::Favorite>,
    ) -> Result<objectiveai::RemotePathCommitOptional, crate::error::Error> {
        if let Some(name) = self.favorite {
            let favorites = get_favorites();
            let fav = favorites.into_iter().find(|f| f.get_name() == name)
                .ok_or_else(|| crate::error::Error::FavoriteNotFound(name))?;
            Ok(fav.path.clone())
        } else {
            self.path.into_path()
                .ok_or_else(|| crate::error::Error::MissingArgs("--remote, --owner, and --repository are required (or use --favorite)"))
        }
    }
}

/// CLI args for getting a function-profile pair by remote paths or favorite name.
#[derive(Args)]
pub struct GetPairArgs {
    /// Get by pair favorite name (mutually exclusive with remote path args)
    #[arg(long, conflicts_with_all = [
        "function_remote", "function_owner", "function_repository", "function_commit",
        "profile_remote", "profile_owner", "profile_repository", "profile_commit",
    ])]
    pub favorite: Option<String>,
    #[command(flatten)]
    pub paths: crate::remote::PairRemotePathCommitOptional,
}

impl GetPairArgs {
    /// Resolves to a (function, profile) pair of RemotePathCommitOptional.
    pub fn resolve(
        self,
        get_favorites: impl FnOnce() -> Vec<objectiveai::config::PairFavorite>,
    ) -> Result<(objectiveai::RemotePathCommitOptional, objectiveai::RemotePathCommitOptional), crate::error::Error> {
        if let Some(name) = self.favorite {
            let favorites = get_favorites();
            let fav = favorites.into_iter().find(|f| f.get_name() == name)
                .ok_or_else(|| crate::error::Error::FavoriteNotFound(name))?;
            Ok((fav.function.clone(), fav.profile.clone()))
        } else {
            self.paths.into_paths()
                .ok_or_else(|| crate::error::Error::MissingArgs("--function-remote, --function-owner, --function-repository, --profile-remote, --profile-owner, and --profile-repository are required (or use --favorite)"))
        }
    }
}
