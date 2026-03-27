mod config;
mod error;
mod remote;
mod get;
mod list;
mod favorite;
mod python;
mod api;
mod agents;
mod swarms;
mod functions;
mod viewer;
mod schemas;

#[cfg(test)]
mod python_tests;

use clap::{Parser, Subcommand};

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

#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();
    let cli = Cli::parse();
    match cli.command.handle().await {
        Ok(Output::ConfigGet(output)) => println!("{output}"),
        Ok(Output::ConfigSet) => println!("ok"),
        Ok(Output::Api(output)) => println!("{output}"),
        Ok(Output::Schema(output)) => print!("{output}"),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
