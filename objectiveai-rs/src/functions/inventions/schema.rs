use crate::json_schema::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct IndexObject {
    pub index: u64,
}

impl JsonSchema for IndexObject {
    fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        let mut index_prop = serde_json::Map::with_capacity(1);
        index_prop.insert(
            "type".to_string(),
            serde_json::Value::String("integer".to_string()),
        );

        let mut properties = serde_json::Map::with_capacity(1);
        properties.insert(
            "index".to_string(),
            serde_json::Value::Object(index_prop),
        );

        let mut map = serde_json::Map::with_capacity(4);
        map.insert(
            "type".to_string(),
            serde_json::Value::String("object".to_string()),
        );
        map.insert(
            "properties".to_string(),
            serde_json::Value::Object(properties),
        );
        map.insert(
            "required".to_string(),
            serde_json::Value::Array(vec![serde_json::Value::String(
                "index".to_string(),
            )]),
        );
        map.insert(
            "additionalProperties".to_string(),
            serde_json::Value::Bool(false),
        );
        map
    }
}

#[derive(Deserialize)]
pub struct EssayObject {
    pub essay: String,
}

impl JsonSchema for EssayObject {
    fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        let mut essay_prop = serde_json::Map::with_capacity(1);
        essay_prop.insert(
            "type".to_string(),
            serde_json::Value::String("string".to_string()),
        );

        let mut properties = serde_json::Map::with_capacity(1);
        properties.insert(
            "essay".to_string(),
            serde_json::Value::Object(essay_prop),
        );

        let mut map = serde_json::Map::with_capacity(4);
        map.insert(
            "type".to_string(),
            serde_json::Value::String("object".to_string()),
        );
        map.insert(
            "properties".to_string(),
            serde_json::Value::Object(properties),
        );
        map.insert(
            "required".to_string(),
            serde_json::Value::Array(vec![serde_json::Value::String(
                "essay".to_string(),
            )]),
        );
        map.insert(
            "additionalProperties".to_string(),
            serde_json::Value::Bool(false),
        );
        map
    }
}

#[derive(Deserialize)]
pub struct EssayTasksObject {
    pub essay_tasks: String,
}

impl JsonSchema for EssayTasksObject {
    fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        let mut essay_tasks_prop = serde_json::Map::with_capacity(1);
        essay_tasks_prop.insert(
            "type".to_string(),
            serde_json::Value::String("string".to_string()),
        );

        let mut properties = serde_json::Map::with_capacity(1);
        properties.insert(
            "essay_tasks".to_string(),
            serde_json::Value::Object(essay_tasks_prop),
        );

        let mut map = serde_json::Map::with_capacity(4);
        map.insert(
            "type".to_string(),
            serde_json::Value::String("object".to_string()),
        );
        map.insert(
            "properties".to_string(),
            serde_json::Value::Object(properties),
        );
        map.insert(
            "required".to_string(),
            serde_json::Value::Array(vec![serde_json::Value::String(
                "essay_tasks".to_string(),
            )]),
        );
        map.insert(
            "additionalProperties".to_string(),
            serde_json::Value::Bool(false),
        );
        map
    }
}

#[derive(Deserialize)]
pub struct DescriptionObject {
    pub description: String,
}

impl JsonSchema for DescriptionObject {
    fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        let mut description_prop = serde_json::Map::with_capacity(1);
        description_prop.insert(
            "type".to_string(),
            serde_json::Value::String("string".to_string()),
        );

        let mut properties = serde_json::Map::with_capacity(1);
        properties.insert(
            "description".to_string(),
            serde_json::Value::Object(description_prop),
        );

        let mut map = serde_json::Map::with_capacity(4);
        map.insert(
            "type".to_string(),
            serde_json::Value::String("object".to_string()),
        );
        map.insert(
            "properties".to_string(),
            serde_json::Value::Object(properties),
        );
        map.insert(
            "required".to_string(),
            serde_json::Value::Array(vec![serde_json::Value::String(
                "description".to_string(),
            )]),
        );
        map.insert(
            "additionalProperties".to_string(),
            serde_json::Value::Bool(false),
        );
        map
    }
}
