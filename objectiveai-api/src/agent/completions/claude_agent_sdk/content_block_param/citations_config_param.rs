use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CitationsConfigParam {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}
