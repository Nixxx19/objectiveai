use serde::{Deserialize, Serialize};

/// Terminal result of a `*/create` endpoint.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "subkind", rename_all = "snake_case")]
pub enum Execution {
    /// Emitted by `functions executions create`.
    Function(Box<objectiveai::functions::executions::response::unary::FunctionExecution>),
    /// Emitted by `laboratories executions create`.
    Laboratory(Box<objectiveai::laboratories::executions::response::unary::LaboratoryExecution>),
    /// Emitted by `functions inventions create`.
    InventionCreate(Box<objectiveai::functions::inventions::response::unary::FunctionInvention>),
    /// Emitted by `functions inventions recursive create`.
    InventionRecursiveCreate(
        Box<
            objectiveai::functions::inventions::recursive::response::unary::FunctionInventionRecursive,
        >,
    ),
    /// Emitted by `vector completions`.
    VectorCompletion(Box<objectiveai::vector::completions::response::unary::VectorCompletion>),
}
