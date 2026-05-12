use axum::Json;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::{self, Next};
use envconfig::Envconfig;
use objectiveai::HttpClient;
use objectiveai::agent::completions::request::AgentCompletionNotifyParams;
use objectiveai::filesystem::Client as FsClient;
use objectiveai::filesystem::plugins::{HttpMethod, ManifestWithNameAndSource, ViewerRoute};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::{mpsc, Notify};
use crate::agent;
use crate::functions;
use crate::laboratories;

#[tauri::command]
fn viewer_ready(state: tauri::State<'_, Arc<Notify>>) {
    state.notify_one();
}

#[tauri::command]
async fn notify_agent_completion(
    state: tauri::State<'_, HttpClient>,
    params: AgentCompletionNotifyParams,
) -> Result<(), String> {
    objectiveai::agent::completions::notify_agent_completion(
        state.inner(),
        params,
    )
    .await
    .map_err(|e| e.to_string())
}

#[derive(Envconfig)]
struct EnvConfigBuilder {
    // -- HttpClient fields (identical order across all 3 structs) --
    #[envconfig(from = "OBJECTIVEAI_ADDRESS")]
    objectiveai_address: Option<String>,
    #[envconfig(from = "OBJECTIVEAI_AUTHORIZATION")]
    objectiveai_authorization: Option<String>,
    #[envconfig(from = "OPENROUTER_ADDRESS")]
    openrouter_address: Option<String>,
    #[envconfig(from = "OPENROUTER_AUTHORIZATION")]
    openrouter_authorization: Option<String>,
    #[envconfig(from = "GITHUB_AUTHORIZATION")]
    github_authorization: Option<String>,
    #[envconfig(from = "MCP_AUTHORIZATION")]
    mcp_authorization: Option<String>,
    #[envconfig(from = "VIEWER_SIGNATURE")]
    viewer_signature: Option<String>,
    #[envconfig(from = "USER_AGENT")]
    user_agent: Option<String>,
    #[envconfig(from = "HTTP_REFERER")]
    http_referer: Option<String>,
    #[envconfig(from = "X_TITLE")]
    x_title: Option<String>,
    #[envconfig(from = "COMMIT_AUTHOR_NAME")]
    commit_author_name: Option<String>,
    #[envconfig(from = "COMMIT_AUTHOR_EMAIL")]
    commit_author_email: Option<String>,
    // -- Other fields --
    #[envconfig(from = "ADDRESS")]
    address: Option<String>,
    #[envconfig(from = "PORT")]
    port: Option<u16>,
    #[envconfig(from = "VIEWER_SECRET")]
    secret: Option<String>,
    #[envconfig(from = "CONFIG_BASE_DIR")]
    config_base_dir: Option<String>,
}

impl EnvConfigBuilder {
    pub fn build(self) -> ConfigBuilder {
        ConfigBuilder {
            // -- HttpClient fields --
            objectiveai_address: self.objectiveai_address,
            objectiveai_authorization: self.objectiveai_authorization,
            openrouter_address: self.openrouter_address,
            openrouter_authorization: self.openrouter_authorization,
            github_authorization: self.github_authorization,
            mcp_authorization: self.mcp_authorization,
            viewer_signature: self.viewer_signature,
            user_agent: self.user_agent,
            http_referer: self.http_referer,
            x_title: self.x_title,
            commit_author_name: self.commit_author_name,
            commit_author_email: self.commit_author_email,
            // -- Other fields --
            address: self.address,
            port: self.port,
            suppress_output: None,
            secret: self.secret,
            config_base_dir: self.config_base_dir,
        }
    }
}

