use super::*;

#[test]
fn test_boolean_roundtrip() {
    let schema = JsonSchema::Boolean(BooleanJsonSchema);
    let json = serde_json::to_string(&schema).unwrap();
    assert_eq!(json, r#"{"type":"boolean"}"#);
    let back: JsonSchema = serde_json::from_str(&json).unwrap();
    assert_eq!(back, schema);
}
