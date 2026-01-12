use crate::chat::completions::response;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolCall {
    Function {
        id: String,
        function: ToolCallFunction,
    },
}

impl Default for ToolCall {
    fn default() -> Self {
        ToolCall::Function {
            id: String::new(),
            function: ToolCallFunction::default(),
        }
    }
}

impl From<response::streaming::ToolCall> for ToolCall {
    fn from(
        response::streaming::ToolCall {
            id,
            function,
            r#type,
            ..
        }: response::streaming::ToolCall,
    ) -> Self {
        match r#type {
            Some(response::streaming::ToolCallType::Function) | None => {
                ToolCall::Function {
                    id: id.unwrap_or_default(),
                    function: function
                        .map(ToolCallFunction::from)
                        .unwrap_or_default(),
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

impl From<response::streaming::ToolCallFunction> for ToolCallFunction {
    fn from(
        response::streaming::ToolCallFunction {
            name,
            arguments,
        } : response::streaming::ToolCallFunction,
    ) -> Self {
        Self {
            name: name.unwrap_or_default(),
            arguments: arguments.unwrap_or_default(),
        }
    }
}
