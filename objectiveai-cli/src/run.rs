use clap::{Parser, Subcommand};
use envconfig::Envconfig;

use crate::api;
use crate::agents;
use crate::swarms;
use crate::functions;
use crate::viewer;
use crate::schemas;
use crate::laboratories;
use crate::error;

#[derive(Envconfig)]
struct EnvConfigBuilder {
    #[envconfig(from = "CONFIG_SET_FORBIDDEN")]
    config_set_forbidden: Option<String>,
    #[envconfig(from = "CONFIG_BASE_DIR")]
    config_base_dir: Option<String>,
}

impl EnvConfigBuilder {
    pub fn build(self) -> ConfigBuilder {
        fn parse_bool(s: &str) -> bool {
            let v = s.trim();
            !v.is_empty() && v != "0" && !v.eq_ignore_ascii_case("false")
        }
        ConfigBuilder {
            config_set_forbidden: self.config_set_forbidden.map(|s| parse_bool(&s)),
            config_base_dir: self.config_base_dir,
        }
    }
}

#[derive(Default)]
pub struct ConfigBuilder {
    pub config_set_forbidden: Option<bool>,
    pub config_base_dir: Option<String>,
}

impl Envconfig for ConfigBuilder {
    #[allow(deprecated)]
    fn init() -> Result<Self, envconfig::Error> {
        EnvConfigBuilder::init().map(|e| e.build())
    }

    fn init_from_env() -> Result<Self, envconfig::Error> {
        EnvConfigBuilder::init_from_env().map(|e| e.build())
    }

    fn init_from_hashmap(hashmap: &std::collections::HashMap<String, String>) -> Result<Self, envconfig::Error> {
        EnvConfigBuilder::init_from_hashmap(hashmap).map(|e| e.build())
    }
}

impl ConfigBuilder {
    pub fn build(self) -> Config {
        Config {
            config_set_forbidden: self.config_set_forbidden.unwrap_or(false),
            config_base_dir: self.config_base_dir,
        }
    }
}

pub struct Config {
    pub config_set_forbidden: bool,
    pub config_base_dir: Option<String>,
}

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
    /// Laboratories management
    Laboratories {
        #[command(subcommand)]
        command: laboratories::Commands,
    },
}

impl Commands {
    pub async fn handle(self, cli_config: &Config) -> Result<Output, error::Error> {
        match self {
            Commands::Api { command } => command.handle(cli_config).await,
            Commands::Agents { command } => command.handle(cli_config).await,
            Commands::Swarms { command } => command.handle(cli_config).await,
            Commands::Functions { command } => command.handle(cli_config).await,
            Commands::Viewer { command } => command.handle(cli_config).await,
            Commands::Schemas { command } => command.handle(),
            Commands::Laboratories { command } => command.handle(cli_config).await,
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
    let cli_config = ConfigBuilder::init_from_env().unwrap_or_default().build();
    match cli.command.handle(&cli_config).await {
        Ok(Output::ConfigGet(output)) => Ok(output),
        Ok(Output::ConfigSet) => Ok("ok".into()),
        Ok(Output::Api(output)) => Ok(output),
        Ok(Output::Schema(output)) => Ok(output.to_string()),
        Err(e) => Err(format!("{e}")),
    }
}
