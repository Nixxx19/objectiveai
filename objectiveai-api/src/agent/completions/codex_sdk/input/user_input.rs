use serde::{Deserialize, Serialize};

/// Externally-tagged input variant. The `type` discriminator is `text` or
/// `local_image`, matching the wire format produced by `_to_wire_item` in the
/// Python SDK (`utils.py`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserInput {
    Text(super::TextInput),
    LocalImage(super::LocalImageInput),
}
