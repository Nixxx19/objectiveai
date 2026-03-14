use std::fs;
use std::path::Path;

fn load_schemas() -> Vec<(String, serde_json::Value)> {
    let schema_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut schemas = Vec::new();
    for entry in fs::read_dir(&schema_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let content: serde_json::Value =
                serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            schemas.push((name, content));
        }
    }
    assert!(!schemas.is_empty(), "no schema files found");
    schemas
}

const ALLOWED_KEYWORDS: &[&str] = &[
    "title",
    "description",
    "type",
    "enum",
    "anyOf",
    "$ref",
    "properties",
    "additionalProperties",
    "items",
    "minItems",
    "maxItems",
    "minimum",
    "maximum",
    "pattern",
    "format",
    "default",
];

fn collect_keywords(value: &serde_json::Value, inside_properties: bool, found: &mut std::collections::BTreeSet<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                if !inside_properties {
                    found.insert(k.clone());
                }
                collect_keywords(v, k == "properties", found);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                collect_keywords(v, false, found);
            }
        }
        _ => {}
    }
}

fn check_keyword_order(value: &serde_json::Value, inside_properties: bool, errors: &mut Vec<String>, path: &str) {
    match value {
        serde_json::Value::Object(map) => {
            if !inside_properties {
                let keys: Vec<&String> = map.keys().collect();
                let mut max_pos: Option<usize> = None;
                for key in &keys {
                    if let Some(pos) = ALLOWED_KEYWORDS.iter().position(|k| *k == key.as_str()) {
                        if let Some(prev) = max_pos {
                            if pos < prev {
                                errors.push(format!("{path}: \"{key}\" is out of order"));
                            }
                        }
                        max_pos = Some(max_pos.map_or(pos, |p| p.max(pos)));
                    }
                }
            }
            for (k, v) in map {
                let child_path = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                check_keyword_order(v, k == "properties", errors, &child_path);
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                check_keyword_order(v, false, errors, &format!("{path}[{i}]"));
            }
        }
        _ => {}
    }
}

fn check_properties_sorted(value: &serde_json::Value, errors: &mut Vec<String>, path: &str) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::Object(props)) = map.get("properties") {
                let keys: Vec<&String> = props.keys().collect();
                for w in keys.windows(2) {
                    if w[0] > w[1] {
                        errors.push(format!(
                            "{path}.properties: \"{0}\" comes before \"{1}\" but should come after",
                            w[0], w[1]
                        ));
                    }
                }
            }
            for (k, v) in map {
                let child_path = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                check_properties_sorted(v, errors, &child_path);
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                check_properties_sorted(v, errors, &format!("{path}[{i}]"));
            }
        }
        _ => {}
    }
}

fn has_type_array(value: &serde_json::Value, inside_properties: bool) -> bool {
    match value {
        serde_json::Value::Object(map) => {
            if !inside_properties {
                if let Some(serde_json::Value::Array(_)) = map.get("type") {
                    return true;
                }
            }
            map.iter()
                .any(|(k, v)| has_type_array(v, k == "properties"))
        }
        serde_json::Value::Array(arr) => arr.iter().any(|v| has_type_array(v, false)),
        _ => false,
    }
}

fn has_key_recursive(value: &serde_json::Value, target_key: &str, inside_properties: bool) -> bool {
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                if !inside_properties && k == target_key {
                    return true;
                }
                if has_key_recursive(v, target_key, k == "properties") {
                    return true;
                }
            }
            false
        }
        serde_json::Value::Array(arr) => arr
            .iter()
            .any(|v| has_key_recursive(v, target_key, false)),
        _ => false,
    }
}

#[test]
fn only_allowed_keywords() {
    let allowed: std::collections::BTreeSet<String> =
        ALLOWED_KEYWORDS.iter().map(|s| s.to_string()).collect();
    let mut all_found = std::collections::BTreeSet::new();
    for (_, schema) in load_schemas() {
        collect_keywords(&schema, false, &mut all_found);
    }
    let unexpected: Vec<_> = all_found.difference(&allowed).collect();
    assert!(
        unexpected.is_empty(),
        "unexpected keywords found: {unexpected:?}"
    );
}

#[test]
fn keywords_in_canonical_order() {
    for (name, schema) in load_schemas() {
        let mut errors = Vec::new();
        check_keyword_order(&schema, false, &mut errors, &name);
        assert!(errors.is_empty(), "keyword ordering violations:\n{}", errors.join("\n"));
    }
}

