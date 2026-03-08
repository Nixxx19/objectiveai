//! Random input schema generation for mock inventions.
//!
//! Generates diverse `ObjectInputSchema` and `VectorFunctionInputSchema` JSON
//! strings from an RNG. This enables the mock to produce arbitrary input schemas
//! rather than picking from a small hardcoded set, so task expression generators
//! must handle any schema they encounter.

use rand::Rng;

/// Pool of realistic property names, grouped by semantic category.
const PROPERTY_NAMES: &[&[&str]] = &[
    // Text content
    &["text", "content", "body", "message", "summary", "description", "title", "headline", "caption", "excerpt"],
    // Identifiers
    &["name", "label", "id", "slug", "key", "code", "tag", "category", "kind"],
    // Queries / prompts
    &["query", "prompt", "question", "topic", "subject", "criteria", "instruction"],
    // Metadata
    &["author", "source", "url", "language", "format", "version", "status"],
    // Numeric
    &["score", "rating", "count", "weight", "priority", "rank", "threshold", "limit"],
    // Boolean
    &["is_draft", "is_active", "enabled", "verified", "flagged", "approved"],
    // Media
    &["image", "photo", "thumbnail", "avatar", "icon", "banner"],
    &["audio", "recording", "clip", "track", "voice"],
    &["video", "footage", "stream", "animation"],
    &["file", "document", "attachment", "upload"],
    // Collections
    &["tags", "keywords", "labels", "categories", "items", "entries", "values"],
];

/// Property type with its JSON `"type"` string.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PropType {
    String,
    Number,
    Integer,
    Boolean,
    Image,
    Audio,
    Video,
    File,
    StringArray,
}

impl PropType {
    fn to_json(self) -> serde_json::Value {
        match self {
            PropType::String => serde_json::json!({"type": "string"}),
            PropType::Number => serde_json::json!({"type": "number"}),
            PropType::Integer => serde_json::json!({"type": "integer"}),
            PropType::Boolean => serde_json::json!({"type": "boolean"}),
            PropType::Image => serde_json::json!({"type": "image"}),
            PropType::Audio => serde_json::json!({"type": "audio"}),
            PropType::Video => serde_json::json!({"type": "video"}),
            PropType::File => serde_json::json!({"type": "file"}),
            PropType::StringArray => serde_json::json!({"type": "array", "items": {"type": "string"}}),
        }
    }

    /// Whether this type can be meaningfully converted to text via `str()`.
    pub(super) fn is_textual(self) -> bool {
        matches!(self, PropType::String | PropType::Number | PropType::Integer | PropType::Boolean)
    }

    /// Whether this is a media/multimodal type that must be passed directly
    /// as a content part (not wrapped in `str()`).
    pub(super) fn is_media(self) -> bool {
        matches!(self, PropType::Image | PropType::Audio | PropType::Video | PropType::File)
    }
}

/// Pick a property type appropriate for the given name.
pub(super) fn type_for_name(name: &str, rng: &mut impl Rng) -> PropType {
    // Media names get media types
    if ["image", "photo", "thumbnail", "avatar", "icon", "banner"].contains(&name) {
        return PropType::Image;
    }
    if ["audio", "recording", "clip", "track", "voice"].contains(&name) {
        return PropType::Audio;
    }
    if ["video", "footage", "stream", "animation"].contains(&name) {
        return PropType::Video;
    }
    if ["file", "document", "attachment", "upload"].contains(&name) {
        return PropType::File;
    }
    // Boolean names
    if name.starts_with("is_") || ["enabled", "verified", "flagged", "approved"].contains(&name) {
        return PropType::Boolean;
    }
    // Numeric names
    if ["score", "rating", "count", "weight", "priority", "rank", "threshold", "limit"].contains(&name) {
        return if rng.random_range(0u32..2) == 0 { PropType::Number } else { PropType::Integer };
    }
    // Collection names
    if ["tags", "keywords", "labels", "categories", "entries", "values"].contains(&name) {
        return PropType::StringArray;
    }
    // Default: mostly string, sometimes number
    match rng.random_range(0u32..10) {
        0 => PropType::Number,
        1 => PropType::Integer,
        2 => PropType::Boolean,
        _ => PropType::String,
    }
}

/// Pick `n` unique property names from the pool.
fn pick_names(n: usize, rng: &mut impl Rng) -> Vec<&'static str> {
    let flat: Vec<&str> = PROPERTY_NAMES.iter().flat_map(|g| g.iter().copied()).collect();
    let mut selected = Vec::with_capacity(n);
    let mut indices: Vec<usize> = (0..flat.len()).collect();
    for i in 0..n.min(flat.len()) {
        let j = rng.random_range(i..indices.len());
        indices.swap(i, j);
        selected.push(flat[indices[i]]);
    }
    selected
}

