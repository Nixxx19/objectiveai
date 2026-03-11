use indexmap::IndexMap;

pub trait JsonSchema {
    fn json_schema() -> serde_json::Map<String, serde_json::Value>;
    fn indexmap_json_schema() -> IndexMap<String, serde_json::Value> {
        Self::json_schema().into_iter().collect()
    }
}

#[derive(schemars::JsonSchema)]
#[schemars(rename = "EmptyObjectJsonSchema")]
pub struct EmptyObjectJsonSchema;

impl JsonSchema for EmptyObjectJsonSchema {
    fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        map.insert(
            "type".to_string(),
            serde_json::Value::String("object".to_string()),
        );
        map.insert(
            "additionalProperties".to_string(),
            serde_json::Value::Bool(false),
        );
        map
    }
}

#[derive(schemars::JsonSchema)]
#[schemars(rename = "AnyObjectJsonSchema")]
pub struct AnyObjectJsonSchema;

impl JsonSchema for AnyObjectJsonSchema {
    fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        map.insert(
            "type".to_string(),
            serde_json::Value::String("object".to_string()),
        );
        map
    }
}
