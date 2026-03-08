use std::collections::HashMap;
use rand::Rng;
use super::super::client::{MockToolCall, random_string};
use crate::agent::completions::ResolvedTool;

/// Generate a mock tool call for the tasks step of a scalar leaf function.
///
/// Scalar leaf tasks are `VectorCompletion` task expressions with `messages`
/// (a Starlark expression) and `responses` (an array of text content parts).
///
/// The `input_schema_json` is the serialized `ScalarFunctionInputSchema`
/// obtained by calling the `ReadInputSchema` invention tool. It can be ANY
/// valid ObjectInputSchema — arbitrary depth, arbitrary types, enums, etc.
/// The generated expressions must correctly reference the schema's fields.
pub async fn tasks_tool_call(
    input_schema_json: &str,
    tool_names: &[String],
    tool_map: &HashMap<String, ResolvedTool>,
    rng: &mut impl Rng,
) -> MockToolCall {
    let tool_name = super::pick_invention_tool("AppendTask", tool_names, tool_map, rng).await;
    let arguments = match tool_name {
        "AppendTask" => {
            let _ = input_schema_json; // schema not needed; messages use str(input) + static media
            let messages_expr = super::build_messages_expr("input");
            let n_responses = rng.random_range(2u32..=5) as usize;
            let responses: Vec<serde_json::Value> = (0..n_responses)
                .map(|_| {
                    serde_json::json!([{"type": "text", "text": random_string(rng, 5, 40)}])
                })
                .collect();
            serde_json::json!({
                "type": "vector.completion",
                "messages": { "$starlark": messages_expr },
                "responses": responses,
            }).to_string()
        }
        "CheckFunction" | "ReadSpec" | "ReadEssay" | "ReadInputSchema"
        | "ReadEssayTasks" | "ReadTasksLength" => "{}".to_string(),
        "DeleteTask" | "ReadTask" => {
            serde_json::json!({ "index": rng.random_range(0u32..5) }).to_string()
        }
        _ => "{}".to_string(),
    };
    MockToolCall {
        tool_name: tool_name.to_string(),
        call_id: format!("call_mock_{}", rng.random_range(0u64..u64::MAX)),
        arguments,
        n_deltas: rng.random_range(1u32..=5) as usize,
    }
}
