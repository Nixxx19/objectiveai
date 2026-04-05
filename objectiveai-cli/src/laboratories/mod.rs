mod create;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new laboratory
    Create {
        #[command(flatten)]
        args: create::CreateArgs,
    },
}

impl Commands {
    pub async fn handle(self, _cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Create { args: _ } => unimplemented!(),
        }
    }
}
