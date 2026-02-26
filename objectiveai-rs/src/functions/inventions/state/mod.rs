mod alpha_scalar_branch_state;
mod alpha_scalar_leaf_state;
mod alpha_vector_branch_state;
mod alpha_vector_leaf_state;

pub use alpha_scalar_branch_state::*;
pub use alpha_scalar_leaf_state::*;
pub use alpha_vector_branch_state::*;
pub use alpha_vector_leaf_state::*;

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
