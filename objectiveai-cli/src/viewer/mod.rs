pub mod config;
pub mod mode;
pub mod local;
pub mod send;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Viewer configuration
    Config {
        #[command(subcommand)]
        command: config::Commands,
    },
    /// Viewer mode (remote or local)
    Mode {
        #[command(subcommand)]
        command: mode::Commands,
    },
    /// Local viewer configuration
    Local {
        #[command(subcommand)]
        command: local::Commands,
    },
    /// Generate a new secret/signature pair
    GenerateSecretSignaturePair,
    /// POST a JSON body to a viewer HTTP route and wait for the
    /// response.
    ///
    /// In **remote** viewer mode, reads `api.headers.x_viewer_address`
    /// and `api.headers.x_viewer_signature` from the filesystem config
    /// (the same source the api server's viewer client uses) and POSTs
    /// there. In **local** viewer mode, spawns the embedded viewer
    /// subprocess on a random port, POSTs to its bound address with the
    /// locally-configured signature, then kills the subprocess on the
    /// way out — matching the spawn-kill lifecycle the api server uses
    /// for transient tasks.
    Send {
        /// URL path on the viewer (must start with `/`), e.g.
        /// `/agent/completions` or `/plugin/<repository>/<route>`.
        path: String,
        /// JSON body to POST. Validated as well-formed JSON before
        /// send.
        body: String,
    },
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config, handle: &objectiveai_sdk::cli::output::Handle) -> Result<(), crate::error::Error> {
        match self {
            Commands::Config { command } => command.handle(cli_config, handle).await,
            Commands::Mode { command } => command.handle(cli_config, handle).await,
            Commands::Local { command } => command.handle(cli_config, handle).await,
            Commands::GenerateSecretSignaturePair => {
                let pair = objectiveai_sdk::filesystem::config::generate_viewer_secret_signature_pair();
                crate::config::emit_value(&pair, handle).await;
                Ok(())
            }
            Commands::Send { path, body } => send::run(cli_config, handle, &path, &body).await,
        }
    }
}
