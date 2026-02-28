use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaCitationContentBlockLocationType {
    ContentBlockLocation,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaCitationContentBlockLocation {
    pub cited_text: String,
    pub document_index: i64,
    pub document_title: Option<String>,
    pub end_block_index: i64,
    pub file_id: Option<String>,
    pub start_block_index: i64,
    pub r#type: BetaCitationContentBlockLocationType,
}
