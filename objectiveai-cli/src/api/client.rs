//! Builds the SDK `HttpClient` for endpoint subcommands.
//!
//! Precedence for every field is `env var → config file → SDK default`.
//! The cli disabled the SDK's `env` feature, so this module re-implements
//! the env-var lookup itself and chains it with the on-disk `ApiConfig`
//! / `ViewerConfig` before handing the resolved values to
//! `objectiveai_sdk::HttpClient::new`.

/// Compose `(addr, port)` into a base URL, ensuring an explicit scheme.
///
/// Rule: if the address already starts with `http://` or `https://`, leave
/// it untouched. Otherwise prefix `http://` — regardless of whether the
/// host is an IPv4, IPv6, or hostname.
fn compose_url(addr: Option<&str>, port: Option<u16>) -> Option<String> {
    fn ensure_scheme(a: &str) -> String {
        if a.starts_with("http://") || a.starts_with("https://") {
            a.to_string()
        } else {
            format!("http://{a}")
        }
    }
    match (addr, port) {
        (Some(a), Some(p)) => Some(format!("{}:{p}", ensure_scheme(a).trim_end_matches('/'))),
        (Some(a), None) => Some(ensure_scheme(a)),
        (None, Some(p)) => Some(format!("http://127.0.0.1:{p}")),
        (None, None) => None,
    }
}

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

pub fn build_http_client(
    config: &mut objectiveai_sdk::filesystem::config::Config,
) -> objectiveai_sdk::HttpClient {
    // Snapshot every field up-front so we don't hold overlapping
    // `&mut` borrows of `config` across the SDK call.
    let address = env("OBJECTIVEAI_ADDRESS").or_else(|| {
        let api = config.api();
        compose_url(api.get_address(), api.get_port())
    });

    let authorization = env("OBJECTIVEAI_AUTHORIZATION").or_else(|| {
        config
            .api()
            .headers()
            .get_x_objectiveai_authorization()
            .map(String::from)
    });

    let user_agent = env("USER_AGENT")
        .or_else(|| config.api().headers().get_user_agent().map(String::from));

    let x_title = env("X_TITLE")
        .or_else(|| config.api().headers().get_x_title().map(String::from));

    let http_referer = env("HTTP_REFERER")
        .or_else(|| config.api().headers().get_http_referer().map(String::from));

    let x_github_authorization = env("GITHUB_AUTHORIZATION").or_else(|| {
        config
            .api()
            .headers()
            .get_x_github_authorization()
            .map(String::from)
    });

    let x_openrouter_authorization = env("OPENROUTER_AUTHORIZATION").or_else(|| {
        config
            .api()
            .headers()
            .get_x_openrouter_authorization()
            .map(String::from)
    });

    let x_mcp_authorization: Option<std::collections::HashMap<String, String>> =
        env("MCP_AUTHORIZATION")
            .and_then(|v| serde_json::from_str(&v).ok())
            .or_else(|| {
                config
                    .api()
                    .headers()
                    .get_x_mcp_authorization()
                    .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            });

    let x_viewer_signature = env("VIEWER_SIGNATURE")
        .or_else(|| config.viewer().get_signature().map(String::from))
        .or_else(|| {
            config
                .api()
                .headers()
                .get_x_viewer_signature()
                .map(String::from)
        });

    let x_viewer_address = env("VIEWER_ADDRESS")
        .or_else(|| {
            let viewer = config.viewer();
            compose_url(viewer.get_address(), viewer.get_port())
        })
        .or_else(|| {
            config
                .api()
                .headers()
                .get_x_viewer_address()
                .map(String::from)
        });

    let x_commit_author_name = env("COMMIT_AUTHOR_NAME").or_else(|| {
        config
            .api()
            .headers()
            .get_x_commit_author_name()
            .map(String::from)
    });

    let x_commit_author_email = env("COMMIT_AUTHOR_EMAIL").or_else(|| {
        config
            .api()
            .headers()
            .get_x_commit_author_email()
            .map(String::from)
    });

    objectiveai_sdk::HttpClient::new(
        reqwest::Client::new(),
        address,
        authorization,
        user_agent,
        x_title,
        http_referer,
        x_github_authorization,
        x_openrouter_authorization,
        x_mcp_authorization,
        x_viewer_signature,
        x_viewer_address,
        x_commit_author_name,
        x_commit_author_email,
    )
}

#[cfg(test)]
mod tests {
    use super::compose_url;

    #[test]
    fn compose_url_cases() {
        assert_eq!(compose_url(None, None), None);
        assert_eq!(
            compose_url(Some("127.0.0.1"), Some(8080)).as_deref(),
            Some("http://127.0.0.1:8080"),
        );
        assert_eq!(
            compose_url(Some("::1"), Some(8080)).as_deref(),
            Some("http://::1:8080"),
        );
        assert_eq!(
            compose_url(Some("::1"), None).as_deref(),
            Some("http://::1"),
        );
        assert_eq!(
            compose_url(Some("api.x"), None).as_deref(),
            Some("http://api.x"),
        );
        assert_eq!(
            compose_url(Some("https://api.x"), None).as_deref(),
            Some("https://api.x"),
        );
        assert_eq!(
            compose_url(Some("https://api.x"), Some(443)).as_deref(),
            Some("https://api.x:443"),
        );
        assert_eq!(
            compose_url(Some("https://api.x/"), Some(443)).as_deref(),
            Some("https://api.x:443"),
        );
        assert_eq!(
            compose_url(None, Some(8080)).as_deref(),
            Some("http://127.0.0.1:8080"),
        );
    }
}
