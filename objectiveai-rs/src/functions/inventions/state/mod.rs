mod alpha_scalar_branch_state;
mod alpha_scalar_leaf_state;
mod alpha_scalar_state;
mod alpha_vector_branch_state;
mod alpha_vector_leaf_state;
mod alpha_vector_state;
mod params;
mod readme;

pub use alpha_scalar_branch_state::*;
pub use alpha_scalar_leaf_state::*;
pub use alpha_scalar_state::*;
pub use alpha_vector_branch_state::*;
pub use alpha_vector_leaf_state::*;
pub use alpha_vector_state::*;
pub use params::*;

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

/// Constructs a child name by appending the task index to the parent's path.
///
/// Splits `parent_name` by `-`, takes the last segment, and tries to decode
/// it as a base62 path. If successful, pushes `task_index` and re-encodes.
/// If not, appends a new `-` segment with just the index encoded.
fn child_name(parent_name: &str, task_index: usize) -> String {
    if let Some((prefix, last)) = parent_name.rsplit_once('-') {
        if let Ok(mut path) = super::path::b62_to_path::<u64>(last) {
            path.push(task_index as u64);
            if let Ok(b62) = super::path::path_to_b62(&path) {
                return format!("{}-{}", prefix, b62);
            }
        }
    }
    // Couldn't parse existing path segment — start a new one.
    let path = [task_index as u64];
    let b62 = super::path::path_to_b62(&path).unwrap_or_else(|_| format!("{}", task_index));
    format!("{}-{}", parent_name, b62)
}

/// Fixes a task name after reindexing (e.g. after a delete).
///
/// Tries to parse the last `-` segment as a base62 path. If successful,
/// pops the last element and pushes `new_index`. If parsing fails, leaves
/// the name unchanged.
fn reindex_name(name: &mut String, new_index: usize) {
    if let Some((prefix, last)) = name.rsplit_once('-') {
        if let Ok(mut path) = super::path::b62_to_path::<u64>(last) {
            if !path.is_empty() {
                path.pop();
                path.push(new_index as u64);
                if let Ok(b62) = super::path::path_to_b62(&path) {
                    *name = format!("{}-{}", prefix, b62);
                }
            }
        }
    }
}

/// Abstracts over the 4 routed state variants for invention step orchestration.
pub trait InventionState: Clone + Send + 'static {
    fn params(this: &Arc<Mutex<Self>>) -> Params;
    fn is_scalar() -> bool;
    fn object() -> super::response::streaming::Object;
    fn into_state(self) -> State;

    fn essay_tools(this: &Arc<Mutex<Self>>) -> Vec<super::InventionTool>;
    fn validate_essay(this: &Arc<Mutex<Self>>) -> Result<(), String>;

    fn input_schema_tools(this: &Arc<Mutex<Self>>) -> Vec<super::InventionTool>;
    fn validate_input_schema(this: &Arc<Mutex<Self>>) -> Result<(), String>;

    fn essay_tasks_tools(this: &Arc<Mutex<Self>>) -> Vec<super::InventionTool>;
    fn validate_essay_tasks(this: &Arc<Mutex<Self>>) -> Result<(), String>;

    fn tasks_tools(this: &Arc<Mutex<Self>>) -> Vec<super::InventionTool>;
    fn validate_function(this: &Arc<Mutex<Self>>) -> Result<(), String>;
    fn build_function(this: &Arc<Mutex<Self>>) -> Option<crate::functions::FullRemoteFunction>;

    fn description_tools(this: &Arc<Mutex<Self>>) -> Vec<super::InventionTool>;
    fn validate_description(this: &Arc<Mutex<Self>>) -> Result<(), String>;

    fn write_readme(this: &Arc<Mutex<Self>>);

    fn replace_placeholders(
        this: &Arc<Mutex<Self>>,
        paths: &[crate::functions::RemoteFunctionPath],
    );
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum State {
    #[serde(rename = "alpha.scalar.branch.function")]
    AlphaScalarBranch(AlphaScalarBranchState),
    #[serde(rename = "alpha.scalar.leaf.function")]
    AlphaScalarLeaf(AlphaScalarLeafState),
    #[serde(rename = "alpha.vector.branch.function")]
    AlphaVectorBranch(AlphaVectorBranchState),
    #[serde(rename = "alpha.vector.leaf.function")]
    AlphaVectorLeaf(AlphaVectorLeafState),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ParamsState {
    #[serde(rename = "alpha.scalar.branch.function")]
    AlphaScalarBranch(AlphaScalarBranchState),
    #[serde(rename = "alpha.scalar.leaf.function")]
    AlphaScalarLeaf(AlphaScalarLeafState),
    #[serde(rename = "alpha.vector.branch.function")]
    AlphaVectorBranch(AlphaVectorBranchState),
    #[serde(rename = "alpha.vector.leaf.function")]
    AlphaVectorLeaf(AlphaVectorLeafState),
    #[serde(
        rename = "alpha.scalar.function",
        alias = "placeholder.alpha.scalar.function"
    )]
    AlphaScalar(AlphaScalarState),
    #[serde(
        rename = "alpha.vector.function",
        alias = "placeholder.alpha.vector.function"
    )]
    AlphaVector(AlphaVectorState),
}

impl ParamsState {
    pub fn route(self) -> State {
        match self {
            ParamsState::AlphaScalarBranch(s) => State::AlphaScalarBranch(s),
            ParamsState::AlphaScalarLeaf(s) => State::AlphaScalarLeaf(s),
            ParamsState::AlphaVectorBranch(s) => State::AlphaVectorBranch(s),
            ParamsState::AlphaVectorLeaf(s) => State::AlphaVectorLeaf(s),
            ParamsState::AlphaScalar(s) => {
                if s.params.depth == 0 {
                    State::AlphaScalarLeaf(AlphaScalarLeafState {
                        params: s.params,
                        essay: None,
                        input_schema: s.input_schema,
                        essay_tasks: None,
                        tasks: None,
                        tasks_length: None,
                        description: None,
                        readme: None,
                    })
                } else {
                    State::AlphaScalarBranch(AlphaScalarBranchState {
                        params: s.params,
                        essay: None,
                        input_schema: s.input_schema,
                        essay_tasks: None,
                        tasks: None,
                        tasks_length: None,
                        description: None,
                        readme: None,
                    })
                }
            }
            ParamsState::AlphaVector(s) => {
                if s.params.depth == 0 {
                    State::AlphaVectorLeaf(AlphaVectorLeafState {
                        params: s.params,
                        essay: None,
                        input_schema: s.input_schema,
                        essay_tasks: None,
                        tasks: None,
                        tasks_length: None,
                        description: None,
                        readme: None,
                    })
                } else {
                    State::AlphaVectorBranch(AlphaVectorBranchState {
                        params: s.params,
                        essay: None,
                        input_schema: s.input_schema,
                        essay_tasks: None,
                        tasks: None,
                        tasks_length: None,
                        description: None,
                        readme: None,
                    })
                }
            }
        }
    }
}
