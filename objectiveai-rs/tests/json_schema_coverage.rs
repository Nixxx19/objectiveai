use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use syn::{Item, Visibility};
use walkdir::WalkDir;

/// Whitelisted paths and types exempt from JsonSchema requirements.
/// Each entry is either:
/// - A folder path (ending with `/`) to skip all types in that subtree
/// - A `("src/path.rs", "TypeName")` pair to skip a specific type
const WHITELIST_FOLDERS: &[&str] = &[
    "src/functions/check/example_inputs/",
    "src/http/",
];

const WHITELIST_TYPES: &[(&str, &str)] = &[
    ("src/functions/expression/error.rs", "ExpressionError"),
    ("src/functions/inventions/tool.rs", "InventionTool"),
    ("src/prefixed_uuid.rs", "ParseError"),
];

fn is_whitelisted(path: &str, name: &str) -> bool {
    WHITELIST_FOLDERS
        .iter()
        .any(|folder| path.starts_with(folder))
        || WHITELIST_TYPES
            .iter()
            .any(|(p, n)| path == *p && *n == name)
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

/// Build a module prefix from a path like "src/functions/check/example_inputs/file.rs".
/// Strips "src/" prefix, drops the filename entirely, then PascalCases each folder segment.
fn module_prefix(path: &str) -> String {
    let inner = path.strip_prefix("src/").unwrap_or(path);
    let segments: Vec<&str> = inner.split('/').collect();

    // Take only folder segments (skip the last segment which is the filename)
    segments[..segments.len().saturating_sub(1)]
        .iter()
        .map(|seg| to_pascal_case(seg))
        .collect::<Vec<_>>()
        .join("")
}

fn has_json_schema_derive(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("derive") {
            let tokens = attr
                .meta
                .require_list()
                .ok()
                .map(|list| list.tokens.to_string());
            tokens.map_or(false, |t| {
                t.split(',').any(|s| {
                    s.split("::")
                        .last()
                        .map_or(false, |last| last.trim() == "JsonSchema")
                })
            })
        } else {
            false
        }
    })
}

fn get_schemars_rename(attrs: &[syn::Attribute]) -> Option<String> {
    attrs.iter().find_map(|attr| {
        if attr.path().is_ident("schemars") {
            let list = attr.meta.require_list().ok()?;
            let tokens = list.tokens.to_string();
            let rest = tokens.strip_prefix("rename")?;
            let rest = rest.trim().strip_prefix('=')?;
            let rest = rest.trim().strip_prefix('"')?;
            rest.strip_suffix('"').map(|s| s.to_string())
        } else {
            None
        }
    })
}

/// Returns true if the item has type parameters (not lifetime or const params).
fn has_type_params(item: &Item) -> bool {
    let generics = match item {
        Item::Struct(s) => &s.generics,
        Item::Enum(e) => &e.generics,
        _ => return false,
    };
    generics
        .params
        .iter()
        .any(|p| matches!(p, syn::GenericParam::Type(_)))
}

#[test]
fn all_public_types_have_json_schema() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let source_root = Path::new(manifest_dir).join("src");

    let mut errors: Vec<String> = Vec::new();

    for entry in WalkDir::new(&source_root) {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }

        let relative = path
            .strip_prefix(manifest_dir)
            .unwrap()
            .to_str()
            .unwrap()
            .replace('\\', "/");

        // Skip entire whitelisted folders early
        if WHITELIST_FOLDERS
            .iter()
            .any(|folder| relative.starts_with(folder))
        {
            continue;
        }

        let source = fs::read_to_string(path).unwrap();
        let file = match syn::parse_file(&source) {
            Ok(f) => f,
            Err(_) => continue,
        };

        let prefix = module_prefix(&relative);

        for item in &file.items {
            let (name, attrs) = match item {
                Item::Struct(s) if matches!(s.vis, Visibility::Public(_)) => {
                    (s.ident.to_string(), &s.attrs)
                }
                Item::Enum(e) if matches!(e.vis, Visibility::Public(_)) => {
                    (e.ident.to_string(), &e.attrs)
                }
                _ => continue,
            };

            let full_name = format!("{prefix}{name}");

            if is_whitelisted(&relative, &name) {
                continue;
            }

            if !has_json_schema_derive(attrs) {
                errors.push(format!(
                    "{name} in {relative} is missing #[derive(JsonSchema)]"
                ));
                continue;
            }

            // For types with type parameters, the rename contains {T} placeholders,
            // so we check the prefix matches rather than exact equality.
            let has_type_param = has_type_params(item);

            match get_schemars_rename(attrs) {
                None => {
                    errors.push(format!(
                        "{name} in {relative} is missing \
                         #[schemars(rename = \"{full_name}\")]"
                    ));
                }
                Some(rename) if has_type_param => {
                    let expected_prefix = format!("{full_name}{{");
                    if !rename.starts_with(&expected_prefix) {
                        errors.push(format!(
                            "{name} in {relative} has wrong schemars rename: \
                             got \"{rename}\", expected \"{full_name}{{...}}\""
                        ));
                    }
                }
                Some(rename) if rename != full_name => {
                    errors.push(format!(
                        "{name} in {relative} has wrong schemars rename: \
                         got \"{rename}\", expected \"{full_name}\""
                    ));
                }
                _ => {}
            }
        }
    }

    if !errors.is_empty() {
        panic!(
            "JsonSchema coverage errors ({}):\n{}",
            errors.len(),
            errors.join("\n")
        );
    }
}

