use crate::functions;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BranchTaskExpression {
    #[serde(rename = "alpha.scalar.function")]
    ScalarFunction(ScalarFunctionTaskExpression),
    #[serde(rename = "alpha.vector.function")]
    VectorFunction(VectorFunctionTaskExpression),
    #[serde(rename = "placeholder.alpha.scalar.function")]
    PlaceholderScalarFunction(PlaceholderScalarFunctionTaskExpression),
    #[serde(rename = "placeholder.alpha.vector.function")]
    PlaceholderVectorFunction(PlaceholderVectorFunctionTaskExpression),
}

impl BranchTaskExpression {
    pub fn url(&self) -> Option<String> {
        match self {
            BranchTaskExpression::ScalarFunction(task) => Some(task.url()),
            BranchTaskExpression::VectorFunction(task) => Some(task.url()),
            BranchTaskExpression::PlaceholderScalarFunction(_) => None,
            BranchTaskExpression::PlaceholderVectorFunction(_) => None,
        }
    }

    pub fn transpile(self) -> functions::TaskExpression {
        match self {
            BranchTaskExpression::ScalarFunction(task) => {
                functions::TaskExpression::ScalarFunction(task.transpile())
            }
            BranchTaskExpression::VectorFunction(task) => {
                functions::TaskExpression::VectorFunction(task.transpile())
            }
            BranchTaskExpression::PlaceholderScalarFunction(task) => {
                functions::TaskExpression::PlaceholderScalarFunction(
                    task.transpile(),
                )
            }
            BranchTaskExpression::PlaceholderVectorFunction(task) => {
                functions::TaskExpression::PlaceholderVectorFunction(
                    task.transpile(),
                )
            }
        }
    }

