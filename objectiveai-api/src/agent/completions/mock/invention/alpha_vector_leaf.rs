use std::collections::HashMap;
use rand::Rng;
use super::super::client::MockToolCall;
use crate::agent::completions::ResolvedTool;

/// Generate a mock tool call for the tasks step of a vector leaf function.
///
/// Vector leaf tasks are `VectorCompletion` task expressions with `messages`
/// (a Starlark expression) and `responses` (a Starlark expression producing
/// an array derived from input items).
///
/// The `input_schema_json` is the serialized `VectorFunctionInputSchema`
/// obtained by calling `ReadInputSchema`. It can be ANY valid vector input
/// schema — items of any type, arbitrary item object depth, optional context
/// with any structure, etc.
pub async fn tasks_tool_call(
    input_schema_json: &str,
    tool_names: &[String],
    tool_map: &HashMap<String, ResolvedTool>,
    rng: &mut impl Rng,
) -> MockToolCall {
    let tool_name = super::pick_invention_tool("AppendTask", tool_names, tool_map, rng).await;
    let arguments = match tool_name {
        "AppendTask" => {
            let item_fields = super::extract_vector_item_fields(input_schema_json);
            let context_fields = super::extract_vector_context_fields(input_schema_json);

            // Messages: use context fields if available, otherwise a static prompt
            let messages_expr = if !context_fields.is_empty() {
                super::build_messages_expr(&context_fields, rng)
            } else {
                // No context — use a static ranking prompt
                let prompts = [
                    "Rank the following items",
                    "Which of the following is best?",
                    "Order these by quality",
                    "Compare and rank these",
                ];
                let prompt = prompts[rng.random_range(0..prompts.len())];
                format!(
                    r#"[{{"role": "user", "content": [{{"type": "text", "text": "{prompt}"}}]}}]"#,
                )
            };

            // Responses: map items to content part arrays
            let responses_expr = super::build_responses_expr(&item_fields, rng);

            serde_json::json!({
                "type": "vector.completion",
                "messages": { "$starlark": messages_expr },
                "responses": { "$starlark": responses_expr },
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
