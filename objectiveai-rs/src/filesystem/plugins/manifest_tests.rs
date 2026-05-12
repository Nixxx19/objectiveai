use super::*;
use indexmap::IndexMap;

#[test]
fn manifest_minimal_roundtrip() {
    let m = Manifest {
        description: "tiny test plugin".to_string(),
        version: "0.1.0".to_string(),
        author: None,
        homepage: None,
        license: None,
        binaries: IndexMap::new(),
        viewer_zip: None,
        viewer_routes: vec![],
        mobile_ready: false,
    };
    let json = serde_json::to_value(&m).unwrap();
    // `skip_serializing_if = "Option::is_none"` keeps the wire shape lean.
    let obj = json.as_object().unwrap();
    assert_eq!(obj.len(), 2);
    assert_eq!(obj["description"], "tiny test plugin");
    assert_eq!(obj["version"], "0.1.0");
    // Roundtrip back.
    let back: Manifest = serde_json::from_value(json).unwrap();
    assert_eq!(back.description, "tiny test plugin");
    assert_eq!(back.version, "0.1.0");
    assert!(back.author.is_none());
    assert!(back.homepage.is_none());
    assert!(back.license.is_none());
    assert!(back.binaries.is_empty());
}

#[test]
fn manifest_full_roundtrip() {
    let m = Manifest {
        description: "Generate viral psyops content from a topic spec".to_string(),
        version: "0.3.1".to_string(),
        author: Some("Wiggidy".to_string()),
        homepage: Some("https://github.com/Wiggidy/psychological-operations".to_string()),
        license: Some("MIT".to_string()),
        binaries: IndexMap::new(),
        viewer_zip: None,
        viewer_routes: vec![],
        mobile_ready: false,
    };
    let json = serde_json::to_value(&m).unwrap();
    let back: Manifest = serde_json::from_value(json).unwrap();
    assert_eq!(back.description, m.description);
    assert_eq!(back.version, m.version);
    assert_eq!(back.author, m.author);
    assert_eq!(back.homepage, m.homepage);
    assert_eq!(back.license, m.license);
}

#[test]
fn manifest_with_name_and_source_field_order() {
    let m = ManifestWithNameAndSource {
        name: "psyops".to_string(),
        manifest: Manifest {
            description: "do things".to_string(),
            version: "1.2.3".to_string(),
            author: Some("Wiggidy".to_string()),
            homepage: None,
            license: Some("MIT".to_string()),
            binaries: IndexMap::new(),
            viewer_zip: None,
            viewer_routes: vec![],
            mobile_ready: false,
        },
        source: "/home/user/.objectiveai/plugins/psyops.manifest.json".to_string(),
    };
    let s = serde_json::to_string(&m).unwrap();
    // With preserve_order, name comes first, the flattened manifest
    // fields in declaration order, then source last. Optional `None`s
    // are skipped (homepage); empty `binaries` map is also skipped.
    let expected = concat!(
        r#"{"#,
        r#""name":"psyops","#,
        r#""description":"do things","#,
        r#""version":"1.2.3","#,
        r#""author":"Wiggidy","#,
        r#""license":"MIT","#,
        r#""source":"/home/user/.objectiveai/plugins/psyops.manifest.json""#,
        r#"}"#,
    );
    assert_eq!(s, expected);

    // Roundtrip back.
    let back: ManifestWithNameAndSource = serde_json::from_str(&s).unwrap();
    assert_eq!(back.name, "psyops");
    assert_eq!(back.manifest.description, "do things");
    assert_eq!(back.manifest.version, "1.2.3");
    assert_eq!(back.manifest.author.as_deref(), Some("Wiggidy"));
    assert!(back.manifest.homepage.is_none());
    assert_eq!(back.manifest.license.as_deref(), Some("MIT"));
    assert!(back.manifest.binaries.is_empty());
    assert_eq!(back.source, "/home/user/.objectiveai/plugins/psyops.manifest.json");
}

