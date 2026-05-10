use serde::{Deserialize, Serialize};

/// JSON Schema browsing output.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "subkind", rename_all = "snake_case")]
pub enum Schema {
    /// Emitted by every `schemas <category> <type> get` command. The
    /// schema is a real JSON Schema object, not a stringified blob.
    Get { name: String, schema: schemars::Schema },
}
