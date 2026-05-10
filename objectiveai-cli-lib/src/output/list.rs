use serde::{Deserialize, Serialize};

/// Listings of resources, logs, schemas, or favorites.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "subkind", rename_all = "snake_case")]
pub enum List {
    /// Emitted by `agents list`.
    Agents {
        source: ListSource,
        items: Vec<ListItem>,
    },
    /// Emitted by `swarms list`.
    Swarms {
        source: ListSource,
        items: Vec<ListItem>,
    },
    /// Emitted by `functions list`.
    Functions {
        source: ListSource,
        items: Vec<ListItem>,
    },
    /// Emitted by `functions profiles list`.
    Profiles {
        source: ListSource,
        items: Vec<ListItem>,
    },
    /// Emitted by `functions profiles pairs list`.
    Pairs {
        source: ListSource,
        items: Vec<PairListItem>,
    },
    /// Emitted by `<scope> logs list` and the global `logs list`.
    Logs {
        items: Vec<objectiveai::filesystem::logs::ListItem>,
    },
    /// Emitted by `schemas list` and the per-category `schemas <category> list`.
    Schemas { names: Vec<String> },
    /// Emitted by `<resource> favorites get`. The `resource` field tells
    /// the consumer which favorite list this is.
    Favorites {
        resource: FavoriteResource,
        items: Vec<objectiveai::filesystem::config::Favorite>,
    },
    /// Emitted by `functions profiles pairs favorites get`. Pair
    /// favorites have a different shape (two paths instead of one) so
    /// they get their own variant.
    PairFavorites {
        items: Vec<objectiveai::filesystem::config::PairFavorite>,
    },
}

/// Where a listing was fetched from. Mirrors `objectiveai_cli::list::Source`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ListSource {
    Filesystem,
    Favorites,
    Objectiveai,
    Mock,
    All,
}

/// One entry in a non-pair listing — either a favorite reference or a
/// resolved remote path. Untagged so the wire shape is whichever
/// underlying object matches.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ListItem {
    Favorite(objectiveai::filesystem::config::Favorite),
    Path(objectiveai::RemotePath),
}

/// One entry in a function-profile pair listing.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PairListItem {
    Favorite(objectiveai::filesystem::config::PairFavorite),
    Item(objectiveai::functions::response::ListFunctionProfilePairItem),
}

/// Which favorite collection a Favorites notification (or Ack) refers to.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FavoriteResource {
    Agent,
    Swarm,
    Function,
    Profile,
}
