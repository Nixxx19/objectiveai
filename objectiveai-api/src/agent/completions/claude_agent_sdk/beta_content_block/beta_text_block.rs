use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaTextBlockType {
    Text,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaTextBlock {
    pub citations: Option<Vec<super::super::beta_text_citation::BetaTextCitation>>,
    pub text: String,
    pub r#type: BetaTextBlockType,
}
