use crate::functions;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RemoteFunction {
    #[serde(rename = "alpha.scalar.branch.function")]
    Branch {
        description: String,
        input_schema: super::expression::ScalarFunctionInputSchema,
        tasks: Vec<super::BranchTaskExpression>,
    },
    #[serde(rename = "alpha.scalar.leaf.function")]
    Leaf {
        description: String,
        input_schema: super::expression::ScalarFunctionInputSchema,
        tasks: Vec<super::LeafTaskExpression>,
    },
}

impl RemoteFunction {
    pub fn transpile(self) -> functions::RemoteFunction {
        match self {
            RemoteFunction::Branch {
                description,
                input_schema,
                tasks,
            } => functions::RemoteFunction::Scalar {
                description,
                input_schema:
                    super::expression::scalar_function_input_schema::transpile(
                        input_schema,
                    ),
                tasks: tasks
                    .into_iter()
                    .map(super::BranchTaskExpression::transpile)
                    .collect(),
            },
            RemoteFunction::Leaf {
                description,
                input_schema,
                tasks,
            } => functions::RemoteFunction::Scalar {
                description,
                input_schema:
                    super::expression::scalar_function_input_schema::transpile(
                        input_schema,
                    ),
                tasks: tasks
                    .into_iter()
                    .map(super::LeafTaskExpression::transpile)
                    .collect(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InlineFunction {
    #[serde(rename = "alpha.scalar.branch.function")]
    Branch {
        tasks: Vec<super::BranchTaskExpression>,
    },
    #[serde(rename = "alpha.scalar.leaf.function")]
    Leaf {
        tasks: Vec<super::LeafTaskExpression>,
    },
}

impl InlineFunction {
    pub fn transpile(self) -> functions::InlineFunction {
        match self {
            InlineFunction::Branch { tasks } => {
                functions::InlineFunction::Scalar {
                    tasks: tasks
                        .into_iter()
                        .map(super::BranchTaskExpression::transpile)
                        .collect(),
                }
            }
            InlineFunction::Leaf { tasks } => {
                functions::InlineFunction::Scalar {
                    tasks: tasks
                        .into_iter()
                        .map(super::LeafTaskExpression::transpile)
                        .collect(),
                }
            }
        }
    }
}