use clap::{Parser, Subcommand};

use crate::api;
use crate::agents;
use crate::swarms;
use crate::functions;
use crate::viewer;
use crate::schemas;
use crate::error;

#[derive(Parser)]
#[command(name = "objectiveai")]
#[command(about = "ObjectiveAI CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

pub enum Output {
    ConfigGet(String),
    ConfigSet,
    Api(String),
    Schema(&'static str),
}

#[derive(Subcommand)]
enum Commands {
    /// API configuration and operations
    Api {
        #[command(subcommand)]
        command: api::Commands,
    },
    /// Agents management
    Agents {
        #[command(subcommand)]
        command: agents::Commands,
    },
    /// Swarms management
    Swarms {
        #[command(subcommand)]
        command: swarms::Commands,
    },
    /// Functions management
    Functions {
        #[command(subcommand)]
        command: functions::Commands,
    },
    /// Viewer management
    Viewer {
        #[command(subcommand)]
        command: viewer::Commands,
    },
    /// Browse JSON schemas
    Schemas {
        #[command(subcommand)]
        command: schemas::Commands,
    },
}

impl Commands {
    pub async fn handle(self) -> Result<Output, error::Error> {
        match self {
            Commands::Api { command } => command.handle(),
            Commands::Agents { command } => command.handle().await,
            Commands::Swarms { command } => command.handle().await,
            Commands::Functions { command } => command.handle().await,
            Commands::Viewer { command } => command.handle(),
            Commands::Schemas { command } => command.handle(),
        }
    }
}

/// Run the CLI, parsing arguments from the provided iterator.
/// The iterator should include the binary name as the first element
/// (e.g., `["objectiveai", "agents", "list"]`).
pub async fn run<I, T>(args: I) -> Result<String, String>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = Cli::try_parse_from(args).map_err(|e| e.to_string())?;
    match cli.command.handle().await {
        Ok(Output::ConfigGet(output)) => Ok(output),
        Ok(Output::ConfigSet) => Ok("ok".into()),
        Ok(Output::Api(output)) => Ok(output),
        Ok(Output::Schema(output)) => Ok(output.to_string()),
        Err(e) => Err(format!("{e}")),
    }
}
