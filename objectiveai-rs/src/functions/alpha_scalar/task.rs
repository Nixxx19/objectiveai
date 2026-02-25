use crate::{chat, functions};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BranchTaskExpression {
    #[serde(rename = "alpha.scalar.function")]
    ScalarFunction(ScalarFunctionTaskExpression),
    #[serde(rename = "alpha.placeholder.scalar.function")]
    PlaceholderScalarFunction(PlaceholderScalarFunctionTaskExpression),
}

impl BranchTaskExpression {
    pub fn transpile(self) -> functions::TaskExpression {
        match self {
            BranchTaskExpression::ScalarFunction(task) => {
                functions::TaskExpression::ScalarFunction(task.transpile())
            }
            BranchTaskExpression::PlaceholderScalarFunction(task) => {
                functions::TaskExpression::PlaceholderScalarFunction(
                    task.transpile(),
                )
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LeafTaskExpression {
    #[serde(rename = "alpha.vector.completion")]
    VectorCompletion(VectorCompletionTaskExpression),
}

impl LeafTaskExpression {
    pub fn transpile(self) -> functions::TaskExpression {
        match self {
            LeafTaskExpression::VectorCompletion(task) => {
                functions::TaskExpression::VectorCompletion(task.transpile())
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalarFunctionTaskExpression {
    pub remote: functions::Remote,
    pub owner: String,
    pub repository: String,
    pub commit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<functions::expression::Expression>,
    pub input: super::expression::ScalarFunctionInputExpression,
}

impl ScalarFunctionTaskExpression {
    pub fn transpile(self) -> functions::ScalarFunctionTaskExpression {
        functions::ScalarFunctionTaskExpression {
            remote: self.remote,
            owner: self.owner,
            repository: self.repository,
            commit: self.commit,
            skip: self.skip,
            map: None,
            input:
                super::expression::scalar_function_input_expression::transpile(
                    self.input,
                ),
            output: functions::expression::Expression::Special(
                functions::expression::Special::Output,
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceholderScalarFunctionTaskExpression {
    pub depth: u64,
    pub min_branch_width: u64,
    pub max_branch_width: u64,
    pub min_leaf_width: u64,
    pub max_leaf_width: u64,
    pub spec: String,
    pub input_schema: super::expression::ScalarFunctionInputSchema,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<functions::expression::Expression>,
    pub input: super::expression::ScalarFunctionInputExpression,
}

impl PlaceholderScalarFunctionTaskExpression {
    pub fn transpile(
        self,
    ) -> functions::PlaceholderScalarFunctionTaskExpression {
        functions::PlaceholderScalarFunctionTaskExpression {
            input_schema:
                super::expression::scalar_function_input_schema::transpile(
                    self.input_schema,
                ),
            skip: self.skip,
            map: None,
            input:
                super::expression::scalar_function_input_expression::transpile(
                    self.input,
                ),
            output: functions::expression::Expression::Special(
                functions::expression::Special::Output,
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorCompletionTaskExpression {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<functions::expression::Expression>,
    pub messages: functions::expression::Expression,
    pub responses: Vec<chat::completions::request::RichContent>,
}

impl VectorCompletionTaskExpression {
    pub fn transpile(self) -> functions::VectorCompletionTaskExpression {
        functions::VectorCompletionTaskExpression {
            skip: self.skip,
            map: None,
            messages: functions::expression::WithExpression::Expression(
                self.messages,
            ),
            tools: None,
            responses: functions::expression::WithExpression::Value(
                self.responses
                    .into_iter()
                    .map(chat::completions::request::RichContentExpression::from)
                    .map(functions::expression::WithExpression::Value)
                    .collect(),
            ),
            output: functions::expression::Expression::Special(
                functions::expression::Special::VectorCompletionScoresWeightedSum,
            ),
        }
    }
}
