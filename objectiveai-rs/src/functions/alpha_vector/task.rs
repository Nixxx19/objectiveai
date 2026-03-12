use crate::functions;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
#[schemars(rename = "functions.alpha_vector.BranchTaskExpression")]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
#[schemars(rename = "functions.alpha_vector.PartialPlaceholderBranchTaskExpression")]
pub enum PartialPlaceholderBranchTaskExpression {
    #[serde(rename = "placeholder.alpha.scalar.function")]
    PlaceholderScalarFunction(PartialPlaceholderScalarFunctionTaskExpression),
    #[serde(rename = "placeholder.alpha.vector.function")]
    PlaceholderVectorFunction(PartialPlaceholderVectorFunctionTaskExpression),
}

impl PartialPlaceholderBranchTaskExpression {
    pub fn complete(
        self,
        name: String,
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
                    name,
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
                    name,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
#[schemars(rename = "functions.alpha_vector.LeafTaskExpression")]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.alpha_vector.ScalarFunctionTaskExpression")]
pub struct ScalarFunctionTaskExpression {
    pub remote: functions::Remote,
    pub owner: String,
    pub repository: String,
    pub commit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<functions::expression::Expression>,
    pub input: super::expression::ScalarFunctionInputValueExpression,
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
                super::expression::scalar_function_input_value_expression::transpile(
                    self.input,
                ),
            output: functions::expression::Expression::Special(
                functions::expression::Special::TaskOutputL1Normalized,
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.alpha_vector.VectorFunctionTaskExpression")]
pub struct VectorFunctionTaskExpression {
    pub remote: functions::Remote,
    pub owner: String,
    pub repository: String,
    pub commit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<functions::expression::Expression>,
    pub input: super::expression::VectorFunctionInputValueExpression,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.alpha_vector.PlaceholderScalarFunctionTaskExpression")]
pub struct PlaceholderScalarFunctionTaskExpression {
    #[serde(flatten)]
    pub params: functions::inventions::Params,
    pub input_schema: super::expression::ScalarFunctionInputSchema,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<functions::expression::Expression>,
    pub input: super::expression::ScalarFunctionInputValueExpression,
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
            map: Some(functions::expression::Expression::Special(
                functions::expression::Special::InputItemsOutputLength,
            )),
            input:
                super::expression::scalar_function_input_value_expression::transpile(
                    self.input,
                ),
            output: functions::expression::Expression::Special(
                functions::expression::Special::TaskOutputL1Normalized,
            ),
        }
    }

    pub fn replace(
        self,
        path: &functions::RemoteFunctionPath,
    ) -> ScalarFunctionTaskExpression {
        ScalarFunctionTaskExpression {
            remote: path.remote,
            owner: path.owner.clone(),
            repository: path.repository.clone(),
            commit: path.commit.clone(),
            skip: self.skip,
            input: self.input,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.alpha_vector.PartialPlaceholderScalarFunctionTaskExpression")]
pub struct PartialPlaceholderScalarFunctionTaskExpression {
    pub spec: String,
    pub input_schema: super::expression::ScalarFunctionInputSchema,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<functions::expression::Expression>,
    pub input: super::expression::ScalarFunctionInputValueExpression,
}

impl PartialPlaceholderScalarFunctionTaskExpression {
    pub fn complete(
        self,
        name: String,
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
                name,
                spec: self.spec,
            },
            input_schema: self.input_schema,
            skip: self.skip,
            input: self.input,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.alpha_vector.PlaceholderVectorFunctionTaskExpression")]
pub struct PlaceholderVectorFunctionTaskExpression {
    #[serde(flatten)]
    pub params: functions::inventions::Params,
    pub input_schema: super::expression::VectorFunctionInputSchema,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<functions::expression::Expression>,
    pub input: super::expression::VectorFunctionInputValueExpression,
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

    pub fn replace(
        self,
        path: &functions::RemoteFunctionPath,
    ) -> VectorFunctionTaskExpression {
        VectorFunctionTaskExpression {
            remote: path.remote,
            owner: path.owner.clone(),
            repository: path.repository.clone(),
            commit: path.commit.clone(),
            skip: self.skip,
            input: self.input,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.alpha_vector.PartialPlaceholderVectorFunctionTaskExpression")]
pub struct PartialPlaceholderVectorFunctionTaskExpression {
    pub spec: String,
    pub input_schema: super::expression::VectorFunctionInputSchema,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<functions::expression::Expression>,
    pub input: super::expression::VectorFunctionInputValueExpression,
}

impl PartialPlaceholderVectorFunctionTaskExpression {
    pub fn complete(
        self,
        name: String,
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
                name,
                spec: self.spec,
            },
            input_schema: self.input_schema,
            skip: self.skip,
            input: self.input,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.alpha_vector.VectorCompletionTaskExpression")]
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
            responses: functions::expression::WithExpression::Expression(
                self.responses,
            ),
            output: functions::expression::Expression::Special(
                functions::expression::Special::Output,
            ),
        }
    }
}
