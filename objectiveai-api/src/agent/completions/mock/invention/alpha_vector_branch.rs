use std::collections::HashMap;
use rand::Rng;
use super::super::client::{MockToolCall, generate_tool_arguments, random_string};
use crate::agent::completions::ResolvedTool;

/// Generate a mock tool call for the tasks step of a vector branch function.
///
/// Vector branch tasks can be either:
/// - `placeholder.alpha.vector.function` — ranks items relative to each other
/// - `placeholder.alpha.scalar.function` — scores individual items (max 50% of tasks)
///
/// The `input_schema_json` is the serialized `VectorFunctionInputSchema`
/// obtained by calling `ReadInputSchema`.
///
/// `scalar_count` and `total_count` track how many scalar tasks have been
/// appended so far, ensuring at most 50% are scalar.
pub fn tasks_tool_call(
    input_schema_json: &str,
    scalar_count: u32,
    total_count: u32,
    tool_names: &[String],
    tool_map: &HashMap<String, ResolvedTool>,
    rng: &mut impl Rng,
) -> MockToolCall {
    let tool_name = &tool_names[rng.random_range(0..tool_names.len())];
    let arguments = match tool_name.as_str() {
        "AppendTask" => {
            let item_fields = extract_item_fields(input_schema_json);
            let has_context = has_context_field(input_schema_json);

            // Decide scalar vs vector. Scalar allowed if under 50% so far.
            let scalar_allowed = total_count == 0
                || (scalar_count as f64 / (total_count + 1) as f64) < 0.5;
            let use_scalar = scalar_allowed && rng.random_range(0u32..3) == 0; // ~33% chance

            if use_scalar {
                let task = random_placeholder_scalar_task(
                    &item_fields, has_context, rng,
                );
                serde_json::json!({
                    "placeholder.alpha.scalar.function": task,
                }).to_string()
            } else {
                let task = random_placeholder_vector_task(
                    &item_fields, has_context, rng,
                );
                serde_json::json!({
                    "placeholder.alpha.vector.function": task,
                }).to_string()
            }
        }
        "CheckFunction" | "ReadSpec" | "ReadEssay" | "ReadInputSchema"
        | "ReadEssayTasks" | "ReadTasksLength" => "{}".to_string(),
        "DeleteTask" | "ReadTask" => {
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

/// Generate a random placeholder vector function task expression.
fn random_placeholder_vector_task(
    item_fields: &[String],
    has_context: bool,
    rng: &mut impl Rng,
) -> serde_json::Value {
    let name = format!("vector-sub-{}", rng.random_range(0u32..1000));
    let spec = random_string(rng, 50, 200);

    // Child input schema mirrors parent structure (items + optional context)
    let mut schema = serde_json::Map::new();
    if !item_fields.is_empty() {
        let mut item_props = serde_json::Map::new();
        for f in item_fields {
            item_props.insert(f.clone(), serde_json::json!({"type": "string"}));
        }
        schema.insert("items".into(), serde_json::json!({
            "type": "array",
            "items": {
                "type": "object",
                "properties": item_props,
                "required": item_fields,
            }
        }));
    } else {
        schema.insert("items".into(), serde_json::json!({
            "type": "array",
            "items": {"type": "string"}
        }));
    }
    if has_context {
        schema.insert("context".into(), serde_json::json!({
            "properties": {"query": {"type": "string"}},
            "required": ["query"],
        }));
    }

    // Input expression: pass through
    let input_expr = "input";

    serde_json::json!({
        "name": name,
        "spec": spec,
        "input_schema": schema,
        "input": { "$starlark": input_expr },
    })
}

/// Generate a random placeholder scalar function task expression.
/// Scalar sub-functions in a vector branch score individual items via `map`.
fn random_placeholder_scalar_task(
    item_fields: &[String],
    _has_context: bool,
    rng: &mut impl Rng,
) -> serde_json::Value {
    let name = format!("scalar-sub-{}", rng.random_range(0u32..1000));
    let spec = random_string(rng, 50, 200);

    // Child input schema: the individual item
    let child_schema = if !item_fields.is_empty() {
        let mut props = serde_json::Map::new();
        for f in item_fields {
            props.insert(f.clone(), serde_json::json!({"type": "string"}));
        }
        serde_json::json!({
            "properties": props,
            "required": item_fields,
        })
    } else {
        serde_json::json!({
            "properties": {"value": {"type": "string"}},
            "required": ["value"],
        })
    };

    // Input expression: index into items with map
    let input_expr = "input['items'][map]";

    serde_json::json!({
        "name": name,
        "spec": spec,
        "input_schema": child_schema,
        "input": { "$starlark": input_expr },
    })
}

/// Extract property names from the `items` schema within a VectorFunctionInputSchema.
fn extract_item_fields(input_schema_json: &str) -> Vec<String> {
    serde_json::from_str::<serde_json::Value>(input_schema_json)
        .ok()
        .and_then(|v| {
            let items = v.get("items")?;
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