#[test]
fn manifest_deserializes_minimal_json() {
    let json = serde_json::json!({
        "description": "x",
        "version": "0.1.0"
    });
    let m: Manifest = serde_json::from_value(json).unwrap();
    assert_eq!(m.description, "x");
    assert_eq!(m.version, "0.1.0");
    assert!(m.author.is_none());
    assert!(m.homepage.is_none());
    assert!(m.license.is_none());
    assert!(m.binaries.is_empty());
}

fn full_binaries() -> IndexMap<Platform, String> {
    let mut bins = IndexMap::new();
    bins.insert(Platform::LinuxX86_64, "psyops-linux-x86_64".to_string());
    bins.insert(Platform::LinuxAarch64, "psyops-linux-aarch64".to_string());
    bins.insert(Platform::WindowsX86_64, "psyops-windows-x86_64.exe".to_string());
    bins.insert(Platform::MacosAarch64, "psyops-macos-aarch64".to_string());
    bins
}

#[test]
fn manifest_with_binaries_roundtrip() {
    let m = Manifest {
        description: "x".to_string(),
        version: "1.0.0".to_string(),
        author: None,
        homepage: None,
        license: None,
        binaries: full_binaries(),
        viewer_zip: None,
        viewer_routes: vec![],
        mobile_ready: false,
    };
    let json = serde_json::to_value(&m).unwrap();
    let back: Manifest = serde_json::from_value(json).unwrap();
    assert_eq!(back.binaries.len(), 4);
    assert_eq!(back.binaries[&Platform::LinuxX86_64], "psyops-linux-x86_64");
    assert_eq!(back.binaries[&Platform::LinuxAarch64], "psyops-linux-aarch64");
    assert_eq!(back.binaries[&Platform::WindowsX86_64], "psyops-windows-x86_64.exe");
    assert_eq!(back.binaries[&Platform::MacosAarch64], "psyops-macos-aarch64");
}

#[test]
fn manifest_omits_empty_binaries_field() {
    let m = Manifest {
        description: "x".to_string(),
        version: "1.0.0".to_string(),
        author: None,
        homepage: None,
        license: None,
        binaries: IndexMap::new(),
        viewer_zip: None,
        viewer_routes: vec![],
        mobile_ready: false,
    };
    let json = serde_json::to_value(&m).unwrap();
    let obj = json.as_object().unwrap();
    assert!(!obj.contains_key("binaries"), "empty map should be skipped, got {obj:?}");
}

#[test]
fn manifest_deserializes_without_binaries_field() {
    let json = serde_json::json!({
        "description": "x",
        "version": "1.0.0"
    });
    let m: Manifest = serde_json::from_value(json).unwrap();
    assert!(m.binaries.is_empty());
}

#[test]
fn manifest_with_binaries_field_order() {
    let m = Manifest {
        description: "x".to_string(),
        version: "1.0.0".to_string(),
        author: None,
        homepage: None,
        license: None,
        binaries: full_binaries(),
        viewer_zip: None,
        viewer_routes: vec![],
        mobile_ready: false,
    };
    let s = serde_json::to_string(&m).unwrap();
    // Insertion order: linux-x86_64, linux-aarch64, windows-x86_64,
    // macos-aarch64. preserve_order + IndexMap should keep it.
    let i_lx = s.find("linux-x86_64").unwrap();
    let i_la = s.find("linux-aarch64").unwrap();
    let i_wx = s.find("windows-x86_64").unwrap();
    let i_ma = s.find("macos-aarch64").unwrap();
    assert!(i_lx < i_la, "linux-x86_64 should come before linux-aarch64: {s}");
    assert!(i_la < i_wx, "linux-aarch64 should come before windows-x86_64: {s}");
    assert!(i_wx < i_ma, "windows-x86_64 should come before macos-aarch64: {s}");

    let back: Manifest = serde_json::from_str(&s).unwrap();
    let keys: Vec<&Platform> = back.binaries.keys().collect();
    assert_eq!(
        keys,
        vec![&Platform::LinuxX86_64, &Platform::LinuxAarch64, &Platform::WindowsX86_64, &Platform::MacosAarch64]
    );
}

