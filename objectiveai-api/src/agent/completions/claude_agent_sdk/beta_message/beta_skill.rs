use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaSkillType {
    Anthropic,
    Custom,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaSkill {
    pub skill_id: String,
    pub r#type: BetaSkillType,
    pub version: String,
}
