use std::net::SocketAddr;

use envconfig::Envconfig;
use objectiveai_mcp_proxy::spawn_proxy;

#[derive(Envconfig)]
struct EnvConfigBuilder {
    #[envconfig(from = "ADDRESS")]
    address: Option<String>,
    #[envconfig(from = "PORT")]
    port: Option<u16>,
}

impl EnvConfigBuilder {
    fn build(self) -> ConfigBuilder {
        ConfigBuilder {
            address: self.address,
            port: self.port,
        }
    }
}

#[derive(Default)]
struct ConfigBuilder {
    address: Option<String>,
    port: Option<u16>,
}

impl Envconfig for ConfigBuilder {
    #[allow(deprecated)]
    fn init() -> Result<Self, envconfig::Error> {
        EnvConfigBuilder::init().map(|e| e.build())
    }

    fn init_from_env() -> Result<Self, envconfig::Error> {
        EnvConfigBuilder::init_from_env().map(|e| e.build())
    }

    fn init_from_hashmap(
        hashmap: &std::collections::HashMap<String, String>,
    ) -> Result<Self, envconfig::Error> {
        EnvConfigBuilder::init_from_hashmap(hashmap).map(|e| e.build())
    }
}

impl ConfigBuilder {
    fn build(self) -> Config {
        Config {
            address: self.address.unwrap_or_else(|| "0.0.0.0".into()),
            port: self.port.unwrap_or(3000),
        }
    }
}

struct Config {
    address: String,
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::DEBUG.into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let _ = dotenv::dotenv();
    let config = ConfigBuilder::init_from_env()
        .unwrap_or_default()
        .build();

    let address: SocketAddr = format!("{}:{}", config.address, config.port).parse()?;
    tracing::info!("Starting ObjectiveAI MCP proxy on {}", address);

    let handle = spawn_proxy(address).await?;
    tracing::info!("Listening on {}", handle.address);

    handle.serve_task.await??;
    Ok(())
}
