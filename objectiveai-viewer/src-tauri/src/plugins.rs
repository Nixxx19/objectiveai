//! Plugin glue: dynamic axum route registration, Tauri commands
//! invoked from the React frontend's postMessage bridge, and the
//! custom `plugin://` URI scheme handler that serves plugin UI
//! bundles out of `<plugins_dir>/<name>/viewer/`.

use axum::Json;
use axum::http::StatusCode;
use objectiveai::filesystem::Client as FsClient;
use objectiveai::filesystem::plugins::{HttpMethod, ViewerRoute};
use serde::Serialize;

use crate::events::{Event, EventSender, PluginEvent, PluginRequest};

/// Register one plugin viewer route on the given axum router. The
/// route lands at `/plugin/<plugin>/<route.path>`; a hit emits an
/// `Event::Plugin` carrying the manifest-declared `type` tag and the
/// request body. Body-less requests (GET, or POST with no body) yield
/// `Value::Null` as the request value.
pub(crate) fn register_plugin_route(
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
/// [`list_plugins_with_viewer`]. Pre-filtered to plugins that have a
/// viewer surface (either `viewer_zip` or `viewer_routes`).
#[derive(Clone, Debug, Serialize)]
pub struct PluginTab {
    pub name: String,
    pub description: String,
    pub version: String,
    pub has_viewer_bundle: bool,
}

#[tauri::command]
pub(crate) async fn list_plugins_with_viewer(
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
pub(crate) async fn plugin_invoke(
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

/// Handler for the custom `plugin://` URI scheme. Resolves
/// `plugin://localhost/<plugin>/<path>` to a file under
/// `<plugins_dir>/<plugin>/viewer/<path>`. Rejects path components
/// containing `..` so a plugin can't read outside its own viewer
/// subtree. Falls back to `<path>` = `index.html` when the request
/// path ends with `/`.
pub(crate) fn serve_plugin_asset(
    plugins_dir: &std::path::Path,
    request: tauri::http::Request<Vec<u8>>,
) -> tauri::http::Response<Vec<u8>> {
    let uri = request.uri();
    let mut segments: Vec<&str> = uri
        .path()
        .trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    if segments.is_empty() || segments.iter().any(|s| *s == "..") {
        return not_found();
    }
    if uri.path().ends_with('/') || segments.len() == 1 {
        segments.push("index.html");
    }
    let plugin = segments.remove(0);
    let rest: std::path::PathBuf = segments.iter().collect();
    let abs = plugins_dir.join(plugin).join("viewer").join(&rest);
    let canon_root = plugins_dir.join(plugin).join("viewer");

    // Path-traversal defense: the resolved path must remain inside
    // <plugins_dir>/<plugin>/viewer/.
    let canon_abs = match abs.canonicalize() {
        Ok(p) => p,
        Err(_) => return not_found(),
    };
    let canon_root = match canon_root.canonicalize() {
        Ok(p) => p,
        Err(_) => return not_found(),
    };
    if !canon_abs.starts_with(&canon_root) {
        return not_found();
    }

    let bytes = match std::fs::read(&canon_abs) {
        Ok(b) => b,
        Err(_) => return not_found(),
    };
    let mime = guess_mime(&canon_abs);
    tauri::http::Response::builder()
        .status(200)
        .header("Content-Type", mime)
        .body(bytes)
        .unwrap_or_else(|_| not_found())
}

fn not_found() -> tauri::http::Response<Vec<u8>> {
    tauri::http::Response::builder()
        .status(404)
        .body(b"not found".to_vec())
        .unwrap()
}

fn guess_mime(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("js") | Some("mjs") => "application/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("wasm") => "application/wasm",
        Some("txt") => "text/plain; charset=utf-8",
        Some("map") => "application/json; charset=utf-8",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tokio::sync::mpsc;
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
            .oneshot(Request::get("/plugin/p/status").body(Body::empty()).unwrap())
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
}
