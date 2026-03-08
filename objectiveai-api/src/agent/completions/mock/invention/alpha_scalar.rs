use std::collections::HashMap;
use rand::Rng;
use super::super::client::{MockToolCall, random_string};
use crate::agent::completions::ResolvedTool;

/// Possible random input schemas for a scalar function.
/// Each is a valid `ScalarFunctionInputSchema` (= `ObjectInputSchema`) serialized as JSON.
const INPUT_SCHEMAS: &[&str] = &[
    // Simple text input
    r#"{"properties":{"text":{"type":"string","description":"The text to evaluate"}},"required":["text"]}"#,
    // Text with optional context
    r#"{"properties":{"text":{"type":"string","description":"The text to score"},"context":{"type":"string","description":"Additional context"}},"required":["text"]}"#,
    // Numeric input
    r#"{"properties":{"value":{"type":"number","description":"A numeric value"},"label":{"type":"string","description":"A label for the value"}},"required":["value","label"]}"#,
    // Object with nested fields
    r#"{"properties":{"title":{"type":"string","description":"Title of the item"},"body":{"type":"string","description":"Body content"},"tags":{"type":"array","items":{"type":"string"},"description":"Tags for classification"}},"required":["title","body"]}"#,
    // Boolean flag with text
    r#"{"properties":{"content":{"type":"string","description":"Content to analyze"},"is_draft":{"type":"boolean","description":"Whether this is a draft"}},"required":["content"]}"#,
    // Image input
    r#"{"properties":{"image":{"type":"image","description":"Image to evaluate"},"prompt":{"type":"string","description":"Evaluation prompt"}},"required":["image","prompt"]}"#,
];

/// Generate a mock tool call for the essay step of a scalar function.
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

/// Generate a mock tool call for the input_schema step of a scalar function.
///
/// If the chosen tool is `WriteInputSchema`, picks one of several predefined
/// `ScalarFunctionInputSchema` variants at random. If `ReadSpec`, `ReadEssay`,
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

/// Generate a mock tool call for the essay_tasks step of a scalar function.
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
