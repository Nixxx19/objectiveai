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

use serde::Serialize;

use crate::filesystem::logs::LogReference;

use super::RichContentPart;

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum RichContentLog {
    Text(String),
    Parts(Vec<RichContentLogPart>),
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum RichContentLogPart {
    Reference(LogReference),
    Original(RichContentPart),
}
