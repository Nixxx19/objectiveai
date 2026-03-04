use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaCitationsDeltaType {
    CitationsDelta,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaCitationsDelta {
    pub citation: super::super::beta_text_citation::BetaTextCitation,
    pub r#type: BetaCitationsDeltaType,
}
