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
