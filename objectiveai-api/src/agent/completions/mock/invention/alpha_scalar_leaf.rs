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
/// obtained by calling the `ReadInputSchema` invention tool, used to derive
/// realistic `messages` expressions referencing input fields.
pub fn tasks_tool_call(
    input_schema_json: &str,
    tool_names: &[String],
    tool_map: &HashMap<String, ResolvedTool>,
    rng: &mut impl Rng,
) -> MockToolCall {
    let tool_name = super::pick_invention_tool("AppendTask", tool_names, tool_map, rng);
    let arguments = match tool_name {
        "AppendTask" => {
            let fields = super::extract_input_fields(input_schema_json);
            let messages_expr = random_messages_expr(&fields, rng);
            let n_responses = rng.random_range(2u32..=5) as usize;
            let responses: Vec<serde_json::Value> = (0..n_responses)
                .map(|_| {
                    serde_json::json!([{"type": "text", "text": random_string(rng, 5, 40)}])
                })
                .collect();
            serde_json::json!({
                "vector.completion": {
                    "messages": { "$starlark": messages_expr },
                    "responses": responses,
                }
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

/// Generate a random Starlark `messages` expression referencing input fields.
fn random_messages_expr(fields: &[String], rng: &mut impl Rng) -> String {
    let field = &fields[rng.random_range(0..fields.len())];
    let templates = [
        format!(r#"[{{"role": "user", "content": [{{"type": "text", "text": "Evaluate: " + str(input['{field}'])}}]}}]"#),
        format!(r#"[{{"role": "user", "content": [{{"type": "text", "text": str(input['{field}'])}}]}}]"#),
        format!(r#"[{{"role": "user", "content": [{{"type": "text", "text": "Rate the following: " + str(input['{field}'])}}]}}]"#),
    ];
    templates[rng.random_range(0..templates.len())].clone()
}
