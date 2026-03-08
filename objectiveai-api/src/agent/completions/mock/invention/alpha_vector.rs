use std::collections::HashMap;
use rand::Rng;
use super::super::client::{MockToolCall, random_string};
use crate::agent::completions::ResolvedTool;

/// Possible random input schemas for a vector function.
/// Each is a valid `VectorFunctionInputSchema` serialized as JSON
/// (has `items` and optional `context`).
const INPUT_SCHEMAS: &[&str] = &[
    // Simple text items, no context
    r#"{"items":{"type":"array","items":{"type":"string","description":"An item to rank"}}}"#,
    // Text items with context
    r#"{"context":{"properties":{"query":{"type":"string","description":"Search query"}},"required":["query"]},"items":{"type":"array","items":{"type":"string","description":"A candidate result"}}}"#,
    // Object items, no context
    r#"{"items":{"type":"array","items":{"type":"object","properties":{"title":{"type":"string"},"body":{"type":"string"}},"required":["title","body"]}}}"#,
    // Object items with context
    r#"{"context":{"properties":{"topic":{"type":"string","description":"Topic for ranking"},"criteria":{"type":"string","description":"Ranking criteria"}},"required":["topic"]},"items":{"type":"array","items":{"type":"object","properties":{"name":{"type":"string"},"description":{"type":"string"}},"required":["name"]}}}"#,
    // Image items with text context
    r#"{"context":{"properties":{"prompt":{"type":"string","description":"Evaluation prompt"}},"required":["prompt"]},"items":{"type":"array","items":{"type":"image","description":"An image to rank"}}}"#,
    // Simple string items with multi-field context
    r#"{"context":{"properties":{"category":{"type":"string"},"max_score":{"type":"number"}},"required":["category"]},"items":{"type":"array","items":{"type":"string"}}}"#,
];

/// Generate a mock tool call for the essay step of a vector function.
///
/// Picks a random tool from the available tools. If the chosen tool is an
/// invention tool (`WriteEssay`, `ReadSpec`), generates appropriate arguments.
/// Otherwise falls back to schema-based argument generation.
pub fn essay_tool_call(
    tool_names: &[String],
    tool_map: &HashMap<String, ResolvedTool>,
    rng: &mut impl Rng,
) -> MockToolCall {
    let tool_name = super::pick_invention_tool("WriteEssay", tool_names, tool_map, rng);
    let arguments = match tool_name {
        "WriteEssay" => {
            let essay = random_string(rng, 200, 800);
            serde_json::json!({ "essay": essay }).to_string()
        }
        "ReadSpec" => "{}".to_string(),
        _ => "{}".to_string(),
    };
    MockToolCall {
        tool_name: tool_name.to_string(),
        call_id: format!("call_mock_{}", rng.random_range(0u64..u64::MAX)),
        arguments,
        n_deltas: rng.random_range(1u32..=5) as usize,
    }
}

/// Generate a mock tool call for the input_schema step of a vector function.
///
/// If the chosen tool is `WriteInputSchema`, picks one of several predefined
/// `VectorFunctionInputSchema` variants at random. If `ReadSpec`, `ReadEssay`,
/// or a schema tool, generates appropriate arguments.
pub fn input_schema_tool_call(
    tool_names: &[String],
    tool_map: &HashMap<String, ResolvedTool>,
    rng: &mut impl Rng,
) -> MockToolCall {
    let tool_name = super::pick_invention_tool("WriteInputSchema", tool_names, tool_map, rng);
    let arguments = match tool_name {
        "WriteInputSchema" => {
            let idx = rng.random_range(0..INPUT_SCHEMAS.len());
            INPUT_SCHEMAS[idx].to_string()
        }
        "ReadSpec" | "ReadEssay" | "ReadInputSchema" => "{}".to_string(),
        _ => "{}".to_string(),
    };
    MockToolCall {
        tool_name: tool_name.to_string(),
        call_id: format!("call_mock_{}", rng.random_range(0u64..u64::MAX)),
        arguments,
        n_deltas: rng.random_range(1u32..=5) as usize,
    }
}

/// Generate a mock tool call for the essay_tasks step of a vector function.
///
/// If the chosen tool is `WriteEssayTasks`, generates a random essay tasks
/// string. Read tools get empty arguments. Other tools use schema-based
/// generation.
pub fn essay_tasks_tool_call(
    tool_names: &[String],
    tool_map: &HashMap<String, ResolvedTool>,
    rng: &mut impl Rng,
) -> MockToolCall {
    let tool_name = super::pick_invention_tool("WriteEssayTasks", tool_names, tool_map, rng);
    let arguments = match tool_name {
        "WriteEssayTasks" => {
            let essay_tasks = random_string(rng, 100, 500);
            serde_json::json!({ "essay_tasks": essay_tasks }).to_string()
        }
        "ReadSpec" | "ReadEssay" | "ReadInputSchema" => "{}".to_string(),
        _ => "{}".to_string(),
    };
    MockToolCall {
        tool_name: tool_name.to_string(),
        call_id: format!("call_mock_{}", rng.random_range(0u64..u64::MAX)),
        arguments,
        n_deltas: rng.random_range(1u32..=5) as usize,
    }
}
