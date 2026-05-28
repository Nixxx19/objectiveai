//! `Content` — polymorphic content shape. Mirrors
//! [`crate::agent::completions::message::RichContentLog`] /
//! [`crate::agent::completions::message::SimpleContentLog`]
//! flattened to bare [`Id`] paths.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Id;
use crate::agent::completions::message::{RichContentLog, SimpleContentLog};

/// Either a single file (one text part or one media part) or a
/// list of files (multi-part content). Untagged: distinguishable on
/// the wire because [`Id`] is a string and `Vec<Id>` is an array.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(rename = "filesystem.logs.queue.Content")]
pub enum Content {
    #[schemars(title = "One")]
    One(Id),
    #[schemars(title = "Many")]
    Many(Vec<Id>),
}

impl From<RichContentLog> for Content {
    fn from(l: RichContentLog) -> Self {
        match l {
            RichContentLog::Reference(r) => Content::One(r.into()),
            RichContentLog::Parts(rs) => {
                Content::Many(rs.into_iter().map(Id::from).collect())
            }
        }
    }
}

impl From<SimpleContentLog> for Content {
    fn from(l: SimpleContentLog) -> Self {
        match l {
            SimpleContentLog::Reference(r) => Content::One(r.into()),
            SimpleContentLog::Parts(rs) => {
                Content::Many(rs.into_iter().map(Id::from).collect())
            }
        }
    }
}
