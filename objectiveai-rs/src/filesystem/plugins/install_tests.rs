use super::super::Client;
use super::{InstallError, Platform};
use indexmap::IndexMap;
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn temp_base() -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("oai-install-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn cleanup(d: &std::path::Path) {
    let _ = std::fs::remove_dir_all(d);
}

fn binary_filename() -> &'static str {
    if cfg!(windows) { "plugin.exe" } else { "plugin" }
}

/// Serialize `Platform::current()` to its kebab-case wire form for
/// constructing mock manifest bodies.
fn current_platform_key() -> String {
    let p = Platform::current().expect("test requires a supported host platform");
    serde_json::to_value(p)
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
}

fn client_for(base: &std::path::Path) -> Client {
    Client::new(Some(base.to_path_buf()), None::<&str>, None::<&str>)
}

const FAKE_BIN: &[u8] = b"FAKEBIN";

#[tokio::test]
async fn install_succeeds_when_platform_supported() {
    let base = temp_base();
    let server = MockServer::start().await;
    let platform_key = current_platform_key();

    let manifest_body = json!({
        "description": "test plugin",
        "version": "1.0.0",
        "binaries": {
            platform_key: "asset-bin"
        }
    });

    Mock::given(method("GET"))
        .and(path("/owner/repo/HEAD/objectiveai.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(manifest_body))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/owner/repo/releases/download/v1.0.0/asset-bin"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(FAKE_BIN.to_vec()))
        .mount(&server)
        .await;

    let client = client_for(&base);
    let result = client
        .install_plugin_at(&server.uri(), &server.uri(), "owner", "repo", None, None)
        .await;

    assert!(matches!(result, Ok(true)), "got {result:?}");

    let binary_path = base.join("plugins").join("repo").join(binary_filename());
    assert!(binary_path.exists(), "binary missing at {binary_path:?}");
    let bytes = std::fs::read(&binary_path).unwrap();
    assert_eq!(bytes, FAKE_BIN);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&binary_path).unwrap().permissions().mode();
        assert!(mode & 0o111 != 0, "binary not executable, mode={mode:o}");
    }

    cleanup(&base);
}

