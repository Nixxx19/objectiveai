use clap::{Parser, Subcommand};
use objectiveai::{Config, ConfigClient, ConfigError};

#[derive(Parser)]
#[command(name = "objectiveai")]
#[command(about = "ObjectiveAI CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Query full config using jq syntax
    Get {
        /// jq filter expression (defaults to '.' if omitted)
        filter: Option<String>,
    },
    /// API configuration
    Api {
        #[command(subcommand)]
        command: ApiCommands,
    },
    /// Authorization configuration
    Authorization {
        #[command(subcommand)]
        command: GetCommand,
    },
    /// Agents configuration
    Agents {
        #[command(subcommand)]
        command: GetCommand,
    },
    /// Swarms configuration
    Swarms {
        #[command(subcommand)]
        command: GetCommand,
    },
    /// Functions configuration
    Functions {
        #[command(subcommand)]
        command: FunctionsCommands,
    },
}

#[derive(Subcommand)]
enum ApiCommands {
    /// Query API config using jq syntax
    Get {
        /// jq filter expression (defaults to '.' if omitted)
        filter: Option<String>,
    },
    /// Remote API configuration
    Remote {
        #[command(subcommand)]
        command: GetCommand,
    },
    /// Local API configuration
    Local {
        #[command(subcommand)]
        command: GetCommand,
    },
}

#[derive(Subcommand)]
enum FunctionsCommands {
    /// Query functions config using jq syntax
    Get {
        /// jq filter expression (defaults to '.' if omitted)
        filter: Option<String>,
    },
    /// Functions inventions configuration
    Inventions {
        #[command(subcommand)]
        command: GetCommand,
    },
    /// Functions profiles configuration
    Profiles {
        #[command(subcommand)]
        command: GetCommand,
    },
}

#[derive(Subcommand)]
enum GetCommand {
    /// Query using jq syntax
    Get {
        /// jq filter expression (defaults to '.' if omitted)
        filter: Option<String>,
    },
}

fn filter(f: Option<String>) -> String {
    f.unwrap_or_else(|| ".".to_string())
}

fn read_config() -> Config {
    let client = ConfigClient::new(None::<String>);
    match client.read() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error reading config: {e}");
            std::process::exit(1);
        }
    }
}

fn print_jq(results: Result<Vec<serde_json::Value>, ConfigError>) {
    match results {
        Ok(results) => {
            let output = match results.len() {
                0 => serde_json::to_string(&serde_json::Value::Null).unwrap(),
                1 => serde_json::to_string(&results[0]).unwrap(),
                _ => serde_json::to_string(&results).unwrap(),
            };
            println!("{output}");
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Config { command } => {
            let mut config = read_config();
            let results = match command {
                ConfigCommands::Get { filter: f } => config.jq(&filter(f)),
                ConfigCommands::Api { command: ApiCommands::Get { filter: f } } => config.api().jq(&filter(f)),
                ConfigCommands::Api { command: ApiCommands::Remote { command: GetCommand::Get { filter: f } } } => config.api().remote().jq(&filter(f)),
                ConfigCommands::Api { command: ApiCommands::Local { command: GetCommand::Get { filter: f } } } => config.api().local().jq(&filter(f)),
                ConfigCommands::Authorization { command: GetCommand::Get { filter: f } } => config.authorization().jq(&filter(f)),
                ConfigCommands::Agents { command: GetCommand::Get { filter: f } } => config.agents().jq(&filter(f)),
                ConfigCommands::Swarms { command: GetCommand::Get { filter: f } } => config.swarms().jq(&filter(f)),
                ConfigCommands::Functions { command: FunctionsCommands::Get { filter: f } } => config.functions().jq(&filter(f)),
                ConfigCommands::Functions { command: FunctionsCommands::Inventions { command: GetCommand::Get { filter: f } } } => config.functions().inventions().jq(&filter(f)),
                ConfigCommands::Functions { command: FunctionsCommands::Profiles { command: GetCommand::Get { filter: f } } } => config.functions().profiles().jq(&filter(f)),
            };
            print_jq(results);
        }
    }
}
