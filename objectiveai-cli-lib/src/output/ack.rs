use serde::{Deserialize, Serialize};

use super::FavoriteResource;

/// Silent successes — side-effect commands that previously emitted `"ok"`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "subkind", rename_all = "snake_case")]
pub enum Ack {
    /// Emitted by every `* config set` command.
    ConfigSet { key: String },
    /// Emitted by `<resource> favorites add`.
    FavoriteAdded {
        resource: FavoriteResource,
        path: objectiveai::RemotePathCommitOptional,
    },
    /// Emitted by `<resource> favorites del`.
    FavoriteRemoved {
        resource: FavoriteResource,
        name: String,
    },
    /// Emitted by `<resource> favorites edit`.
    FavoriteEdited {
        resource: FavoriteResource,
        name: String,
    },
    /// Emitted by `functions profiles pairs favorites add`.
    PairFavoriteAdded {
        function: objectiveai::RemotePathCommitOptional,
        profile: objectiveai::RemotePathCommitOptional,
    },
    /// Emitted by `functions profiles pairs favorites del`.
    PairFavoriteRemoved { name: String },
    /// Emitted by `functions profiles pairs favorites edit`.
    PairFavoriteEdited { name: String },
    /// Emitted by `instructions clear` and the per-scope `instructions clear`.
    InstructionsCleared,
    /// Emitted by `agents publish` (and any future `<resource> publish`).
    /// The SHA identifies the resulting commit on the local filesystem repo.
    Published { sha: String },
}