#[derive(Default)]
pub struct ConfigBuilder {
    // -- HttpClient fields (identical order across all 3 structs) --
    pub objectiveai_address: Option<String>,
    pub objectiveai_authorization: Option<String>,
    pub openrouter_address: Option<String>,
    pub openrouter_authorization: Option<String>,
    pub github_authorization: Option<String>,
    pub mcp_authorization: Option<String>,
    pub viewer_signature: Option<String>,
    pub user_agent: Option<String>,
    pub http_referer: Option<String>,
    pub x_title: Option<String>,
    pub commit_author_name: Option<String>,
    pub commit_author_email: Option<String>,
    // -- Other fields --
    pub address: Option<String>,
    pub port: Option<u16>,
    pub suppress_output: Option<bool>,
    pub secret: Option<String>,
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
            // -- HttpClient fields --
            objectiveai_address: self.objectiveai_address,
            objectiveai_authorization: self.objectiveai_authorization,
            openrouter_address: self.openrouter_address,
            openrouter_authorization: self.openrouter_authorization,
            github_authorization: self.github_authorization,
            mcp_authorization: self.mcp_authorization,
            viewer_signature: self.viewer_signature,
            user_agent: self.user_agent,
            http_referer: self.http_referer,
            x_title: self.x_title,
            commit_author_name: self.commit_author_name,
            commit_author_email: self.commit_author_email,
            // -- Other fields --
            address: self.address.unwrap_or_else(|| "0.0.0.0".to_string()),
            port: self.port.unwrap_or(5001),
            suppress_output: self.suppress_output.unwrap_or(false),
            secret: self.secret,
            config_base_dir: self.config_base_dir,
        }
    }
}

pub struct Config {
    // -- HttpClient fields (identical order across all 3 structs) --
    pub objectiveai_address: Option<String>,
    pub objectiveai_authorization: Option<String>,
    pub openrouter_address: Option<String>,
    pub openrouter_authorization: Option<String>,
    pub github_authorization: Option<String>,
    pub mcp_authorization: Option<String>,
    pub viewer_signature: Option<String>,
    pub user_agent: Option<String>,
    pub http_referer: Option<String>,
    pub x_title: Option<String>,
    pub commit_author_name: Option<String>,
    pub commit_author_email: Option<String>,
    // -- Other fields --
    pub address: String,
    pub port: u16,
    pub suppress_output: bool,
    pub secret: Option<String>,
    pub config_base_dir: Option<String>,
}

pub async fn setup(config: Config) -> std::io::Result<(tokio::net::TcpListener, axum::Router, EventReceiver, HttpClient, FsClient)> {
    let (tx, rx) = mpsc::unbounded_channel::<Event>();
    let secret = config.secret.map(Arc::new);

    let mcp_authorization: Option<std::collections::HashMap<String, String>> =
        config.mcp_authorization.and_then(|s| serde_json::from_str(&s).ok());

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.address, config.port)).await?;
    let viewer_address = format!("http://{}", listener.local_addr()?);

    let commit_author_name = config.commit_author_name.clone();
    let commit_author_email = config.commit_author_email.clone();
    let http_client = HttpClient::new(
        reqwest::Client::new(),
        config.objectiveai_address,
        config.objectiveai_authorization,
        config.user_agent,
        config.x_title,
        config.http_referer,
        config.github_authorization,
        config.openrouter_authorization,
        mcp_authorization,
        config.viewer_signature,
        Some(viewer_address),
        config.commit_author_name,
        config.commit_author_email,
    );

    let mut app = axum::Router::new()
        .route(
            "/agent/completions",
            axum::routing::post({
                let tx = tx.clone();
                move |Json(request): Json<agent::completions::request::Request>| async move {
                    tx.send(Event::AgentCompletions(request)).ok();
                    StatusCode::OK
                }
            }),
        )
        .route(
            "/functions/executions",
            axum::routing::post({
                let tx = tx.clone();
                move |Json(request): Json<functions::executions::request::Request>| async move {
                    tx.send(Event::FunctionsExecutions(request)).ok();
                    StatusCode::OK
                }
            }),
        )
        .route(
            "/functions/inventions/recursive",
            axum::routing::post({
                let tx = tx.clone();
                move |Json(request): Json<functions::inventions::recursive::request::Request>| async move {
                    tx.send(Event::FunctionsInventionsRecursive(request)).ok();
                    StatusCode::OK
                }
            }),
        )
        .route(
            "/laboratories/executions",
            axum::routing::post({
                let tx = tx.clone();
                move |Json(request): Json<laboratories::executions::request::Request>| async move {
                    tx.send(Event::LaboratoriesExecutions(request)).ok();
                    StatusCode::OK
                }
            }),
        );

    let fs_client = FsClient::new(
        config.config_base_dir.as_deref(),
        commit_author_name.as_deref(),
        commit_author_email.as_deref(),
    );

    // Scan installed plugins and register any viewer routes they
    // declare. Listing is once-at-startup; the user opts in to
    // refresh by restarting the viewer.
    let plugins: Vec<ManifestWithNameAndSource> = fs_client.list_plugins(0, usize::MAX).await;
    for plugin in plugins {
        let plugin_name = plugin.name.clone();
        for route in plugin.manifest.viewer_routes {
            if !route.path.starts_with('/') {
                eprintln!(
                    "skipping plugin {plugin_name:?} route with non-`/`-prefixed path: {:?}",
                    route.path
                );
                continue;
            }
            app = register_plugin_route(app, tx.clone(), plugin_name.clone(), route);
        }
    }

    let app = app.layer(middleware::from_fn_with_state(secret, signature_middleware));

    Ok((listener, app, rx, http_client, fs_client))
}

