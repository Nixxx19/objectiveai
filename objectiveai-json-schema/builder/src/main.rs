use std::fs;
use std::path::Path;

fn strip_defs_and_rewrite_refs(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            map.remove("$defs");
            if let Some(serde_json::Value::String(r)) = map.get_mut("$ref") {
                if let Some(name) = r.strip_prefix("#/$defs/") {
                    *r = name.to_string();
                }
            }
            for v in map.values_mut() {
                strip_defs_and_rewrite_refs(v);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                strip_defs_and_rewrite_refs(v);
            }
        }
        _ => {}
    }
}

fn main() {
    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");

    // Clear everything except the builder folder
    for entry in fs::read_dir(&out_dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        let name = name.to_str().unwrap();
        if name == "builder" {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path).unwrap();
        } else {
            fs::remove_file(&path).unwrap();
        }
    }

    let schemas = objectiveai::json_schemas();
    let mut count = 0;

    for schema in &schemas {
        let mut json = serde_json::to_value(schema).unwrap();
        let title = json
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or_else(|| panic!("schema missing title: {json}"))
            .to_string();

        strip_defs_and_rewrite_refs(&mut json);

        let filename = format!("{title}JsonSchema.json");
        let path = out_dir.join(&filename);
        let contents = serde_json::to_string_pretty(&json).unwrap();
        fs::write(&path, format!("{contents}\n")).unwrap();
        count += 1;
    }

    println!("Wrote {count} schema files to {}", out_dir.display());
}
