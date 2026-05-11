use super::*;

#[test]
fn manifest_minimal_roundtrip() {
    let m = Manifest {
        description: "tiny test plugin".to_string(),
        version: "0.1.0".to_string(),
        author: None,
        homepage: None,
        license: None,
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
}

#[test]
fn manifest_full_roundtrip() {
    let m = Manifest {
        description: "Generate viral psyops content from a topic spec".to_string(),
        version: "0.3.1".to_string(),
        author: Some("Wiggidy".to_string()),
        homepage: Some("https://github.com/Wiggidy/psychological-operations".to_string()),
        license: Some("MIT".to_string()),
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
        },
        source: "/home/user/.objectiveai/plugins/psyops.manifest.json".to_string(),
    };
    let s = serde_json::to_string(&m).unwrap();
    // With preserve_order, name comes first, the flattened manifest
    // fields in declaration order, then source last. Optional `None`s
    // are skipped (homepage).
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
}
