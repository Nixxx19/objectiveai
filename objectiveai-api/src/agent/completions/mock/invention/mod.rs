pub(super) mod alpha_scalar;
pub(super) mod alpha_scalar_branch;
pub(super) mod alpha_scalar_leaf;
pub(super) mod alpha_vector;
pub(super) mod alpha_vector_branch;
pub(super) mod alpha_vector_leaf;
mod route;
mod schema_gen;

pub use route::*;

/// Pick an invention tool with weighted selection.
///
/// 50% chance: return the key write tool for this step.
/// 50% chance: pick randomly from `tool_names`, re-rolling non-invention tools.
pub(super) async fn pick_invention_tool<'a>(
    key_tool: &str,
    tool_names: &'a [String],
    tool_map: &std::collections::HashMap<String, crate::agent::completions::ResolvedTool>,
    rng: &mut impl rand::Rng,
) -> &'a str {
    // 50% chance: return the key tool
    if rng.random_range(0u32..2) == 0 {
        if let Some(t) = tool_names.iter().find(|t| t.as_str() == key_tool) {
            return t.as_str();
        }
    }
    // 50% chance: pick randomly, re-rolling non-invention tools
    for attempt in 0u32.. {
        if attempt > 0 && attempt % 100 == 0 {
            // Yield so the tokio runtime can make progress (fire timeouts, etc.)
            tokio::task::yield_now().await;
        }
        let name = &tool_names[rng.random_range(0..tool_names.len())];
        if matches!(tool_map.get(name.as_str()), Some(crate::agent::completions::ResolvedTool::InventionTool(_))) {
            return name.as_str();
        }
    }
    unreachable!()
}

/// Generate a mock tool call for the description step (shared across all routes).
///
/// Tools: `[ReadSpec, ReadEssay, ReadInputSchema, ReadEssayTasks, ReadTask,
/// ReadTasksLength, WriteDescription]`.
pub async fn description_tool_call(
    tool_names: &[String],
    tool_map: &std::collections::HashMap<String, crate::agent::completions::ResolvedTool>,
    rng: &mut impl rand::Rng,
) -> super::client::MockToolCall {
    use super::client::{MockToolCall, random_string};

    let tool_name = pick_invention_tool("WriteDescription", tool_names, tool_map, rng).await;
    let arguments = match tool_name {
        "WriteDescription" => {
            let description = random_string(rng, 50, 350);
            serde_json::json!({ "description": description }).to_string()
        }
        "ReadSpec" | "ReadEssay" | "ReadInputSchema" | "ReadEssayTasks"
        | "ReadTasksLength" => "{}".to_string(),
        "ReadTask" => {
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

// ---------------------------------------------------------------------------
// Static multimodal content parts for Starlark expressions.
//
// Every vector completion task includes all 4 media types as static content
// parts. This guarantees modality coverage (AS20/AV18) regardless of what
// the input schema declares, without needing to walk or classify the schema.
// ---------------------------------------------------------------------------

const STATIC_IMAGE: &str =
    r#"{"type": "image_url", "image_url": {"url": "https://example.com/test.png"}}"#;
const STATIC_AUDIO: &str =
    r#"{"type": "input_audio", "input_audio": {"data": "dGVzdA==", "format": "wav"}}"#;
const STATIC_VIDEO: &str =
    r#"{"type": "video_url", "video_url": {"url": "https://example.com/test.mp4"}}"#;
const STATIC_FILE: &str =
    r#"{"type": "file", "file": {"file_data": "dGVzdA=="}}"#;

/// Build a Starlark messages expression for a vector completion task.
///
/// Stringifies the entire input (or sub-path) as a text content part and
/// includes all 4 static multimodal content parts for modality coverage.
fn build_messages_expr(input_expr: &str) -> String {
    let text_part = format!(
        r#"{{"type": "text", "text": str({input_expr})}}"#,
    );
    let content = [
        text_part.as_str(),
        STATIC_IMAGE,
        STATIC_AUDIO,
        STATIC_VIDEO,
        STATIC_FILE,
    ].join(", ");
    format!(r#"[{{"role": "user", "content": [{content}]}}]"#)
}

/// Build a Starlark responses expression for a vector leaf function.
///
/// Stringifies each item as a text content part and includes all 4 static
/// multimodal content parts for modality coverage.
fn build_responses_expr() -> String {
    let text_part = r#"{"type": "text", "text": str(item)}"#;
    let content = [
        text_part,
        STATIC_IMAGE,
        STATIC_AUDIO,
        STATIC_VIDEO,
        STATIC_FILE,
    ].join(", ");
    format!("[[{content}] for item in input['items']]")
}

/// Check whether a VectorFunctionInputSchema JSON string has a context object.
fn has_vector_context(input_schema_json: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(input_schema_json)
        .ok()
        .and_then(|v| v.get("context").cloned())
        .is_some()
}

/// Extract required field names with their full JSON schemas from a serialized
/// `ObjectInputSchema` JSON. Returns `(name, schema)` pairs.
///
/// Used by branch task generators to build child placeholder schemas with
/// the actual parent field types.
fn extract_input_field_schemas(input_schema_json: &str) -> Vec<(String, serde_json::Value)> {
    serde_json::from_str::<serde_json::Value>(input_schema_json)
        .ok()
        .and_then(|v| {
            let props = v.get("properties")?.as_object()?;
            // Prefer required fields
            let names: Vec<String> = v.get("required")
                .and_then(|r| r.as_array())
                .map(|arr| arr.iter().filter_map(|s| s.as_str().map(String::from)).collect())
                .unwrap_or_else(|| props.keys().cloned().collect());
            let result: Vec<(String, serde_json::Value)> = names.into_iter()
                .map(|name| {
                    let schema = props.get(&name).cloned()
                        .unwrap_or_else(|| serde_json::json!({"type": "string"}));
                    (name, schema)
                })
                .collect();
            if result.is_empty() { None } else { Some(result) }
        })
        .unwrap_or_else(|| vec![("text".to_string(), serde_json::json!({"type": "string"}))])
}

/// Extract the task count range (min, max) from a tasks step prompt.
///
/// Looks for "between N and M" or "Create N " patterns.
pub fn extract_task_count_range(prompt: &str) -> (u32, u32) {
    // Try "between N and M"
    if let Some(idx) = prompt.find("between ") {
        let rest = &prompt[idx + 8..];
        let parts: Vec<&str> = rest.splitn(4, ' ').collect();
        if parts.len() >= 3 && parts[1] == "and" {
            if let (Ok(min), Ok(max)) = (parts[0].parse::<u32>(), parts[2].parse::<u32>()) {
                return (min, max);
            }
        }
    }
    // Try "Create N "
    if let Some(idx) = prompt.find("Create ") {
        let rest = &prompt[idx + 7..];
        if let Some(n_str) = rest.split_whitespace().next() {
            if let Ok(n) = n_str.parse::<u32>() {
                return (n, n);
            }
        }
    }
    // Fallback
    (3, 5)
}