/// A function that exits the viewer's event loop with the given exit code.
pub type Exiter = Box<dyn FnOnce(i32) + Send>;

/// Must be called on the main thread. Tauri's event loop panics otherwise.
/// Spawn `setup` and other async work on tokio tasks instead.
///
/// If `exiter_tx` is provided, an `Exiter` is sent through it once
/// Tauri is initialized. Call the exiter from a spawned task
/// to make `serve` return.
///
/// Returns the exit code from Tauri's event loop.
pub fn serve(
    listener: tokio::net::TcpListener,
    app: axum::Router,
    mut rx: EventReceiver,
    http_client: HttpClient,
    fs_client: FsClient,
    exiter_tx: Option<tokio::sync::oneshot::Sender<Exiter>>,
) -> i32 {
    tokio::spawn(async move {
        axum::serve(listener, app).await
    });

    let ready = Arc::new(Notify::new());
    let ready_for_task = ready.clone();

    tauri::Builder::default()
        .manage(ready)
        .manage(http_client)
        .manage(fs_client)
        .invoke_handler(tauri::generate_handler![
            viewer_ready,
            notify_agent_completion,
            list_plugins_with_viewer,
            plugin_invoke
        ])
        .setup(move |tauri_app| {
            let handle = tauri_app.handle().clone();
            if let Some(tx) = exiter_tx {
                let exit_handle = handle.clone();
                tx.send(Box::new(move |code| exit_handle.exit(code))).ok();
            }
            tauri::async_runtime::spawn(async move {
                // Buffer events until the frontend signals it is listening.
                let mut buffer = Vec::new();
                loop {
                    tokio::select! {
                        biased;
                        _ = ready_for_task.notified() => break,
                        event = rx.recv() => {
                            match event {
                                Some(e) => buffer.push(e),
                                None => return,
                            }
                        }
                    }
                }
                // Drain buffered events.
                for event in buffer {
                    let _ = handle.emit(&event.name(), &event);
                }
                // Forward remaining events directly.
                while let Some(event) = rx.recv().await {
                    let _ = handle.emit(&event.name(), &event);
                }
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error building tauri application")
        .run_return(|_, _| {})
}

/// Sets up and serves the viewer. Returns the exit code from Tauri's event loop.
/// The caller should use `std::process::exit(code)` with the returned value.
pub async fn run(config: Config) -> std::io::Result<i32> {
    let suppress_output = config.suppress_output;
    let (listener, app, rx, http_client, fs_client) = setup(config).await?;
    if !suppress_output {
        let addr = listener.local_addr()?;
        eprintln!("listening on {addr}");
    }
    Ok(serve(listener, app, rx, http_client, fs_client, None))
}

async fn signature_middleware(
    State(secret): State<Option<Arc<String>>>,
    request: axum::extract::Request,
    next: Next,
) -> Result<axum::response::Response, StatusCode> {
    if let Some(secret) = &secret {
        let (parts, body) = request.into_parts();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.map_err(|_| StatusCode::BAD_REQUEST)?;
        let headers = &parts.headers;
        let signature = headers
            .get("X-VIEWER-SIGNATURE")
            .or_else(|| headers.get("VIEWER-SIGNATURE"))
            .or_else(|| headers.get("X-OBJECTIVEAI-SIGNATURE"))
            .or_else(|| headers.get("OBJECTIVEAI-SIGNATURE"))
            .and_then(|v| v.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;
        if !verify_signature(secret, &bytes, signature) {
            return Err(StatusCode::UNAUTHORIZED);
        }
        let rebuilt = axum::http::Request::from_parts(parts, axum::body::Body::from(bytes));
        Ok(next.run(rebuilt).await)
    } else {
        Ok(next.run(request).await)
    }
}

fn verify_signature(secret: &str, _body: &[u8], signature_header: &str) -> bool {
    let Some(hex_sig) = signature_header.strip_prefix("sha256=") else {
        return false;
    };
    let Ok(sig_bytes) = hex::decode(hex_sig) else {
        return false;
    };
    // Compute SHA256(secret) and compare against the provided signature.
    // The signature is a static pre-computed value: sha256=<SHA256(secret)>.
    // Knowing the signature does not reveal the secret (preimage resistance).
    use sha2::{Sha256, Digest};
    let expected = Sha256::digest(secret.as_bytes());
    expected.ct_eq(&sig_bytes).into()
}

#[derive(Clone, Serialize)]
#[serde(untagged)]
pub enum Event {
    AgentCompletions(agent::completions::request::Request),
    FunctionsExecutions(functions::executions::request::Request),
    FunctionsInventionsRecursive(functions::inventions::recursive::request::Request),
    LaboratoriesExecutions(laboratories::executions::request::Request),
    Plugin(PluginEvent),
}

/// Payload emitted whenever a plugin's viewer route is hit. `plugin`
/// identifies which plugin's iframe should receive the event; the
/// host's tab shell uses this to route via postMessage. `request`
/// wraps the route's manifest-declared `type` tag and the JSON body.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginEvent {
    pub plugin: String,
    pub request: PluginRequest,
}

/// Wire shape forwarded to a plugin's iframe. `type` is the string
/// tag the plugin author declared in their manifest's `viewer_routes`
/// entry; `value` is the JSON body of the HTTP request (or
/// `Value::Null` for bodies axum couldn't parse / GET requests).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginRequest {
    #[serde(rename = "type")]
    pub r#type: String,
    pub value: serde_json::Value,
}

impl Event {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        match self {
            Event::AgentCompletions(_) => std::borrow::Cow::Borrowed("agent-completions"),
            Event::FunctionsExecutions(_) => std::borrow::Cow::Borrowed("functions-executions"),
            Event::FunctionsInventionsRecursive(_) => {
                std::borrow::Cow::Borrowed("functions-inventions-recursive")
            }
            Event::LaboratoriesExecutions(_) => std::borrow::Cow::Borrowed("laboratories-executions"),
            Event::Plugin(p) => std::borrow::Cow::Owned(format!("plugin-{}", p.plugin)),
        }
    }
}

pub type EventReceiver = mpsc::UnboundedReceiver<Event>;
pub type EventSender = mpsc::UnboundedSender<Event>;

/// Register one plugin viewer route on the given axum router. The
/// route lands at `/plugin/<plugin>/<route.path>`; a hit emits an
/// `Event::Plugin` carrying the manifest-declared `type` tag and the
/// request body. Body-less requests (GET, or POST with no body) yield
/// `Value::Null` as the request value.
fn register_plugin_route(
    app: axum::Router,
    tx: EventSender,
    plugin: String,
    route: ViewerRoute,
) -> axum::Router {
    let full_path = format!("/plugin/{plugin}{}", route.path);
    let r#type = route.r#type.clone();
    let method = route.method;
    let plugin_for_handler = plugin.clone();

    let handler = move |body: Option<Json<serde_json::Value>>| {
        let tx = tx.clone();
        let plugin = plugin_for_handler.clone();
        let r#type = r#type.clone();
        async move {
            let value = body.map(|Json(v)| v).unwrap_or(serde_json::Value::Null);
            tx.send(Event::Plugin(PluginEvent {
                plugin,
                request: PluginRequest { r#type, value },
            }))
            .ok();
            StatusCode::OK
        }
    };

    let method_router = match method {
        HttpMethod::Get => axum::routing::get(handler),
        HttpMethod::Post => axum::routing::post(handler),
        HttpMethod::Put => axum::routing::put(handler),
        HttpMethod::Patch => axum::routing::patch(handler),
        HttpMethod::Delete => axum::routing::delete(handler),
    };
    app.route(&full_path, method_router)
}

/// Subset of `ManifestWithNameAndSource` returned to the frontend by
/// `list_plugins_with_viewer`. Pre-filtered to plugins that have a
/// viewer surface (either `viewer_zip` or `viewer_routes`).
#[derive(Clone, Debug, Serialize)]
pub struct PluginTab {
    pub name: String,
    pub description: String,
    pub version: String,
    pub has_viewer_bundle: bool,
}

#[tauri::command]
async fn list_plugins_with_viewer(
    state: tauri::State<'_, FsClient>,
) -> Result<Vec<PluginTab>, String> {
    let plugins = state.inner().list_plugins(0, usize::MAX).await;
    Ok(plugins
        .into_iter()
        .filter(|p| p.manifest.viewer_zip.is_some() || !p.manifest.viewer_routes.is_empty())
        .map(|p| PluginTab {
            name: p.name.clone(),
            description: p.manifest.description,
            version: p.manifest.version,
            has_viewer_bundle: p.manifest.viewer_zip.is_some(),
        })
        .collect())
}

/// Invoke a plugin's binary with a single JSON payload, return its
/// stdout JSONL lines as parsed `serde_json::Value`s. The plugin's
/// stdin receives `{"type": method, "value": args}` followed by EOF;
/// the plugin's stdout is read until EOF. Each line is parsed as
/// JSON (parse failures become string-valued JSON) and collected.
///
/// This is the first-cut bridge target for the frontend's
/// postMessage router (plugin iframes call `invoke(method, args)`
/// → bridge → this command → plugin binary).
#[tauri::command]
async fn plugin_invoke(
    state: tauri::State<'_, FsClient>,
    name: String,
    method: String,
    args: serde_json::Value,
) -> Result<Vec<serde_json::Value>, String> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

    let exe = state
        .inner()
        .resolve_plugin(&name)
        .await
        .ok_or_else(|| format!("plugin not found: {name}"))?;

    let mut child = tokio::process::Command::new(&exe)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .map_err(|e| format!("spawn failed: {e}"))?;

    let request = serde_json::json!({ "type": method, "value": args });
    let request_line = format!("{}\n", serde_json::to_string(&request).unwrap_or_default());
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(request_line.as_bytes())
            .await
            .map_err(|e| format!("write stdin: {e}"))?;
        // Drop stdin so the plugin sees EOF and can exit.
        drop(stdin);
    }

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "stdout missing".to_string())?;
    let mut reader = tokio::io::BufReader::new(stdout);
    let mut lines = Vec::new();
    let mut buf = String::new();
    loop {
        buf.clear();
        let n = reader
            .read_line(&mut buf)
            .await
            .map_err(|e| format!("read stdout: {e}"))?;
        if n == 0 {
            break;
        }
        let trimmed = buf.trim_end_matches(['\r', '\n']);
        let value = serde_json::from_str::<serde_json::Value>(trimmed)
            .unwrap_or_else(|_| serde_json::Value::String(trimmed.to_string()));
        lines.push(value);
    }

    let _ = child.wait().await;
    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn register_plugin_route_emits_event_with_type_and_value() {
        let (tx, mut rx) = mpsc::unbounded_channel::<Event>();
        let route = ViewerRoute {
            path: "/echo".to_string(),
            method: HttpMethod::Post,
            r#type: "echo_request".to_string(),
        };
        let app = register_plugin_route(axum::Router::new(), tx, "myplugin".to_string(), route);

        let response = app
            .oneshot(
                Request::post("/plugin/myplugin/echo")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"hello":"world"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let event = rx.try_recv().expect("expected an event");
        match event {
            Event::Plugin(PluginEvent { plugin, request }) => {
                assert_eq!(plugin, "myplugin");
                assert_eq!(request.r#type, "echo_request");
                assert_eq!(request.value["hello"], "world");
            }
            _ => panic!("expected Event::Plugin"),
        }
    }

    #[tokio::test]
    async fn register_plugin_route_emits_null_value_for_get() {
        let (tx, mut rx) = mpsc::unbounded_channel::<Event>();
        let route = ViewerRoute {
            path: "/status".to_string(),
            method: HttpMethod::Get,
            r#type: "status_request".to_string(),
        };
        let app = register_plugin_route(axum::Router::new(), tx, "p".to_string(), route);

        let response = app
            .oneshot(
                Request::get("/plugin/p/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let event = rx.try_recv().expect("expected an event");
        match event {
            Event::Plugin(PluginEvent { request, .. }) => {
                assert_eq!(request.r#type, "status_request");
                assert!(request.value.is_null());
            }
            _ => panic!("expected Event::Plugin"),
        }
    }

    #[test]
    fn event_name_for_plugin_includes_plugin_name() {
        let e = Event::Plugin(PluginEvent {
            plugin: "myplugin".to_string(),
            request: PluginRequest {
                r#type: "x".to_string(),
                value: serde_json::Value::Null,
            },
        });
        assert_eq!(&*e.name(), "plugin-myplugin");
    }
}
