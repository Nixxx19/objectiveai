//! `RichContentLog` — on-disk shape of message content, with
//! extractable media replaced by [`LogReference`] entries.
//!
//! Mirrors [`super::RichContent`]'s untagged Text/Parts split:
//!
//! - `Text(String)` serializes as a plain JSON string (used when the
//!   message has no parts to extract).
//! - `Parts(Vec<RichContentLogPart>)` serializes as a JSON array
//!   whose entries are either a reference object
//!   (`{"type":"reference","path":"..."}`) for extracted media, or
//!   the original [`super::RichContentPart`] (`{"type":"image_url",
//!   "image_url":{...}}` etc.) for parts that stay inline.

use schemars::JsonSchema;
use serde::Serialize;

use crate::filesystem::logs::LogReference;

use super::RichContentPart;

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(untagged)]
#[schemars(rename = "agent.completions.message.RichContentLog")]
pub enum RichContentLog {
    #[schemars(title = "Text")]
    Text(String),
    #[schemars(title = "Parts")]
    Parts(Vec<RichContentLogPart>),
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(untagged)]
#[schemars(rename = "agent.completions.message.RichContentLogPart")]
pub enum RichContentLogPart {
    #[schemars(title = "Reference")]
    Reference(LogReference),
    #[schemars(title = "Original")]
    Original(RichContentPart),
}
