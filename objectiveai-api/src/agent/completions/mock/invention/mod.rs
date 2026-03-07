mod alpha_scalar;
mod alpha_scalar_branch;
mod alpha_scalar_leaf;
mod alpha_vector;
mod alpha_vector_branch;
mod alpha_vector_leaf;
mod route;

pub use route::*;

/// Generate a mock tool call for the description step (shared across all routes).
///
/// Tools: `[ReadSpec, ReadEssay, ReadInputSchema, ReadEssayTasks, ReadTask,
/// ReadTasksLength, WriteDescription]`.
pub fn description_tool_call(
    tool_names: &[String],
    tool_map: &std::collections::HashMap<String, crate::agent::completions::ResolvedTool>,
    rng: &mut impl rand::Rng,
) -> super::client::MockToolCall {
    use super::client::{MockToolCall, generate_tool_arguments, random_string};

    let tool_name = &tool_names[rng.random_range(0..tool_names.len())];
    let arguments = match tool_name.as_str() {
        "WriteDescription" => {
            let description = random_string(rng, 50, 350);
            serde_json::json!({ "description": description }).to_string()
        }
        "ReadSpec" | "ReadEssay" | "ReadInputSchema" | "ReadEssayTasks"
        | "ReadTasksLength" => "{}".to_string(),
        "ReadTask" => {
            serde_json::json!({ "index": rng.random_range(0u32..5) }).to_string()
        }
        _ => generate_tool_arguments(tool_map, tool_name, rng),
    };
    MockToolCall {
        tool_name: tool_name.clone(),
        call_id: format!("call_mock_{}", rng.random_range(0u64..u64::MAX)),
        arguments,
        n_deltas: rng.random_range(1u32..=5) as usize,
    }
}

/// Extract property names from a serialized `ObjectInputSchema` JSON.
fn extract_input_fields(input_schema_json: &str) -> Vec<String> {
    serde_json::from_str::<serde_json::Value>(input_schema_json)
        .ok()
        .and_then(|v| v.get("properties")?.as_object().map(|o| {
            o.keys().cloned().collect()
        }))
        .unwrap_or_else(|| vec!["text".to_string()])
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
