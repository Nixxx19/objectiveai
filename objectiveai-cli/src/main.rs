mod config;
mod error;
mod favorite;
mod api;
mod authorization;
mod agents;
mod swarms;
mod functions;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "objectiveai")]
#[command(about = "ObjectiveAI CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// API configuration and operations
    Api {
        #[command(subcommand)]
        command: api::Commands,
    },
    /// Authorization management
    Authorization {
        #[command(subcommand)]
        command: authorization::Commands,
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
}

impl Commands {
    pub fn handle(self) -> Result<Option<String>, error::Error> {
        match self {
            Commands::Api { command } => command.handle(),
            Commands::Authorization { command } => command.handle(),
            Commands::Agents { command } => command.handle(),
            Commands::Swarms { command } => command.handle(),
            Commands::Functions { command } => command.handle(),
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command.handle() {
        Ok(Some(output)) => println!("{output}"),
        Ok(None) => {}
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}
