mod config;
mod error;
mod favorite;
mod api;
mod agents;
mod swarms;
mod functions;
mod viewer;

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
}

impl Commands {
    pub async fn handle(self) -> Result<Output, error::Error> {
        match self {
            Commands::Api { command } => command.handle(),
            Commands::Agents { command } => command.handle(),
            Commands::Swarms { command } => command.handle(),
            Commands::Functions { command } => command.handle(),
            Commands::Viewer { command } => command.handle(),
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command.handle().await {
        Ok(Output::ConfigGet(output)) => println!("{output}"),
        Ok(Output::ConfigSet) => println!("ok"),
        Ok(Output::Api(output)) => println!("{output}"),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
