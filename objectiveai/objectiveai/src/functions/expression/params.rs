use crate::vector;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Params<'i, 't, 'to, 'm> {
    Owned(ParamsOwned),
    Ref(ParamsRef<'i, 't, 'to, 'm>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamsOwned {
    pub input: super::Input,
    pub tasks: Vec<Option<TaskOutputOwned>>, // only provided to output expressions
    pub map: Option<super::Input>, // only provided to task expressions, other than skip
}

#[derive(Debug, Clone, Serialize)]
pub struct ParamsRef<'i, 't, 'to, 'm> {
    pub input: &'i super::Input,
    pub tasks: &'t [Option<TaskOutput<'to>>], // only provided to output expressions
    pub map: Option<&'m super::Input>, // only provided to task expressions, other than skip
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum TaskOutput<'a> {
    Owned(TaskOutputOwned),
    Ref(TaskOutputRef<'a>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TaskOutputOwned {
    Function(FunctionOutput),
    MapFunction(Vec<FunctionOutput>),
    VectorCompletion(VectorCompletionOutput),
    MapVectorCompletion(Vec<VectorCompletionOutput>),
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum TaskOutputRef<'a> {
    Function(&'a FunctionOutput),
    MapFunction(&'a [FunctionOutput]),
    VectorCompletion(&'a VectorCompletionOutput),
    MapVectorCompletion(&'a [VectorCompletionOutput]),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorCompletionOutput {
    pub votes: Vec<vector::completions::response::Vote>,
    pub scores: Vec<rust_decimal::Decimal>,
    pub weights: Vec<rust_decimal::Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionOutput {
    Scalar(rust_decimal::Decimal),
    Vector(Vec<rust_decimal::Decimal>),
    Err(serde_json::Value),
}