/// Verifies that `json_schemas()` returns a schema for every non-whitelisted,
/// non-generic public struct/enum that derives JsonSchema.
#[test]
fn json_schemas_covers_all_types() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let source_root = Path::new(manifest_dir).join("src");

    // Collect all expected full_names from AST walking, skipping generic types
    let mut expected: BTreeSet<String> = BTreeSet::new();

    for entry in WalkDir::new(&source_root) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let relative = path
            .strip_prefix(manifest_dir)
            .unwrap()
            .to_str()
            .unwrap()
            .replace('\\', "/");
        if WHITELIST_FOLDERS
            .iter()
            .any(|folder| relative.starts_with(folder))
        {
            continue;
        }
        let source = fs::read_to_string(path).unwrap();
        let file = match syn::parse_file(&source) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let prefix = module_prefix(&relative);
        for item in &file.items {
            let (name, attrs) = match item {
                Item::Struct(s)
                    if matches!(s.vis, Visibility::Public(_)) =>
                {
                    (s.ident.to_string(), &s.attrs)
                }
                Item::Enum(e)
                    if matches!(e.vis, Visibility::Public(_)) =>
                {
                    (e.ident.to_string(), &e.attrs)
                }
                _ => continue,
            };
            if is_whitelisted(&relative, &name) {
                continue;
            }
            if !has_json_schema_derive(attrs) {
                continue;
            }
            // Skip types with type parameters — their titles contain
            // concrete substitutions that don't match the template name.
            if has_type_params(item) {
                continue;
            }
            let full_name = format!("{prefix}{name}");
            expected.insert(full_name);
        }
    }

    // Collect titles from json_schemas()
    let schemas = objectiveai::json_schemas();
    let mut actual: BTreeSet<String> = BTreeSet::new();
    for schema in &schemas {
        let json = serde_json::to_value(schema).unwrap();
        if let Some(title) = json.get("title").and_then(|t| t.as_str()) {
            actual.insert(title.to_string());
        }
    }

    let missing: Vec<&String> = expected.difference(&actual).collect();
    if !missing.is_empty() {
        panic!(
            "Types in source but missing from json_schemas() ({}):\n{}",
            missing.len(),
            missing
                .iter()
                .map(|m| format!("  - {m}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

/// Collects all `$ref` targets from a JSON value recursively.
fn collect_refs(value: &serde_json::Value, refs: &mut BTreeSet<String>) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::String(r)) = map.get("$ref") {
                if let Some(name) = r.strip_prefix("#/$defs/") {
                    refs.insert(name.to_string());
                }
            }
            for v in map.values() {
                collect_refs(v, refs);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                collect_refs(v, refs);
            }
        }
        _ => {}
    }
}

/// Verifies that every `$ref` used across all schemas returned by `json_schemas()`
/// exists as a title of some schema in the returned set.
#[test]
fn json_schemas_refs_are_complete() {
    let schemas = objectiveai::json_schemas();

    // Collect all titles
    let mut all_titles: BTreeSet<String> = BTreeSet::new();
    for schema in &schemas {
        let json = serde_json::to_value(schema).unwrap();
        if let Some(title) = json.get("title").and_then(|t| t.as_str()) {
            all_titles.insert(title.to_string());
        }
    }

    // Collect all $ref targets across all schemas
    let mut all_refs: BTreeSet<String> = BTreeSet::new();
    for schema in &schemas {
        let json = serde_json::to_value(schema).unwrap();
        collect_refs(&json, &mut all_refs);
    }

    // Every $ref target must exist as a title of some schema
    let unresolved: Vec<&String> =
        all_refs.difference(&all_titles).collect();

    if !unresolved.is_empty() {
        panic!(
            "$ref targets not found as titles in json_schemas() ({}):\n{}",
            unresolved.len(),
            unresolved
                .iter()
                .map(|r| format!("  - {r}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}
