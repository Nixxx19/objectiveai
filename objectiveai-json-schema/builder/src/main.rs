use std::fs;
use std::path::Path;

fn normalize(value: &mut serde_json::Value, inside_properties: bool, title: &str) {
    match value {
        serde_json::Value::Object(map) => {
            if !inside_properties {
                map.remove("$defs");
                map.remove("$schema");
                // Convert oneOf → anyOf
                if let Some(one_of) = map.remove("oneOf") {
                    map.insert("anyOf".to_string(), one_of);
                }
                // Flatten single-variant anyOf: merge the variant's keys into the parent
                if let Some(serde_json::Value::Array(variants)) = map.remove("anyOf") {
                    if variants.len() == 1 {
                        if let Some(serde_json::Value::Object(inner)) = variants.into_iter().next()
                        {
                            for (k, v) in inner {
                                map.insert(k, v);
                            }
                        }
                    } else {
                        map.insert(
                            "anyOf".to_string(),
                            serde_json::Value::Array(variants),
                        );
                    }
                }
                // Post-flatten fixups (inlined keys may include $ref, required, const)
                if let Some(serde_json::Value::String(r)) = map.get_mut("$ref") {
                    if *r == "#" {
                        *r = title.to_string();
                    } else if let Some(name) = r.strip_prefix("#/$defs/") {
                        *r = name.to_string();
                    }
                }
                map.remove("required");
                // Convert const → single-element enum
                if let Some(const_val) = map.remove("const") {
                    map.insert(
                        "enum".to_string(),
                        serde_json::Value::Array(vec![const_val]),
                    );
                }
                // Convert type: [T, "null"] → anyOf: [{type: T, ...constraints}, {type: "null"}]
                if let Some(serde_json::Value::Array(types)) = map.get("type") {
                    let types: Vec<String> = types
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                    let non_null: Vec<&str> =
                        types.iter().map(|s| s.as_str()).filter(|t| *t != "null").collect();
                    let has_null = types.iter().any(|t| t == "null");
                    if has_null && non_null.len() == 1 {
                        map.remove("type");
                        // Partition siblings: type-specific constraints go on the inner
                        // schema, metadata (description, default) stays on the outer.
                        let mut inner = serde_json::Map::new();
                        inner.insert(
                            "type".to_string(),
                            serde_json::Value::String(non_null[0].to_string()),
                        );
                        let constraint_keys: &[&str] = &[
                            "items",
                            "properties",
                            "additionalProperties",
                            "minimum",
                            "maximum",
                            "format",
                            "pattern",
                            "minItems",
                            "maxItems",
                            "enum",
                        ];
                        for &key in constraint_keys {
                            if let Some(v) = map.remove(key) {
                                inner.insert(key.to_string(), v);
                            }
                        }
                        let null_variant = serde_json::json!({"type": "null"});
                        map.insert(
                            "anyOf".to_string(),
                            serde_json::Value::Array(vec![
                                serde_json::Value::Object(inner),
                                null_variant,
                            ]),
                        );
                    }
                }
                // Resolve numeric format → minimum/maximum, then delete format
                match map.get("type").and_then(|t| t.as_str()) {
                    Some("integer") => {
                        let format = map.remove("format")
                            .and_then(|v| v.as_str().map(String::from));
                        let (default_min, default_max): (i128, i128) = match format.as_deref() {
                            Some("int8") => (i8::MIN as i128, i8::MAX as i128),
                            Some("int16") => (i16::MIN as i128, i16::MAX as i128),
                            Some("int32") | Some("int") => (i32::MIN as i128, i32::MAX as i128),
                            Some("int64") | None => (i64::MIN as i128, i64::MAX as i128),
                            Some("int128") => (i128::MIN, i128::MAX),
                            Some("uint8") => (u8::MIN as i128, u8::MAX as i128),
                            Some("uint16") => (u16::MIN as i128, u16::MAX as i128),
                            Some("uint32") | Some("uint") => (u32::MIN as i128, u32::MAX as i128),
                            Some("uint64") => (u64::MIN as i128, u64::MAX as i128),
                            Some("uint128") => (u128::MIN as i128, i128::MAX), // u128::MAX exceeds i128
                            Some(_) => (i64::MIN as i128, i64::MAX as i128),
                        };
                        if !map.contains_key("minimum") {
                            map.insert("minimum".to_string(), serde_json::json!(default_min));
                        }
                        if !map.contains_key("maximum") {
                            map.insert("maximum".to_string(), serde_json::json!(default_max));
                        }
                    }
                    Some("number") => {
                        map.remove("format");
                        if !map.contains_key("minimum") {
                            map.insert("minimum".to_string(), serde_json::json!(f32::MIN));
                        }
                        if !map.contains_key("maximum") {
                            map.insert("maximum".to_string(), serde_json::json!(f32::MAX));
                        }
                    }
                    _ => {}
                }
            }
            for (k, v) in map.iter_mut() {
                // Keys inside "properties" are field names, not JSON Schema keywords
                normalize(v, k == "properties", title);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                normalize(v, false, title);
            }
        }
        _ => {}
    }
}

const KEYWORD_ORDER: &[&str] = &[
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

fn order_keys(value: &mut serde_json::Value, inside_properties: bool) {
    match value {
        serde_json::Value::Object(map) => {
            // Recurse first
            for (k, v) in map.iter_mut() {
                order_keys(v, k == "properties");
            }
            // Reorder this object's keys
            let entries: Vec<(String, serde_json::Value)> = map.into_iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            map.clear();
            if inside_properties {
                // Property field names: sort alphabetically
                let mut sorted = entries;
                sorted.sort_by(|(a, _), (b, _)| a.cmp(b));
                for (k, v) in sorted {
                    map.insert(k, v);
                }
            } else {
                // Schema keywords: sort by canonical order
                let mut sorted = entries;
                sorted.sort_by_key(|(k, _)| {
                    KEYWORD_ORDER.iter().position(|kw| kw == k).unwrap_or(usize::MAX)
                });
                for (k, v) in sorted {
                    map.insert(k, v);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                order_keys(v, false);
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

        normalize(&mut json, false, &title);
        order_keys(&mut json, false);

        let filename = format!("{title}.json");
        let path = out_dir.join(&filename);
        let contents = serde_json::to_string_pretty(&json).unwrap();
        fs::write(&path, format!("{contents}\n")).unwrap();
        count += 1;
    }

    println!("Wrote {count} schema files to {}", out_dir.display());
}
