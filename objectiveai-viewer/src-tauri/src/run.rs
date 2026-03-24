use axum::Json;
use envconfig::Envconfig;
use serde::Serialize;
use tauri::Emitter;
use tokio::sync::mpsc;
use crate::functions;

#[derive(Envconfig)]
struct EnvConfigBuilder {
    #[envconfig(from = "ADDRESS")]
    address: Option<String>,
    #[envconfig(from = "PORT")]
    port: Option<u16>,
}

impl EnvConfigBuilder {
    pub fn build(self) -> ConfigBuilder {
        ConfigBuilder {
            address: self.address,
            port: self.port,
            suppress_output: None,
        }
    }
}

#[derive(Default)]
pub struct ConfigBuilder {
    pub address: Option<String>,
    pub port: Option<u16>,
    pub suppress_output: Option<bool>,
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
            address: self.address.unwrap_or_else(|| "0.0.0.0".to_string()),
            port: self.port.unwrap_or(5001),
            suppress_output: self.suppress_output.unwrap_or(false),
        }
    }
}

pub struct Config {
    pub address: String,
    pub port: u16,
    pub suppress_output: bool,
}

pub async fn setup(config: Config) -> std::io::Result<(tokio::net::TcpListener, axum::Router, EventReceiver)> {
    let (tx, rx) = mpsc::unbounded_channel::<Event>();

    let app = axum::Router::new()
        .route(
            "/functions/executions",
            axum::routing::post({
                let tx = tx.clone();
                move |Json(request): Json<functions::executions::request::Request>| async move {
                    tx.send(Event::FunctionsExecutions(request)).ok();
                    axum::http::StatusCode::OK
                }
            }),
        )
        .route(
            "/functions/inventions/recursive",
            axum::routing::post({
                let tx = tx.clone();
                move |Json(request): Json<functions::inventions::recursive::request::Request>| async move {
                    tx.send(Event::FunctionsInventionsRecursive(request)).ok();
                    axum::http::StatusCode::OK
                }
            }),
        );

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.address, config.port)).await?;

    Ok((listener, app, rx))
}

pub async fn serve(listener: tokio::net::TcpListener, app: axum::Router, mut rx: EventReceiver) -> std::io::Result<()> {
    tokio::spawn(async move {
        axum::serve(listener, app).await
    });

    tauri::Builder::default()
        .setup(move |tauri_app| {
            let handle = tauri_app.handle().clone();
            tauri::async_runtime::spawn(async move {
                while let Some(event) = rx.recv().await {
                    handle.emit(event.name(), &event).ok();
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}

pub async fn run(config: Config) -> std::io::Result<()> {
    let suppress_output = config.suppress_output;
    let (listener, app, rx) = setup(config).await?;
    if !suppress_output {
        let addr = listener.local_addr()?;
        eprintln!("listening on {addr}");
    }
    serve(listener, app, rx).await
}

#[derive(Clone, Serialize)]
#[serde(untagged)]
pub enum Event {
    FunctionsExecutions(functions::executions::request::Request),
    FunctionsInventionsRecursive(functions::inventions::recursive::request::Request),
}

impl Event {
    fn name(&self) -> &'static str {
        match self {
            Event::FunctionsExecutions(_) => "functions-executions",
            Event::FunctionsInventionsRecursive(_) => "functions-inventions-recursive",
        }
    }
}

pub type EventReceiver = mpsc::UnboundedReceiver<Event>;
