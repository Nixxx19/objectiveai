pub(super) mod alpha_scalar;
pub(super) mod alpha_scalar_branch;
pub(super) mod alpha_scalar_leaf;
pub(super) mod alpha_vector;
pub(super) mod alpha_vector_branch;
pub(super) mod alpha_vector_leaf;
mod route;

pub use route::*;

/// Pick an invention tool with weighted selection.
///
/// 50% chance: return the key write tool for this step.
/// 50% chance: pick randomly from `tool_names`, re-rolling non-invention tools.
pub(super) fn pick_invention_tool<'a>(
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
    loop {
        let name = &tool_names[rng.random_range(0..tool_names.len())];
        if matches!(tool_map.get(name.as_str()), Some(crate::agent::completions::ResolvedTool::InventionTool(_))) {
            return name.as_str();
        }
    }
}

/// Generate a mock tool call for the description step (shared across all routes).
///
/// Tools: `[ReadSpec, ReadEssay, ReadInputSchema, ReadEssayTasks, ReadTask,
/// ReadTasksLength, WriteDescription]`.
pub fn description_tool_call(
    tool_names: &[String],
    tool_map: &std::collections::HashMap<String, crate::agent::completions::ResolvedTool>,
    rng: &mut impl rand::Rng,
) -> super::client::MockToolCall {
    use super::client::{MockToolCall, random_string};

    let tool_name = pick_invention_tool("WriteDescription", tool_names, tool_map, rng);
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
