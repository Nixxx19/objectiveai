use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaServerToolUsage {
    pub web_fetch_requests: i64,
    pub web_search_requests: i64,
}