    pub fn is_placeholder(&self) -> bool {
        match self {
            BranchTaskExpression::ScalarFunction(_) => false,
            BranchTaskExpression::VectorFunction(_) => false,
            BranchTaskExpression::PlaceholderScalarFunction(_) => true,
            BranchTaskExpression::PlaceholderVectorFunction(_) => true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PartialPlaceholderBranchTaskExpression {
    #[serde(rename = "placeholder.alpha.scalar.function")]
    PlaceholderScalarFunction(PartialPlaceholderScalarFunctionTaskExpression),
    #[serde(rename = "placeholder.alpha.vector.function")]
    PlaceholderVectorFunction(PartialPlaceholderVectorFunctionTaskExpression),
}

impl PartialPlaceholderBranchTaskExpression {
    pub fn complete(
        self,
        depth: u64,
        min_branch_width: u64,
        max_branch_width: u64,
        min_leaf_width: u64,
        max_leaf_width: u64,
    ) -> BranchTaskExpression {
        match self {
            PartialPlaceholderBranchTaskExpression::PlaceholderScalarFunction(
                task,
            ) => BranchTaskExpression::PlaceholderScalarFunction(
                task.complete(
                    depth,
                    min_branch_width,
                    max_branch_width,
                    min_leaf_width,
                    max_leaf_width,
                ),
            ),
            PartialPlaceholderBranchTaskExpression::PlaceholderVectorFunction(
                task,
            ) => BranchTaskExpression::PlaceholderVectorFunction(
                task.complete(
                    depth,
                    min_branch_width,
                    max_branch_width,
                    min_leaf_width,
                    max_leaf_width,
                ),
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LeafTaskExpression {
    #[serde(rename = "vector.completion")]
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
    pub fn url(&self) -> String {
        self.remote.url(&self.owner, &self.repository, &self.commit)
    }

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
                functions::expression::Special::L1NormalizedFunctionOutput,
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorFunctionTaskExpression {
    pub remote: functions::Remote,
    pub owner: String,
    pub repository: String,
    pub commit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<functions::expression::Expression>,
    pub input: super::expression::VectorFunctionInputExpression,
}

impl VectorFunctionTaskExpression {
    pub fn url(&self) -> String {
        self.remote.url(&self.owner, &self.repository, &self.commit)
    }

    pub fn transpile(self) -> functions::VectorFunctionTaskExpression {
        functions::VectorFunctionTaskExpression {
            remote: self.remote,
            owner: self.owner,
            repository: self.repository,
            commit: self.commit,
            skip: self.skip,
            map: None,
            input: self.input.transpile(),
            output: functions::expression::Expression::Special(
                functions::expression::Special::Output,
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceholderScalarFunctionTaskExpression {
    #[serde(flatten)]
    pub params: functions::inventions::Params,
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
                functions::expression::Special::L1NormalizedFunctionOutput,
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialPlaceholderScalarFunctionTaskExpression {
    pub name: String,
    pub spec: String,
    pub input_schema: super::expression::ScalarFunctionInputSchema,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<functions::expression::Expression>,
    pub input: super::expression::ScalarFunctionInputExpression,
}

impl PartialPlaceholderScalarFunctionTaskExpression {
    pub fn complete(
        self,
        depth: u64,
        min_branch_width: u64,
        max_branch_width: u64,
        min_leaf_width: u64,
        max_leaf_width: u64,
    ) -> PlaceholderScalarFunctionTaskExpression {
        PlaceholderScalarFunctionTaskExpression {
            params: functions::inventions::Params {
                depth,
                min_branch_width,
                max_branch_width,
                min_leaf_width,
                max_leaf_width,
                name: self.name,
                spec: self.spec,
            },
            input_schema: self.input_schema,
            skip: self.skip,
            input: self.input,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceholderVectorFunctionTaskExpression {
    #[serde(flatten)]
    pub params: functions::inventions::Params,
    pub input_schema: super::expression::VectorFunctionInputSchema,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<functions::expression::Expression>,
    pub input: super::expression::VectorFunctionInputExpression,
}

impl PlaceholderVectorFunctionTaskExpression {
    pub fn transpile(
        self,
    ) -> functions::PlaceholderVectorFunctionTaskExpression {
        functions::PlaceholderVectorFunctionTaskExpression {
            input_schema: self.input_schema.transpile(),
            output_length: functions::expression::Expression::Special(
                functions::expression::Special::InputItemsOutputLength,
            ),
            input_split: functions::expression::Expression::Special(
                functions::expression::Special::InputItemsOptionalContextSplit,
            ),
            input_merge: functions::expression::Expression::Special(
                functions::expression::Special::InputItemsOptionalContextMerge,
            ),
            skip: self.skip,
            map: None,
            input: self.input.transpile(),
            output: functions::expression::Expression::Special(
                functions::expression::Special::Output,
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialPlaceholderVectorFunctionTaskExpression {
    pub name: String,
    pub spec: String,
    pub input_schema: super::expression::VectorFunctionInputSchema,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<functions::expression::Expression>,
    pub input: super::expression::VectorFunctionInputExpression,
}

impl PartialPlaceholderVectorFunctionTaskExpression {
    pub fn complete(
        self,
        depth: u64,
        min_branch_width: u64,
        max_branch_width: u64,
        min_leaf_width: u64,
        max_leaf_width: u64,
    ) -> PlaceholderVectorFunctionTaskExpression {
        PlaceholderVectorFunctionTaskExpression {
            params: functions::inventions::Params {
                depth,
                min_branch_width,
                max_branch_width,
                min_leaf_width,
                max_leaf_width,
                name: self.name,
                spec: self.spec,
            },
            input_schema: self.input_schema,
            skip: self.skip,
            input: self.input,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorCompletionTaskExpression {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<functions::expression::Expression>,
    pub messages: functions::expression::Expression,
    pub responses: functions::expression::Expression,
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
            responses: functions::expression::WithExpression::Expression(
                self.responses,
            ),
            output: functions::expression::Expression::Special(
                functions::expression::Special::VectorCompletionScores,
            ),
        }
    }
}
