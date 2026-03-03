//! Tool/function definitions for chat completions.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// A tool that can be called by the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Tool {
    /// A function tool.
    Function { function: FunctionTool },
}

impl Tool {
    /// Creates a Tool from an MCP tool definition.
    pub fn new_from_mcp(tool: &crate::mcp::tool::Tool) -> Self {
        let mut map = IndexMap::new();
        map.insert(
            "type".to_string(),
            serde_json::Value::String("object".to_string()),
        );
        if let Some(props) = &tool.input_schema.properties {
            map.insert(
                "properties".to_string(),
                serde_json::Value::Object(
                    props.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                ),
            );
        }
        if let Some(req) = &tool.input_schema.required {
            map.insert(
                "required".to_string(),
                serde_json::Value::Array(
                    req.iter()
                        .map(|s| serde_json::Value::String(s.clone()))
                        .collect(),
                ),
            );
        }
        for (k, v) in &tool.input_schema.extra {
            map.insert(k.clone(), v.clone());
        }

        Self::Function {
            function: FunctionTool {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters: Some(map),
                strict: None,
            },
        }
    }

    /// Creates a Tool from an invention tool definition.
    pub fn new_from_invention(
        tool: &objectiveai::functions::inventions::InventionTool,
    ) -> Self {
        Self::Function {
            function: FunctionTool {
                name: tool.name.to_string(),
                description: Some(tool.description.to_string()),
                parameters: Some(tool.parameters.clone()),
                strict: None,
            },
        }
    }
}

/// A function tool definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionTool {
    /// The name of the function.
    pub name: String,
    /// A description of what the function does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema for the function parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<IndexMap<String, serde_json::Value>>,
    /// Whether to enforce strict schema validation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}
