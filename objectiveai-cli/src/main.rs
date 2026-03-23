use clap::{Parser, Subcommand};
use objectiveai::ConfigClient;

#[derive(Parser)]
#[command(name = "oai")]
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
    /// Query config using jq syntax
    Get {
        /// jq filter expression (e.g. '.api.mode', '.authorization | keys')
        filter: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Config { command } => match command {
            ConfigCommands::Get { filter } => {
                let client = ConfigClient::new(None::<String>);
                let config = match client.read() {
                    Ok(config) => config,
                    Err(e) => {
                        eprintln!("Error reading config: {e}");
                        std::process::exit(1);
                    }
                };
                match config.jq(&filter) {
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
        },
    }
}
