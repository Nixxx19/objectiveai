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
/// obtained by calling `ReadInputSchema`, used to derive realistic expressions
/// referencing item fields.
pub fn tasks_tool_call(
    input_schema_json: &str,
    tool_names: &[String],
    tool_map: &HashMap<String, ResolvedTool>,
    rng: &mut impl Rng,
) -> MockToolCall {
    let tool_name = super::pick_invention_tool("AppendTask", tool_names, tool_map, rng);
    let arguments = match tool_name {
        "AppendTask" => {
            let item_fields = extract_item_fields(input_schema_json);
            let has_context = has_context_field(input_schema_json);
            let messages_expr = random_messages_expr(&item_fields, has_context, rng);
            let responses_expr = random_responses_expr(&item_fields, rng);
            serde_json::json!({
                "vector.completion": {
                    "messages": { "$starlark": messages_expr },
                    "responses": { "$starlark": responses_expr },
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

/// Extract property names from the `items` schema within a VectorFunctionInputSchema.
/// Falls back to treating items as plain strings.
fn extract_item_fields(input_schema_json: &str) -> Vec<String> {
    serde_json::from_str::<serde_json::Value>(input_schema_json)
        .ok()
        .and_then(|v| {
            let items = v.get("items")?;
            // items is an InputSchema; if it's an object type with properties, extract them
            items.get("items")
                .and_then(|inner| inner.get("properties"))
                .and_then(|p| p.as_object())
                .map(|o| o.keys().cloned().collect())
        })
        .unwrap_or_default()
}

/// Check if the VectorFunctionInputSchema has a `context` field.
fn has_context_field(input_schema_json: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(input_schema_json)
        .ok()
        .and_then(|v| v.get("context").cloned())
        .is_some()
}

/// Generate a random Starlark `messages` expression for a vector leaf task.
fn random_messages_expr(
    item_fields: &[String],
    has_context: bool,
    rng: &mut impl Rng,
) -> String {
    if has_context {
        let templates = [
            r#"[{"role": "user", "content": [{"type": "text", "text": str(input['context'])}]}]"#,
            r#"[{"role": "user", "content": [{"type": "text", "text": "Given context: " + str(input['context'])}]}]"#,
        ];
        templates[rng.random_range(0..templates.len())].to_string()
    } else if !item_fields.is_empty() {
        format!(
            r#"[{{"role": "user", "content": [{{"type": "text", "text": "Rank the following items"}}]}}]"#,
        )
    } else {
        r#"[{"role": "user", "content": [{"type": "text", "text": "Which of the following is best?"}]}]"#.to_string()
    }
}

/// Generate a random Starlark `responses` expression for a vector leaf task.
/// Items are turned into response arrays of content parts.
fn random_responses_expr(item_fields: &[String], rng: &mut impl Rng) -> String {
    if !item_fields.is_empty() {
        let field = &item_fields[rng.random_range(0..item_fields.len())];
        format!(
            r#"[[{{"type": "text", "text": str(item['{field}'])}}] for item in input['items']]"#,
        )
    } else {
        // Items are plain strings
        r#"[[{"type": "text", "text": str(item)}] for item in input['items']]"#.to_string()
    }
}
