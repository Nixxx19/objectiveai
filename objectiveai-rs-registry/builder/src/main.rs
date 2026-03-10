use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use syn::{Fields, Item, Visibility};
use walkdir::WalkDir;

#[derive(Serialize)]
struct TypeEntry {
    name: String,
    full_name: String,
    kind: &'static str,
    path: String,
    line_start: usize,
    line_end: usize,
}

/// Convert a snake_case segment to PascalCase: "example_inputs" → "ExampleInputs"
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

/// Build a module prefix from a path like "objectiveai-rs/src/functions/check/example_inputs/file.rs".
/// Strips "objectiveai-rs/src/" prefix, drops "lib.rs"/"mod.rs" filenames (or strips .rs extension),
/// then PascalCases each segment.
fn module_prefix(path: &str) -> String {
    let inner = path
        .strip_prefix("objectiveai-rs/src/")
        .unwrap_or(path);

    let segments: Vec<&str> = inner.split('/').collect();
    let mut parts: Vec<String> = Vec::new();

    for (i, seg) in segments.iter().enumerate() {
        let is_last = i == segments.len() - 1;
        if is_last {
            // Skip lib.rs and mod.rs; otherwise strip .rs extension
            match *seg {
                "lib.rs" | "mod.rs" => {}
                _ => {
                    let name = seg.strip_suffix(".rs").unwrap_or(seg);
                    parts.push(to_pascal_case(name));
                }
            }
        } else {
            parts.push(to_pascal_case(seg));
        }
    }

    parts.join("")
}

fn extract_public_types(source: &str, path: &str) -> Vec<TypeEntry> {
    let file = match syn::parse_file(source) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let prefix = module_prefix(path);
    let mut entries = Vec::new();

    for item in &file.items {
        match item {
            Item::Struct(s) if matches!(s.vis, Visibility::Public(_)) => {
                let line_end = match &s.fields {
                    Fields::Named(f) => f.brace_token.span.close().end().line,
                    _ => s
                        .semi_token
                        .map(|t| t.span.end().line)
                        .unwrap_or(s.ident.span().end().line),
                };
                let name = s.ident.to_string();
                entries.push(TypeEntry {
                    full_name: format!("{prefix}{name}"),
                    name,
                    kind: "struct",
                    path: path.to_owned(),
                    line_start: s.ident.span().start().line,
                    line_end,
                });
            }
            Item::Enum(e) if matches!(e.vis, Visibility::Public(_)) => {
                let name = e.ident.to_string();
                entries.push(TypeEntry {
                    full_name: format!("{prefix}{name}"),
                    name,
                    kind: "enum",
                    path: path.to_owned(),
                    line_start: e.ident.span().start().line,
                    line_end: e.brace_token.span.close().end().line,
                });
            }
            Item::Type(t) if matches!(t.vis, Visibility::Public(_)) => {
                let name = t.ident.to_string();
                entries.push(TypeEntry {
                    full_name: format!("{prefix}{name}"),
                    name,
                    kind: "type",
                    path: path.to_owned(),
                    line_start: t.ident.span().start().line,
                    line_end: t.semi_token.span.end().line,
                });
            }
            _ => {}
        }
    }

    entries
}

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let registry_root = Path::new(manifest_dir).parent().unwrap();
    let repo_root = registry_root.parent().unwrap();
    let source_root = repo_root.join("objectiveai-rs").join("src");
    let output_root = registry_root.join("src");

    // Clean previous output entirely to remove orphans from deleted modules
    if output_root.exists() {
        fs::remove_dir_all(&output_root).unwrap();
    }

    let mut total_files = 0;
    let mut total_types = 0;

    for entry in WalkDir::new(&source_root) {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }

        let source = fs::read_to_string(path).unwrap();
        let relative_to_repo = path
            .strip_prefix(repo_root)
            .unwrap()
            .to_str()
            .unwrap()
            .replace('\\', "/");
        let types = extract_public_types(&source, &relative_to_repo);

        if types.is_empty() {
            continue;
        }

        let relative = path.strip_prefix(&source_root).unwrap();
        let mut output_path: PathBuf = output_root.join(relative);
        let mut file_name = output_path.file_name().unwrap().to_os_string();
        file_name.push(".json");
        output_path.set_file_name(file_name);

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        total_types += types.len();
        total_files += 1;

        let json = serde_json::to_string_pretty(&types).unwrap();
        fs::write(&output_path, json).unwrap();
    }

    println!("Registry built: {total_types} types from {total_files} files");
}
