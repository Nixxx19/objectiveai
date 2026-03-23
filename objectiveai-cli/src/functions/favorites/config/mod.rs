use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get all favorites
    Get,
    /// Add a favorite
    Add {
        #[command(subcommand)]
        command: crate::favorite::AddFavorite,
    },
    /// Delete a favorite by index
    Del { index: usize },
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
            Commands::Get => Ok(crate::Output::ConfigGet(crate::config::format_value(&config.functions().get_favorites()))),
            Commands::Add { command } => {
                config.functions().add_favorite(command.into());
                crate::config::write(&client, &config)?;
                Ok(crate::Output::ConfigSet)
            }
            Commands::Del { index } => {
                config.functions().del_favorite(index)?;
                crate::config::write(&client, &config)?;
                Ok(crate::Output::ConfigSet)
            }
            Commands::Edit { args } => {
                let favorite = config.functions().edit_favorite(args.index)?;
                args.apply(favorite);
                crate::config::write(&client, &config)?;
                Ok(crate::Output::ConfigSet)
            }
        }
    }
}
