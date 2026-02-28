use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaCitationCharLocationType {
    CharLocation,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaCitationCharLocation {
    pub cited_text: String,
    pub document_index: f64,
    pub document_title: Option<String>,
    pub end_char_index: f64,
    pub file_id: Option<String>,
    pub start_char_index: f64,
    pub r#type: BetaCitationCharLocationType,
}
