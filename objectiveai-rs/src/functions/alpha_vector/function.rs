use crate::functions;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, arbitrary::Arbitrary)]
#[serde(tag = "type")]
#[schemars(rename = "functions.alpha_vector.RemoteFunction")]
pub enum RemoteFunction {
    #[serde(rename = "alpha.vector.branch.function")]
    Branch {
        description: String,
        input_schema: super::expression::VectorFunctionInputSchema,
        tasks: Vec<super::BranchTaskExpression>,
    },
    #[serde(rename = "alpha.vector.leaf.function")]
    Leaf {
        description: String,
        input_schema: super::expression::VectorFunctionInputSchema,
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
            } => functions::RemoteFunction::Vector {
                description,
                input_schema: input_schema.transpile(),
                tasks: tasks
                    .into_iter()
                    .map(super::BranchTaskExpression::transpile)
                    .collect(),
                output_length: functions::expression::Expression::Special(
                    functions::expression::Special::InputItemsOutputLength,
                ),
                input_split: functions::expression::Expression::Special(
                    functions::expression::Special::InputItemsOptionalContextSplit,
                ),
                input_merge: functions::expression::Expression::Special(
                    functions::expression::Special::InputItemsOptionalContextMerge,
                ),
            },
            RemoteFunction::Leaf {
                description,
                input_schema,
                tasks,
            } => functions::RemoteFunction::Vector {
                description,
                input_schema: input_schema.transpile(),
                tasks: tasks
                    .into_iter()
                    .map(super::LeafTaskExpression::transpile)
                    .collect(),
                output_length: functions::expression::Expression::Special(
                    functions::expression::Special::InputItemsOutputLength,
                ),
                input_split: functions::expression::Expression::Special(
                    functions::expression::Special::InputItemsOptionalContextSplit,
                ),
                input_merge: functions::expression::Expression::Special(
                    functions::expression::Special::InputItemsOptionalContextMerge,
                ),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, arbitrary::Arbitrary)]
#[serde(tag = "type")]
#[schemars(rename = "functions.alpha_vector.InlineFunction")]
pub enum InlineFunction {
    #[serde(rename = "alpha.vector.branch.function")]
    Branch {
        tasks: Vec<super::BranchTaskExpression>,
    },
    #[serde(rename = "alpha.vector.leaf.function")]
    Leaf {
        tasks: Vec<super::LeafTaskExpression>,
    },
}

impl InlineFunction {
    pub fn transpile(self) -> functions::InlineFunction {
        match self {
            InlineFunction::Branch { tasks } => {
                functions::InlineFunction::Vector {
                    tasks: tasks
                        .into_iter()
                        .map(super::BranchTaskExpression::transpile)
                        .collect(),
                    input_split: Some(functions::expression::Expression::Special(
                        functions::expression::Special::InputItemsOptionalContextSplit,
                    )),
                    input_merge: Some(functions::expression::Expression::Special(
                        functions::expression::Special::InputItemsOptionalContextMerge,
                    )),
                }
            }
            InlineFunction::Leaf { tasks } => {
                functions::InlineFunction::Vector {
                    tasks: tasks
                        .into_iter()
                        .map(super::LeafTaskExpression::transpile)
                        .collect(),
                    input_split: Some(functions::expression::Expression::Special(
                        functions::expression::Special::InputItemsOptionalContextSplit,
                    )),
                    input_merge: Some(functions::expression::Expression::Special(
                        functions::expression::Special::InputItemsOptionalContextMerge,
                    )),
                }
            }
        }
    }
}