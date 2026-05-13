use serde::{Deserialize, Serialize};

/// Generic wire wrapper for every list-style notification:
/// `{"type":"notification","items":[...]}`. The element type varies
/// (e.g. `Items<ListItem>` for `agents list`, `Items<PairListItem>` for
/// `pairs list`, `Items<objectiveai::filesystem::config::Favorite>` for
/// favorites listings, `Items<objectiveai::filesystem::logs::ListItem>`
/// for log listings).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Items<T> {
    pub items: Vec<T>,
}

/// One entry in a non-pair resource listing — either a favorite that
/// matches a remote resource or a resolved remote path. Untagged so the
/// wire shape is whichever underlying object matches.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ListItem {
    Favorite(objectiveai::filesystem::config::Favorite),
    Item(objectiveai::RemotePath),
}

/// One entry in a function-profile pair listing.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PairListItem {
    Favorite(objectiveai::filesystem::config::PairFavorite),
    Item(objectiveai::functions::response::ListFunctionProfilePairItem),
}
