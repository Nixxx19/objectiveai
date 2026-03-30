//! Ensures every `Option` field with `#[serde(skip_serializing_if = "...")]`
//! also has `#[schemars(extend("omitempty" = true))]`.
//!
//! Without this, JSON schema consumers (e.g. Go's `json.Marshal` with
//! `omitempty`) omit nil fields, but snapshot JSON includes them as `null`.
//! The `omitempty` extension lets code generators know the field is
//! conditionally omitted during serialization.

use std::fs;
use std::path::Path;
use syn::{Fields, Item, Type, Visibility};
use walkdir::WalkDir;

/// Check if a type is `Option<...>`.
fn type_is_option(ty: &Type) -> bool {
    match ty {
        Type::Path(tp) => tp
            .path
            .segments
            .last()
            .map_or(false, |seg| seg.ident == "Option"),
        _ => false,
    }
}

/// Check if attributes contain `#[serde(skip_serializing_if = "...")]`.
fn has_skip_serializing_if(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("serde") {
            let tokens = attr
                .meta
                .require_list()
                .ok()
                .map(|list| list.tokens.to_string());
            tokens.map_or(false, |t| t.contains("skip_serializing_if"))
        } else {
            false
        }
    })
}

/// Check if attributes contain `#[schemars(extend("omitempty" = true))]`.
fn has_schemars_omitempty(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("schemars") {
            let tokens = attr
                .meta
                .require_list()
                .ok()
                .map(|list| list.tokens.to_string());
            tokens.map_or(false, |t| t.contains("omitempty"))
        } else {
            false
        }
    })
}

/// Check if an item derives Serialize.
fn has_serialize_derive(attrs: &[syn::Attribute]) -> bool {
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
                        .map_or(false, |last| last.trim() == "Serialize")
                })
            })
        } else {
            false
        }
    })
}

fn check_fields(
    fields: &Fields,
    type_name: &str,
    relative: &str,
    errors: &mut Vec<String>,
) {
    if let Fields::Named(named) = fields {
        for field in &named.named {
            if type_is_option(&field.ty)
                && has_skip_serializing_if(&field.attrs)
                && !has_schemars_omitempty(&field.attrs)
            {
                let field_name = field
                    .ident
                    .as_ref()
                    .map_or("?".to_string(), |i| i.to_string());
                errors.push(format!(
                    "{type_name}::{field_name} in {relative} has skip_serializing_if but is missing #[schemars(extend(\"omitempty\" = true))]"
                ));
            }
        }
    }
}

#[test]
fn all_optional_skip_fields_have_schemars_omitempty() {
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

        let source = fs::read_to_string(path).unwrap();
        let file = match syn::parse_file(&source) {
            Ok(f) => f,
            Err(_) => continue,
        };

        for item in &file.items {
            match item {
                Item::Struct(s) if matches!(s.vis, Visibility::Public(_)) => {
                    let name = s.ident.to_string();
                    if !has_serialize_derive(&s.attrs) {
                        continue;
                    }
                    check_fields(&s.fields, &name, &relative, &mut errors);
                }
                Item::Enum(e) if matches!(e.vis, Visibility::Public(_)) => {
                    let name = e.ident.to_string();
                    if !has_serialize_derive(&e.attrs) {
                        continue;
                    }
                    for variant in &e.variants {
                        let variant_name = format!("{}::{}", name, variant.ident);
                        check_fields(&variant.fields, &variant_name, &relative, &mut errors);
                    }
                }
                _ => {}
            }
        }
    }

    if !errors.is_empty() {
        panic!(
            "Option fields with skip_serializing_if missing #[schemars(extend(\"omitempty\" = true))] ({}):\n\n{}",
            errors.len(),
            errors.join("\n\n")
        );
    }
}