#[tokio::test]
async fn install_returns_false_when_platform_not_in_binaries() {
    let base = temp_base();
    let server = MockServer::start().await;

    let manifest_body = json!({
        "description": "test plugin",
        "version": "1.0.0"
        // no binaries field → empty map → current platform absent
    });

    Mock::given(method("GET"))
        .and(path("/owner/repo/HEAD/objectiveai.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(manifest_body))
        .mount(&server)
        .await;

    let client = client_for(&base);
    let result = client
        .install_plugin_at(&server.uri(), &server.uri(), "owner", "repo", None, None)
        .await;

    assert!(matches!(result, Ok(false)), "got {result:?}");
    // No binary was fetched → no plugin dir should have been created.
    assert!(
        !base.join("plugins").join("repo").exists(),
        "plugin dir should not exist when install returned false"
    );

    cleanup(&base);
}

#[tokio::test]
async fn install_uses_commit_sha_when_provided() {
    let base = temp_base();
    let server = MockServer::start().await;
    let platform_key = current_platform_key();

    let manifest_body = json!({
        "description": "test plugin",
        "version": "1.0.0",
        "binaries": { platform_key: "asset-bin" }
    });

    Mock::given(method("GET"))
        .and(path("/owner/repo/abc123/objectiveai.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(manifest_body))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/owner/repo/releases/download/v1.0.0/asset-bin"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(FAKE_BIN.to_vec()))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&base);
    let result = client
        .install_plugin_at(
            &server.uri(),
            &server.uri(),
            "owner",
            "repo",
            Some("abc123"),
            None,
        )
        .await;

    assert!(matches!(result, Ok(true)), "got {result:?}");
    // Wiremock verifies .expect(1) on drop — if HEAD had been requested
    // instead of abc123 the manifest mock would record 0 hits and
    // panic the test.

    cleanup(&base);
}

#[tokio::test]
async fn install_manifest_404_returns_manifest_bad_status_error() {
    let base = temp_base();
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/owner/repo/HEAD/objectiveai.json"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let client = client_for(&base);
    let result = client
        .install_plugin_at(&server.uri(), &server.uri(), "owner", "repo", None, None)
        .await;

    match result {
        Err(super::super::Error::Install(InstallError::ManifestBadStatus { code, .. })) => {
            assert_eq!(code.as_u16(), 404);
        }
        other => panic!("expected ManifestBadStatus(404), got {other:?}"),
    }

    cleanup(&base);
}

#[tokio::test]
async fn install_binary_404_returns_binary_bad_status_error() {
    let base = temp_base();
    let server = MockServer::start().await;
    let platform_key = current_platform_key();

    let manifest_body = json!({
        "description": "test plugin",
        "version": "1.0.0",
        "binaries": { platform_key: "asset-bin" }
    });

    Mock::given(method("GET"))
        .and(path("/owner/repo/HEAD/objectiveai.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(manifest_body))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/owner/repo/releases/download/v1.0.0/asset-bin"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let client = client_for(&base);
    let result = client
        .install_plugin_at(&server.uri(), &server.uri(), "owner", "repo", None, None)
        .await;

    match result {
        Err(super::super::Error::Install(InstallError::BinaryBadStatus { code, .. })) => {
            assert_eq!(code.as_u16(), 404);
        }
        other => panic!("expected BinaryBadStatus(404), got {other:?}"),
    }

    cleanup(&base);
}

#[tokio::test]
async fn install_malformed_manifest_returns_parse_error() {
    let base = temp_base();
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/owner/repo/HEAD/objectiveai.json"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"not json{{{".to_vec()))
        .mount(&server)
        .await;

    let client = client_for(&base);
    let result = client
        .install_plugin_at(&server.uri(), &server.uri(), "owner", "repo", None, None)
        .await;

    match result {
        Err(super::super::Error::Install(InstallError::ManifestParse(_))) => {}
        other => panic!("expected ManifestParse, got {other:?}"),
    }

    cleanup(&base);
}

#[tokio::test]
async fn fetch_plugin_manifest_returns_parsed_manifest() {
    let base = temp_base();
    let server = MockServer::start().await;
    let platform_key = current_platform_key();

    let manifest_body = json!({
        "description": "test plugin",
        "version": "1.2.3",
        "author": "Wiggidy",
        "binaries": { platform_key: "asset-bin" }
    });

    Mock::given(method("GET"))
        .and(path("/owner/repo/HEAD/objectiveai.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(manifest_body))
        .mount(&server)
        .await;

    let client = client_for(&base);
    let manifest = client
        .fetch_plugin_manifest_at(&server.uri(), "owner", "repo", None, None)
        .await
        .expect("expected Ok(Manifest)");

    assert_eq!(manifest.description, "test plugin");
    assert_eq!(manifest.version, "1.2.3");
    assert_eq!(manifest.author.as_deref(), Some("Wiggidy"));
    assert_eq!(manifest.binaries.len(), 1);

    cleanup(&base);
}

#[tokio::test]
async fn install_passes_headers_to_both_requests() {
    let base = temp_base();
    let server = MockServer::start().await;
    let platform_key = current_platform_key();

    let manifest_body = json!({
        "description": "test plugin",
        "version": "1.0.0",
        "binaries": { platform_key: "asset-bin" }
    });

    // Both mocks require the header — if the SDK doesn't forward it,
    // wiremock falls through and returns 404 (which would surface as
    // ManifestBadStatus / BinaryBadStatus).
    Mock::given(method("GET"))
        .and(path("/owner/repo/HEAD/objectiveai.json"))
        .and(header("authorization", "token abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(manifest_body))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/owner/repo/releases/download/v1.0.0/asset-bin"))
        .and(header("authorization", "token abc"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(FAKE_BIN.to_vec()))
        .mount(&server)
        .await;

    let mut headers = IndexMap::new();
    headers.insert("Authorization".to_string(), "token abc".to_string());

    let client = client_for(&base);
    let result = client
        .install_plugin_at(
            &server.uri(),
            &server.uri(),
            "owner",
            "repo",
            None,
            Some(&headers),
        )
        .await;

    assert!(matches!(result, Ok(true)), "got {result:?}");

    cleanup(&base);
}