#[test]
fn manifest_with_sparse_binaries_is_valid() {
    // Plugin that only ships a Linux x86_64 binary.
    let mut bins = IndexMap::new();
    bins.insert(Platform::LinuxX86_64, "psyops-linux-x86_64".to_string());
    let m = Manifest {
        description: "linux-only plugin".to_string(),
        version: "0.1.0".to_string(),
        author: None,
        homepage: None,
        license: None,
        binaries: bins,
        viewer_zip: None,
        viewer_routes: vec![],
        mobile_ready: false,
    };
    let s = serde_json::to_string(&m).unwrap();
    let back: Manifest = serde_json::from_str(&s).unwrap();
    assert_eq!(back.binaries.len(), 1);
    assert_eq!(back.binaries[&Platform::LinuxX86_64], "psyops-linux-x86_64");
    assert!(!back.binaries.contains_key(&Platform::LinuxAarch64));
    assert!(!back.binaries.contains_key(&Platform::WindowsX86_64));
    assert!(!back.binaries.contains_key(&Platform::WindowsAarch64));
    assert!(!back.binaries.contains_key(&Platform::MacosX86_64));
    assert!(!back.binaries.contains_key(&Platform::MacosAarch64));
}

#[test]
fn manifest_with_viewer_fields_roundtrip() {
    let m = Manifest {
        description: "viewer plugin".to_string(),
        version: "1.0.0".to_string(),
        author: None,
        homepage: None,
        license: None,
        binaries: IndexMap::new(),
        viewer_zip: Some("psyops-viewer.zip".to_string()),
        viewer_routes: vec![
            ViewerRoute {
                path: "/say".to_string(),
                method: HttpMethod::Post,
                r#type: "say_request".to_string(),
            },
            ViewerRoute {
                path: "/status".to_string(),
                method: HttpMethod::Get,
                r#type: "status_request".to_string(),
            },
        ],
        mobile_ready: true,
    };
    let json = serde_json::to_value(&m).unwrap();
    let back: Manifest = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(back.viewer_zip.as_deref(), Some("psyops-viewer.zip"));
    assert_eq!(back.viewer_routes.len(), 2);
    assert_eq!(back.viewer_routes[0].path, "/say");
    assert_eq!(back.viewer_routes[0].method, HttpMethod::Post);
    assert_eq!(back.viewer_routes[0].r#type, "say_request");
    assert_eq!(back.viewer_routes[1].method, HttpMethod::Get);
    assert!(back.mobile_ready);

    // The two viewer routes should serialize methods as uppercase strings.
    let routes_json = json.get("viewer_routes").unwrap();
    assert_eq!(routes_json[0]["method"], "POST");
    assert_eq!(routes_json[1]["method"], "GET");
}

#[test]
fn manifest_omits_viewer_fields_when_absent() {
    let m = Manifest {
        description: "x".to_string(),
        version: "1.0.0".to_string(),
        author: None,
        homepage: None,
        license: None,
        binaries: IndexMap::new(),
        viewer_zip: None,
        viewer_routes: vec![],
        mobile_ready: false,
    };
    let json = serde_json::to_value(&m).unwrap();
    let obj = json.as_object().unwrap();
    assert!(!obj.contains_key("viewer_zip"));
    assert!(!obj.contains_key("viewer_routes"));
    assert!(!obj.contains_key("mobile_ready"));
}

#[test]
fn manifest_deserializes_without_viewer_fields() {
    let json = serde_json::json!({
        "description": "x",
        "version": "1.0.0"
    });
    let m: Manifest = serde_json::from_value(json).unwrap();
    assert!(m.viewer_zip.is_none());
    assert!(m.viewer_routes.is_empty());
    assert!(!m.mobile_ready);
}

#[test]
fn http_method_serializes_uppercase() {
    let cases = [
        (HttpMethod::Get, "\"GET\""),
        (HttpMethod::Post, "\"POST\""),
        (HttpMethod::Put, "\"PUT\""),
        (HttpMethod::Patch, "\"PATCH\""),
        (HttpMethod::Delete, "\"DELETE\""),
    ];
    for (m, expected) in cases {
        let got = serde_json::to_string(&m).unwrap();
        assert_eq!(got, expected);
        let back: HttpMethod = serde_json::from_str(&got).unwrap();
        assert_eq!(back, m);
    }
}