#[test]
fn properties_keys_sorted_alphabetically() {
    for (name, schema) in load_schemas() {
        let mut errors = Vec::new();
        check_properties_sorted(&schema, &mut errors, &name);
        assert!(errors.is_empty(), "properties sorting violations:\n{}", errors.join("\n"));
    }
}

#[test]
fn no_schema_property() {
    for (name, schema) in load_schemas() {
        assert!(
            !has_key_recursive(&schema, "$schema", false),
            "{name} contains a $schema property"
        );
    }
}

#[test]
fn no_type_arrays_outside_properties() {
    for (name, schema) in load_schemas() {
        assert!(
            !has_type_array(&schema, false),
            "{name} contains a type array outside of properties"
        );
    }
}

#[test]
fn no_required_outside_properties() {
    for (name, schema) in load_schemas() {
        assert!(
            !has_key_recursive(&schema, "required", false),
            "{name} contains a required key outside of properties"
        );
    }
}

#[test]
fn no_one_of_outside_properties() {
    for (name, schema) in load_schemas() {
        assert!(
            !has_key_recursive(&schema, "oneOf", false),
            "{name} contains a oneOf key outside of properties"
        );
    }
}

fn has_any_of_with_ref(value: &serde_json::Value, inside_properties: bool) -> bool {
    match value {
        serde_json::Value::Object(map) => {
            if !inside_properties && map.contains_key("anyOf") && map.contains_key("$ref") {
                return true;
            }
            map.iter()
                .any(|(k, v)| has_any_of_with_ref(v, k == "properties"))
        }
        serde_json::Value::Array(arr) => arr.iter().any(|v| has_any_of_with_ref(v, false)),
        _ => false,
    }
}

#[test]
fn no_any_of_with_sibling_ref() {
    for (name, schema) in load_schemas() {
        assert!(
            !has_any_of_with_ref(&schema, false),
            "{name} has anyOf with a sibling $ref"
        );
    }
}

#[test]
fn no_const_outside_properties() {
    for (name, schema) in load_schemas() {
        assert!(
            !has_key_recursive(&schema, "const", false),
            "{name} contains a const key outside of properties"
        );
    }
}

fn has_numeric_format(value: &serde_json::Value, inside_properties: bool) -> bool {
    match value {
        serde_json::Value::Object(map) => {
            if !inside_properties {
                let is_numeric = matches!(
                    map.get("type").and_then(|t| t.as_str()),
                    Some("integer") | Some("number")
                );
                if is_numeric && map.contains_key("format") {
                    return true;
                }
            }
            map.iter()
                .any(|(k, v)| has_numeric_format(v, k == "properties"))
        }
        serde_json::Value::Array(arr) => arr.iter().any(|v| has_numeric_format(v, false)),
        _ => false,
    }
}

#[test]
fn no_numeric_format() {
    for (name, schema) in load_schemas() {
        assert!(
            !has_numeric_format(&schema, false),
            "{name} has a format key on an integer or number type"
        );
    }
}

fn check_format_values(
    value: &serde_json::Value,
    inside_properties: bool,
    errors: &mut Vec<String>,
    path: &str,
) {
    match value {
        serde_json::Value::Object(map) => {
            if !inside_properties {
                if let Some(serde_json::Value::String(fmt)) = map.get("format") {
                    if fmt != "uuid" && fmt != "date-time" {
                        errors.push(format!("{path}: format is \"{fmt}\" (expected \"uuid\" or \"date-time\")"));
                    }
                }
            }
            for (k, v) in map {
                let child_path = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                check_format_values(v, k == "properties", errors, &child_path);
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                check_format_values(v, false, errors, &format!("{path}[{i}]"));
            }
        }
        _ => {}
    }
}

#[test]
fn format_is_uuid_or_datetime_only() {
    for (name, schema) in load_schemas() {
        let mut errors = Vec::new();
        check_format_values(&schema, false, &mut errors, &name);
        assert!(
            errors.is_empty(),
            "format must be \"uuid\" or \"date-time\":\n{}",
            errors.join("\n")
        );
    }
}

fn collect_refs(value: &serde_json::Value, refs: &mut std::collections::BTreeSet<String>, inside_properties: bool) {
    match value {
        serde_json::Value::Object(map) => {
            if !inside_properties {
                if let Some(serde_json::Value::String(r)) = map.get("$ref") {
                    refs.insert(r.clone());
                }
            }
            for (k, v) in map {
                collect_refs(v, refs, k == "properties");
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                collect_refs(v, refs, false);
            }
        }
        _ => {}
    }
}

