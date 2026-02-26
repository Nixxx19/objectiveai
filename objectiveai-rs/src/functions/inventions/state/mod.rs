mod alpha_scalar_branch_state;
mod alpha_scalar_leaf_state;
mod alpha_vector_branch_state;
mod alpha_vector_leaf_state;
mod params;

pub use alpha_scalar_branch_state::*;
pub use alpha_scalar_leaf_state::*;
pub use alpha_vector_branch_state::*;
pub use alpha_vector_leaf_state::*;
pub use params::*;

use serde::{Deserialize, Serialize};

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
pub enum StateParam {
    #[serde(rename = "alpha.scalar.branch.function")]
    AlphaScalarBranch(AlphaScalarBranchState),
    #[serde(rename = "alpha.scalar.leaf.function")]
    AlphaScalarLeaf(AlphaScalarLeafState),
    #[serde(rename = "alpha.vector.branch.function")]
    AlphaVectorBranch(AlphaVectorBranchState),
    #[serde(rename = "alpha.vector.leaf.function")]
    AlphaVectorLeaf(AlphaVectorLeafState),
    #[serde(rename = "alpha.scalar.function")]
    AlphaScalar(Params),
    #[serde(rename = "alpha.vector.function")]
    AlphaVector(Params),
}

impl StateParam {
    pub fn route(self) -> State {
        match self {
            StateParam::AlphaScalarBranch(s) => {
                State::AlphaScalarBranch(s)
            }
            StateParam::AlphaScalarLeaf(s) => {
                State::AlphaScalarLeaf(s)
            }
            StateParam::AlphaVectorBranch(s) => {
                State::AlphaVectorBranch(s)
            }
            StateParam::AlphaVectorLeaf(s) => {
                State::AlphaVectorLeaf(s)
            }
            StateParam::AlphaScalar(params) => {
                if params.depth == 0 {
                    State::AlphaScalarLeaf(AlphaScalarLeafState {
                        params,
                        essay: None,
                        input_schema: None,
                        essay_tasks: None,
                        tasks: None,
                        description: None,
                    })
                } else {
                    State::AlphaScalarBranch(AlphaScalarBranchState {
                        params,
                        essay: None,
                        input_schema: None,
                        essay_tasks: None,
                        tasks: None,
                        description: None,
                    })
                }
            }
            StateParam::AlphaVector(params) => {
                if params.depth == 0 {
                    State::AlphaVectorLeaf(AlphaVectorLeafState {
                        params,
                        essay: None,
                        input_schema: None,
                        essay_tasks: None,
                        tasks: None,
                        description: None,
                    })
                } else {
                    State::AlphaVectorBranch(AlphaVectorBranchState {
                        params,
                        essay: None,
                        input_schema: None,
                        essay_tasks: None,
                        tasks: None,
                        description: None,
                    })
                }
            }
        }
    }
}
