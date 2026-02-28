use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum JsonSchemaOutputFormatType {
    #[serde(rename = "json_schema")]
    JsonSchema,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct JsonSchemaOutputFormat {
    pub r#type: JsonSchemaOutputFormatType,
    pub schema: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum OutputFormat {
    JsonSchema(JsonSchemaOutputFormat),
}
