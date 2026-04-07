use clap::Args;

/// CLI args for getting a resource by path or favorite name.
#[derive(Args)]
pub struct GetArgs {
    /// Resource path (e.g. favorite=name or remote=github,owner=x,repository=y)
    #[arg(long)]
    pub path: crate::favorite_ref::FavoriteRef,
}

impl GetArgs {
    pub fn resolve(
        self,
        get_favorites: impl FnOnce() -> Vec<objectiveai::config::Favorite>,
    ) -> Result<objectiveai::RemotePathCommitOptional, crate::error::Error> {
        self.path.resolve(get_favorites)
    }
}

/// CLI args for getting a function-profile pair by remote paths or favorite name.
#[derive(Args)]
pub struct GetPairArgs {
    /// Pair favorite name (mutually exclusive with --function/--profile)
    #[arg(long, conflicts_with_all = ["function", "profile"])]
    pub favorite: Option<String>,
    /// Function path (e.g. remote=github,owner=x,repository=y)
    #[arg(long)]
    pub function: Option<crate::path_ref::PathRef>,
    /// Profile path (e.g. remote=github,owner=x,repository=y)
    #[arg(long)]
    pub profile: Option<crate::path_ref::PathRef>,
}

impl GetPairArgs {
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
            let function = self.function
                .ok_or(crate::error::Error::MissingArgs("--function is required (or use --favorite)"))?
                .resolve()?;
            let profile = self.profile
                .ok_or(crate::error::Error::MissingArgs("--profile is required (or use --favorite)"))?
                .resolve()?;
            Ok((function, profile))
        }
    }
}