/// Generate a random `ObjectInputSchema` as a JSON string.
///
/// Produces schemas with 1–5 properties, random types, and a random subset
/// marked as required (always at least 1 required, always at least 1 textual
/// required field so that message expressions work).
pub fn random_scalar_input_schema(rng: &mut impl Rng) -> String {
    let n_props = rng.random_range(1u32..=5) as usize;
    let names = pick_names(n_props, rng);

    let mut properties = serde_json::Map::new();
    let mut types: Vec<PropType> = Vec::with_capacity(n_props);
    for &name in &names {
        let pt = type_for_name(name, rng);
        types.push(pt);
        let mut schema = pt.to_json();
        // Occasionally add a description
        if rng.random_range(0u32..3) == 0 {
            schema.as_object_mut().unwrap().insert(
                "description".into(),
                serde_json::Value::String(format!("The {name}")),
            );
        }
        properties.insert(name.to_string(), schema);
    }

    // Required: at least 1, up to all. Must include at least 1 textual field
    // and ALL media fields (so expressions can reference them and AV18/AS20 pass).
    let n_required = rng.random_range(1..=names.len());
    let mut required: Vec<&str> = names[..n_required].to_vec();

    // Ensure all media fields are required
    for (i, &name) in names.iter().enumerate() {
        if types[i].is_media() && !required.contains(&name) {
            required.push(name);
        }
    }

    // Ensure at least one textual required field
    let has_textual = required.iter().any(|&r| {
        names.iter().position(|&n| n == r).map(|i| types[i].is_textual()).unwrap_or(false)
    });
    if !has_textual {
        // Find a textual field and swap it in
        if let Some(ti) = types.iter().position(|t| t.is_textual()) {
            if !required.contains(&names[ti]) {
                required.push(names[ti]);
            }
        } else {
            // No textual fields at all — replace the first with a string
            properties.insert(names[0].to_string(), serde_json::json!({"type": "string"}));
            types[0] = PropType::String;
        }
    }

    let required_json: Vec<serde_json::Value> = required.iter()
        .map(|s| serde_json::Value::String(s.to_string()))
        .collect();

    serde_json::json!({
        "properties": properties,
        "required": required_json,
    }).to_string()
}

/// Generate a random `VectorFunctionInputSchema` as a JSON string.
///
/// The vector schema always has `items` (array with `minItems: 2`).
/// Optionally includes a `context` object with 1–3 properties.
///
/// Item types: plain strings, objects with properties, or media types.
pub fn random_vector_input_schema(rng: &mut impl Rng) -> String {
    let mut schema = serde_json::Map::new();

    // Optional context (50% chance)
    if rng.random_range(0u32..2) == 0 {
        let n_ctx = rng.random_range(1u32..=3) as usize;
        let ctx_names = pick_names(n_ctx, rng);
        let mut ctx_types = Vec::with_capacity(n_ctx);
        let mut ctx_props = serde_json::Map::new();
        for &name in &ctx_names {
            let pt = type_for_name(name, rng);
            ctx_types.push(pt);
            ctx_props.insert(name.to_string(), pt.to_json());
        }
        // At least 1 required, all media fields must be required
        let n_req = rng.random_range(1..=ctx_names.len());
        let mut ctx_required: Vec<&str> = ctx_names[..n_req].to_vec();
        for (i, &name) in ctx_names.iter().enumerate() {
            if ctx_types[i].is_media() && !ctx_required.contains(&name) {
                ctx_required.push(name);
            }
        }
        schema.insert("context".into(), serde_json::json!({
            "properties": ctx_props,
            "required": ctx_required,
        }));
    }

    // Items: choose item type
    let items_schema = match rng.random_range(0u32..5) {
        0 => {
            // Plain strings
            serde_json::json!({
                "type": "array",
                "minItems": 2,
                "items": {"type": "string", "description": random_item_description(rng)}
            })
        }
        1 => {
            // Image items
            serde_json::json!({
                "type": "array",
                "minItems": 2,
                "items": {"type": "image", "description": random_item_description(rng)}
            })
        }
        _ => {
            // Object items with 1-4 properties
            let n_item_props = rng.random_range(1u32..=4) as usize;
            let item_names = pick_names(n_item_props, rng);
            let mut item_types = Vec::with_capacity(n_item_props);
            let mut item_props = serde_json::Map::new();
            for &name in &item_names {
                let pt = type_for_name(name, rng);
                item_types.push(pt);
                item_props.insert(name.to_string(), pt.to_json());
            }
            // At least 1 required, all media fields must be required
            let n_req = rng.random_range(1..=item_names.len());
            let mut item_required: Vec<&str> = item_names[..n_req].to_vec();
            for (i, &name) in item_names.iter().enumerate() {
                if item_types[i].is_media() && !item_required.contains(&name) {
                    item_required.push(name);
                }
            }
            serde_json::json!({
                "type": "array",
                "minItems": 2,
                "items": {
                    "type": "object",
                    "properties": item_props,
                    "required": item_required,
                }
            })
        }
    };
    schema.insert("items".into(), items_schema);

    serde_json::to_string(&serde_json::Value::Object(schema)).unwrap()
}

fn random_item_description(rng: &mut impl Rng) -> &'static str {
    const DESCS: &[&str] = &[
        "An item to rank",
        "A candidate result",
        "An entry to evaluate",
        "A response option",
        "A submission to score",
        "An alternative to compare",
        "A candidate to assess",
        "A sample to judge",
    ];
    DESCS[rng.random_range(0..DESCS.len())]
}
