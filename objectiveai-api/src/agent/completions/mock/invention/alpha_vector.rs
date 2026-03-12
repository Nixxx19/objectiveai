use std::collections::HashMap;
use rand::Rng;
use super::super::client::{MockToolCall, random_string};
use crate::agent::completions::ResolvedTool;


/// Generate a mock tool call for the essay step of a vector function.
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
/// If the chosen tool is `WriteInputSchema`, generates a random
/// `VectorFunctionInputSchema` with diverse item/context structures.
pub fn input_schema_tool_call(
    tool_names: &[String],
    tool_map: &HashMap<String, ResolvedTool>,
    rng: &mut impl Rng,
) -> MockToolCall {
    let tool_name = super::pick_invention_tool("WriteInputSchema", tool_names, tool_map, rng);
    let arguments = match tool_name {
        "WriteInputSchema" => {
            super::schema_gen::random_vector_input_schema(rng)
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
