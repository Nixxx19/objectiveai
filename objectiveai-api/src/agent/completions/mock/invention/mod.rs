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
// Schema introspection for arbitrary input schemas
// ---------------------------------------------------------------------------

/// A discovered field path within a schema, with its type classification.
///
/// `path_expr` is a Starlark expression fragment like `input['photo']` or
/// `input['metadata']['author']` that evaluates to this field's value at
/// runtime.
#[derive(Clone)]
struct SchemaField {
    /// Starlark expression to access this field, e.g. `input['photo']`.
    path_expr: String,
    /// The JSON schema for this field (the full sub-schema, not just type).
    schema: serde_json::Value,
    /// Classified type.
    prop_type: schema_gen::PropType,
}

/// Classify a JSON schema value into a PropType.
fn classify_schema_type(schema: &serde_json::Value) -> schema_gen::PropType {
    // Check for anyOf first
    if schema.get("anyOf").is_some() {
        // Treat union types as string for simplicity
        return schema_gen::PropType::String;
    }
    match schema.get("type").and_then(|t| t.as_str()) {
        Some("string") => schema_gen::PropType::String,
        Some("number") => schema_gen::PropType::Number,
        Some("integer") => schema_gen::PropType::Integer,
        Some("boolean") => schema_gen::PropType::Boolean,
        Some("image") => schema_gen::PropType::Image,
        Some("audio") => schema_gen::PropType::Audio,
        Some("video") => schema_gen::PropType::Video,
        Some("file") => schema_gen::PropType::File,
        Some("array") => schema_gen::PropType::StringArray,
        Some("object") => schema_gen::PropType::String, // nested objects → str() for text
        _ => schema_gen::PropType::String,
    }
}

/// Recursively walk an object schema and collect all leaf fields with their
/// access paths. Handles arbitrary depth.
///
/// `prefix` is the Starlark expression prefix (e.g. `input` or `item`).
/// Only required fields are included (falls back to all properties if no
/// required array).
fn walk_object_fields(
    schema: &serde_json::Value,
    prefix: &str,
    max_depth: usize,
    out: &mut Vec<SchemaField>,
) {
    let props = match schema.get("properties").and_then(|p| p.as_object()) {
        Some(p) => p,
        None => return,
    };

    // Start with required fields. Then add any non-required media fields,
    // because the validator (AV18/AS20) checks all declared properties for
    // multimodal types and expects them to be referenced in expressions.
    // Non-media optional fields are skipped — they may not be present in
    // example inputs generated by the validator.
    let field_names: Vec<String> = {
        let mut names: Vec<String> = schema.get("required")
            .and_then(|r| r.as_array())
            .map(|arr| arr.iter().filter_map(|s| s.as_str().map(String::from)).collect())
            .unwrap_or_default();
        for (key, sub_schema) in props {
            if !names.contains(key) && classify_schema_type(sub_schema).is_media() {
                names.push(key.clone());
            }
        }
        if names.is_empty() {
            props.keys().cloned().collect()
        } else {
            names
        }
    };

    for name in &field_names {
        let field_schema = match props.get(name) {
            Some(s) => s,
            None => continue,
        };

        let path = format!("{prefix}['{name}']");
        let pt = classify_schema_type(field_schema);

        // For nested objects, recurse if we have depth budget
        if pt == schema_gen::PropType::String
            && field_schema.get("type").and_then(|t| t.as_str()) == Some("object")
            && max_depth > 0
        {
            walk_object_fields(field_schema, &path, max_depth - 1, out);
        } else {
            out.push(SchemaField {
                path_expr: path,
                schema: field_schema.clone(),
                prop_type: pt,
            });
        }
    }
}

/// Extract fields from a top-level object input schema JSON string.
///
/// Returns all leaf fields with their Starlark access paths and types.
/// Handles nested objects up to 3 levels deep.
fn extract_schema_fields(input_schema_json: &str, prefix: &str) -> Vec<SchemaField> {
    let mut fields = Vec::new();
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(input_schema_json) {
        walk_object_fields(&v, prefix, 3, &mut fields);
    }
    if fields.is_empty() {
        // Fallback: single text field
        fields.push(SchemaField {
            path_expr: format!("{prefix}['text']"),
            schema: serde_json::json!({"type": "string"}),
            prop_type: schema_gen::PropType::String,
        });
    }
    fields
}

