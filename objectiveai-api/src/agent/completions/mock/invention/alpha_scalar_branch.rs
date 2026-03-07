use std::collections::HashMap;
use rand::Rng;
use super::super::client::{MockToolCall, generate_tool_arguments, random_string};
use crate::agent::completions::ResolvedTool;

/// Generate a mock tool call for the tasks step of a scalar branch function.
///
/// Scalar branch tasks are `PlaceholderScalarFunction` task expressions with
/// `name`, `spec`, `input_schema` (ObjectInputSchema), `input` (expression),
/// and optional `skip`.
///
/// The `input_schema_json` is the serialized parent `ScalarFunctionInputSchema`
/// obtained by calling `ReadInputSchema`, used to derive realistic child
/// input schemas and input expressions.
pub fn tasks_tool_call(
    input_schema_json: &str,
    tool_names: &[String],
    tool_map: &HashMap<String, ResolvedTool>,
    rng: &mut impl Rng,
) -> MockToolCall {
    let tool_name = &tool_names[rng.random_range(0..tool_names.len())];
    let arguments = match tool_name.as_str() {
        "AppendTask" => {
            let fields = super::extract_input_fields(input_schema_json);
            let task = random_placeholder_scalar_task(&fields, rng);
            serde_json::json!({
                "placeholder.alpha.scalar.function": task,
            }).to_string()
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

/// Generate a random placeholder scalar function task expression.
fn random_placeholder_scalar_task(
    parent_fields: &[String],
    rng: &mut impl Rng,
) -> serde_json::Value {
    let name = format!("sub-function-{}", rng.random_range(0u32..1000));
    let spec = random_string(rng, 50, 200);

    // Child input schema: pick a subset of parent fields or pass through
    let use_subset = parent_fields.len() > 1 && rng.random_range(0u32..2) == 0;
    let child_fields: Vec<&String> = if use_subset {
        let n = rng.random_range(1..parent_fields.len());
        parent_fields.iter().take(n).collect()
    } else {
        parent_fields.iter().collect()
    };

    let mut properties = serde_json::Map::new();
    for f in &child_fields {
        properties.insert(
            (*f).clone(),
            serde_json::json!({"type": "string"}),
        );
    }
    let required: Vec<&str> = child_fields.iter().map(|s| s.as_str()).collect();

    // Input expression: pass through matching fields from parent
    let input_expr = if use_subset {
        let field = child_fields[0];
        format!("{{'{field}': input['{field}']}}")
    } else {
        "input".to_string()
    };

    serde_json::json!({
        "name": name,
        "spec": spec,
        "input_schema": {
            "properties": properties,
            "required": required,
        },
        "input": { "$starlark": input_expr },
    })
}
