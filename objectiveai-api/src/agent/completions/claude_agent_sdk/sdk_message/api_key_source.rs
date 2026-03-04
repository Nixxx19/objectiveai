use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApiKeySource {
    User,
    Project,
    Org,
    Temporary,
    #[serde(rename = "oauth")]
    OAuth,
}