fn check_min_max(value: &serde_json::Value, inside_properties: bool, errors: &mut Vec<String>, path: &str) {
    match value {
        serde_json::Value::Object(map) => {
            if !inside_properties {
                if let (Some(min), Some(max)) = (map.get("minimum"), map.get("maximum")) {
                    if let (Some(min_f), Some(max_f)) = (min.as_f64(), max.as_f64()) {
                        if min_f > max_f {
                            errors.push(format!("{path}: minimum ({min_f}) > maximum ({max_f})"));
                        }
                    }
                }
            }
            for (k, v) in map {
                let child_path = if path.is_empty() { k.clone() } else { format!("{path}.{k}") };
                check_min_max(v, k == "properties", errors, &child_path);
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                check_min_max(v, false, errors, &format!("{path}[{i}]"));
            }
        }
        _ => {}
    }
}

#[test]
fn minimum_never_exceeds_maximum() {
    for (name, schema) in load_schemas() {
        let mut errors = Vec::new();
        check_min_max(&schema, false, &mut errors, &name);
        assert!(errors.is_empty(), "min/max violations:\n{}", errors.join("\n"));
    }
}

fn check_multi_variant_anyof_not_nullable(
    value: &serde_json::Value,
    inside_properties: bool,
    errors: &mut Vec<String>,
    path: &str,
) {
    match value {
        serde_json::Value::Object(map) => {
            if !inside_properties {
                if let Some(serde_json::Value::Array(variants)) = map.get("anyOf") {
                    let non_null_count = variants
                        .iter()
                        .filter(|v| v.get("type").and_then(|t| t.as_str()) != Some("null"))
                        .count();
                    let has_null = variants.iter().any(|v| {
                        v.get("type").and_then(|t| t.as_str()) == Some("null")
                    });
                    if non_null_count >= 2 && has_null {
                        errors.push(format!(
                            "{path}: anyOf has {non_null_count} non-null variants plus a null variant"
                        ));
                    }
                }
            }
            for (k, v) in map {
                let child_path = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                check_multi_variant_anyof_not_nullable(v, k == "properties", errors, &child_path);
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                check_multi_variant_anyof_not_nullable(v, false, errors, &format!("{path}[{i}]"));
            }
        }
        _ => {}
    }
}

#[test]
fn multi_variant_anyof_never_nullable() {
    for (name, schema) in load_schemas() {
        let mut errors = Vec::new();
        check_multi_variant_anyof_not_nullable(&schema, false, &mut errors, &name);
        assert!(
            errors.is_empty(),
            "anyOf with 2+ non-null variants must not include a null variant:\n{}",
            errors.join("\n")
        );
    }
}

fn check_anyof_in_properties(
    value: &serde_json::Value,
    in_properties: bool,
    errors: &mut Vec<String>,
    path: &str,
) {
    match value {
        serde_json::Value::Object(map) => {
            if in_properties {
                // We're looking at a property value (sub-schema).
                if let Some(serde_json::Value::Array(variants)) = map.get("anyOf") {
                    if variants.len() != 2 {
                        errors.push(format!(
                            "{path}: anyOf has {} variants (expected exactly 2)",
                            variants.len()
                        ));
                    } else {
                        let has_null = variants.iter().any(|v| {
                            v.get("type").and_then(|t| t.as_str()) == Some("null")
                        });
                        if !has_null {
                            errors.push(format!(
                                "{path}: anyOf with 2 variants but neither is {{\"type\": \"null\"}}"
                            ));
                        }
                    }
                }
            }
            for (k, v) in map {
                let child_path = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                check_anyof_in_properties(v, k == "properties", errors, &child_path);
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                check_anyof_in_properties(v, false, errors, &format!("{path}[{i}]"));
            }
        }
        _ => {}
    }
}

#[test]
fn anyof_in_properties_is_nullable_only() {
    for (name, schema) in load_schemas() {
        let mut errors = Vec::new();
        check_anyof_in_properties(&schema, false, &mut errors, &name);
        assert!(
            errors.is_empty(),
            "anyOf inside properties must be exactly [{{non-null}}, {{\"type\": \"null\"}}]:\n{}",
            errors.join("\n")
        );
    }
}

#[test]
fn all_refs_resolve() {
    let schemas = load_schemas();
    let all_titles: std::collections::BTreeSet<String> = schemas
        .iter()
        .filter_map(|(_, s)| s.get("title").and_then(|t| t.as_str()).map(String::from))
        .collect();
    let mut all_refs = std::collections::BTreeSet::new();
    for (_, schema) in &schemas {
        collect_refs(schema, &mut all_refs, false);
    }
    let unresolved: Vec<&String> = all_refs.difference(&all_titles).collect();
    assert!(
        unresolved.is_empty(),
        "$ref targets not found as schema titles: {unresolved:?}"
    );
}
