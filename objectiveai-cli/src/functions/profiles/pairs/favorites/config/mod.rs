use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get all pair favorites
    Get,
    /// Add a pair favorite
    Add {
        #[command(flatten)]
        args: crate::favorite::AddPairFavorite,
    },
    /// Delete a pair favorite by index
    Del { index: usize },
    /// Edit a pair favorite
    Edit {
        #[command(flatten)]
        args: crate::favorite::EditPairFavorite,
    },
}

impl Commands {
    pub fn handle(self) -> Result<crate::Output, crate::error::Error> {
        let (client, mut config) = crate::config::read()?;
        match self {
            Commands::Get => Ok(crate::Output::ConfigGet(crate::config::format_value(&config.functions().profiles().pairs().get_favorites()))),
            Commands::Add { args } => {
                config.functions().profiles().pairs().add_favorite(args.into());
                crate::config::write(&client, &config)?;
                Ok(crate::Output::ConfigSet)
            }
            Commands::Del { index } => {
                config.functions().profiles().pairs().del_favorite(index)?;
                crate::config::write(&client, &config)?;
                Ok(crate::Output::ConfigSet)
            }
            Commands::Edit { args } => {
                let favorite = config.functions().profiles().pairs().edit_favorite(args.index)?;
                args.apply(favorite);
                crate::config::write(&client, &config)?;
                Ok(crate::Output::ConfigSet)
            }
        }
    }
}