/// Extract fields from the items sub-schema of a VectorFunctionInputSchema.
///
/// If items are objects, returns their property fields with `item['field']` paths.
/// If items are a simple type (string, image, etc.), returns a single field
/// with path `item`.
fn extract_vector_item_fields(input_schema_json: &str) -> Vec<SchemaField> {
    let v = match serde_json::from_str::<serde_json::Value>(input_schema_json) {
        Ok(v) => v,
        Err(_) => return vec![SchemaField {
            path_expr: "item".into(),
            schema: serde_json::json!({"type": "string"}),
            prop_type: schema_gen::PropType::String,
        }],
    };

    let items_array = match v.get("items") {
        Some(arr) => arr,
        None => return vec![SchemaField {
            path_expr: "item".into(),
            schema: serde_json::json!({"type": "string"}),
            prop_type: schema_gen::PropType::String,
        }],
    };

    // The array's `items` field defines the per-element schema
    let item_schema = match items_array.get("items") {
        Some(s) => s,
        None => return vec![SchemaField {
            path_expr: "item".into(),
            schema: serde_json::json!({"type": "string"}),
            prop_type: schema_gen::PropType::String,
        }],
    };

    let item_type = classify_schema_type(item_schema);

    // If items are objects with properties, extract their fields
    if item_schema.get("type").and_then(|t| t.as_str()) == Some("object") {
        let mut fields = Vec::new();
        walk_object_fields(item_schema, "item", 2, &mut fields);
        if !fields.is_empty() {
            return fields;
        }
    }

    // Simple item type (string, image, etc.)
    vec![SchemaField {
        path_expr: "item".into(),
        schema: item_schema.clone(),
        prop_type: item_type,
    }]
}

/// Extract context fields from a VectorFunctionInputSchema, if context exists.
fn extract_vector_context_fields(input_schema_json: &str) -> Vec<SchemaField> {
    let v = match serde_json::from_str::<serde_json::Value>(input_schema_json) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let context = match v.get("context") {
        Some(c) => c,
        None => return Vec::new(),
    };
    let mut fields = Vec::new();
    walk_object_fields(context, "input['context']", 2, &mut fields);
    fields
}

/// Build a Starlark messages expression for a scalar function's vector
/// completion task, given the parent's schema fields.
///
/// Media fields are passed directly as content parts. Textual fields are
/// wrapped in `{"type": "text", "text": str(...)}`.
fn build_messages_expr(
    fields: &[SchemaField],
    rng: &mut impl rand::Rng,
) -> String {
    let mut parts = Vec::new();

    // All media fields → direct content parts
    for f in fields {
        if f.prop_type.is_media() {
            parts.push(f.path_expr.clone());
        }
    }

    // Pick a textual field for the text part
    let textual: Vec<&SchemaField> = fields.iter()
        .filter(|f| f.prop_type.is_textual())
        .collect();

    if !textual.is_empty() {
        let f = textual[rng.random_range(0..textual.len())];
        let prefixes = ["", "Evaluate: ", "Rate the following: ", "Score this: "];
        let prefix = prefixes[rng.random_range(0..prefixes.len())];
        parts.push(format!(
            r#"{{"type": "text", "text": "{prefix}" + str({})}}"#,
            f.path_expr,
        ));
    }

    if parts.is_empty() {
        parts.push(r#"{"type": "text", "text": "evaluate"}"#.to_string());
    }

    let content = parts.join(", ");
    format!(r#"[{{"role": "user", "content": [{content}]}}]"#)
}

/// Build a Starlark responses expression for a vector leaf function.
///
/// Maps each item to a response (array of content parts). Media items
/// are passed directly; textual items/fields are wrapped in text parts.
fn build_responses_expr(
    item_fields: &[SchemaField],
    rng: &mut impl rand::Rng,
) -> String {
    let mut parts = Vec::new();

    // All media fields → direct content parts
    for f in item_fields {
        if f.prop_type.is_media() {
            parts.push(f.path_expr.clone());
        }
    }

    // Pick a textual field for text
    let textual: Vec<&SchemaField> = item_fields.iter()
        .filter(|f| f.prop_type.is_textual())
        .collect();

    if !textual.is_empty() {
        let f = textual[rng.random_range(0..textual.len())];
        parts.push(format!(
            r#"{{"type": "text", "text": str({})}}"#,
            f.path_expr,
        ));
    }

    if parts.is_empty() {
        // No fields at all — use str(item)
        parts.push(r#"{"type": "text", "text": str(item)}"#.to_string());
    }

    let content = parts.join(", ");
    format!("[[{content}] for item in input['items']]")
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
