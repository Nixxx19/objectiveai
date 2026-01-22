#[derive(Debug, serde::Deserialize)]
pub struct ResponseKey {
    pub _think: Option<String>,
    #[allow(dead_code)]
    response_key: String,
}

impl ResponseKey {
    fn schema(
        vector_response_keys: Vec<String>,
        think: bool,
    ) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::with_capacity(4);
        map.insert(
            "type".to_string(),
            serde_json::Value::String("object".to_string()),
        );
        map.insert(
            "properties".to_string(),
            serde_json::Value::Object({
                let mut properties = serde_json::Map::with_capacity(if think { 2 } else { 1 });
                if think {
                    properties.insert(
                        "_think".to_string(),
                        serde_json::json!({
                            "type": "string",
                            "description": "The assistant's internal reasoning.",
                        }),
                    );
                }
                properties.insert(
                    "response_key".to_string(),
                    serde_json::json!({
                        "type": "string",
                        "enum": vector_response_keys
                    }),
                );
                properties
            }),
        );
        map.insert(
            "required".to_string(),
            serde_json::Value::Array({
                let mut required = Vec::with_capacity(if think { 2 } else { 1 });
                if think {
                    required.push(serde_json::Value::String("_think".to_string()));
                }
                required.push(serde_json::Value::String("response_key".to_string()));
                required
            }),
        );
        map.insert(
            "additionalProperties".to_string(),
            serde_json::Value::Bool(false),
        );
        map
    }

    pub fn response_format(
        vector_response_keys: Vec<String>,
        think: bool,
    ) -> objectiveai::chat::completions::request::ResponseFormat {
        objectiveai::chat::completions::request::ResponseFormat::JsonSchema {
            json_schema: objectiveai::chat::completions::request::JsonSchema {
                name: "response_key".to_string(),
                description: None,
                strict: Some(true),
                schema: Some(serde_json::Value::Object(Self::schema(
                    vector_response_keys,
                    think,
                ))),
            },
        }
    }

    pub fn tool(
        vector_response_keys: Vec<String>,
        think: bool,
    ) -> objectiveai::chat::completions::request::Tool {
        objectiveai::chat::completions::request::Tool::Function {
            function: objectiveai::chat::completions::request::FunctionTool {
                name: "response_key".to_string(),
                description: None,
                strict: Some(true),
                parameters: Some(Self::schema(vector_response_keys, think)),
            },
        }
    }

    pub fn tool_choice() -> objectiveai::chat::completions::request::ToolChoice {
        objectiveai::chat::completions::request::ToolChoice::Function(
            objectiveai::chat::completions::request::ToolChoiceFunction::Function {
                function: objectiveai::chat::completions::request::ToolChoiceFunctionFunction {
                    name: "response_key".to_string(),
                },
            },
        )
    }
}
