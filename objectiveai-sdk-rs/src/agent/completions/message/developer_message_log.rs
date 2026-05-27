//! `DeveloperMessageLog` — on-disk shape of [`super::DeveloperMessage`].
//! `content` is replaced by [`super::SimpleContentLog`] (extracted-to-files);
//! all other fields stay inline.

use schemars::JsonSchema;
use serde::Serialize;

use super::SimpleContentLog;

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[schemars(rename = "agent.completions.message.DeveloperMessageLog")]
pub struct DeveloperMessageLog {
    pub content: SimpleContentLog,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub name: Option<String>,
}
