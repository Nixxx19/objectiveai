use serde::{Deserialize, Serialize};

use super::{Ack, Config, Execution, Instructions, List, Log, Process, Resource, Schema};

/// Non-error output. Each variant nests a `subkind`-tagged enum that
/// pins down the exact shape.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Notification {
    Resource(Resource),
    List(List),
    Execution(Execution),
    Log(Log),
    Schema(Schema),
    Instructions(Instructions),
    Config(Config),
    Ack(Ack),
    Process(Process),
}
