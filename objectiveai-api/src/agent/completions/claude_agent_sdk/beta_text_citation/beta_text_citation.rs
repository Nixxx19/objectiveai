use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum BetaTextCitation {
    CharLocation(super::BetaCitationCharLocation),
    PageLocation(super::BetaCitationPageLocation),
    ContentBlockLocation(super::BetaCitationContentBlockLocation),
    WebSearchResultLocation(super::BetaCitationsWebSearchResultLocation),
    SearchResultLocation(super::BetaCitationSearchResultLocation),
}
