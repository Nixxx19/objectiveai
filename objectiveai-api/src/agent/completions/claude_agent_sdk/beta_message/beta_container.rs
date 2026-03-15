use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaContainer {
    pub id: String,
    pub expires_at: String,
    pub skills: Option<Vec<super::BetaSkill>>,
}
