use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get all favorites
    Get,
    /// Add a favorite
    Add {
        #[command(flatten)]
        args: crate::favorite::AddFavorite,
    },
    /// Delete a favorite by name
    Del { name: String },
    /// Edit a favorite
    Edit {
        #[command(flatten)]
        args: crate::favorite::EditFavorite,
    },
}

impl Commands {
    pub fn handle(self) -> Result<crate::Output, crate::error::Error> {
        let (client, mut config) = crate::config::read()?;
        match self {
            Commands::Get => Ok(crate::Output::ConfigGet(crate::config::format_value(&config.functions().profiles().get_favorites()))),
            Commands::Add { args } => {
                config.functions().profiles().add_favorite(args.into_favorite()?);
                crate::config::write(&client, &config)?;
                Ok(crate::Output::ConfigSet)
            }
            Commands::Del { name } => {
                config.functions().profiles().del_favorite(&name)?;
                crate::config::write(&client, &config)?;
                Ok(crate::Output::ConfigSet)
            }
            Commands::Edit { args } => {
                let favorite = config.functions().profiles().edit_favorite(&args.name)?;
                args.apply(favorite)?;
                crate::config::write(&client, &config)?;
                Ok(crate::Output::ConfigSet)
            }
        }
    }
}
