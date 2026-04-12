use crate::functions;
use crate::functions::expression::WithExpression;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The type of invention state a prompt applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "functions.inventions.prompt.StepPromptType")]
pub enum StepPromptType {
    #[serde(rename = "alpha.scalar.branch.function")]
    AlphaScalarBranchFunction,
    #[serde(rename = "alpha.scalar.leaf.function")]
    AlphaScalarLeafFunction,
    #[serde(rename = "alpha.vector.branch.function")]
    AlphaVectorBranchFunction,
    #[serde(rename = "alpha.vector.leaf.function")]
    AlphaVectorLeafFunction,
}

/// A prompt for a single invention step, applicable to one or more state types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "functions.inventions.prompt.StepPromptExpression")]
pub struct StepPromptExpression {
    pub r#type: Vec<StepPromptType>,
    pub value: WithExpression<String>,
}

impl StepPromptExpression {
    pub fn compile(
        self,
        params: &functions::expression::Params,
    ) -> Result<String, functions::expression::ExpressionError> {
        self.value.compile_one(params)
    }
}

/// Invention prompt configuration for all steps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "functions.inventions.Prompt")]
pub struct Prompt {
    pub essay: Vec<StepPromptExpression>,
    pub input_schema: Vec<StepPromptExpression>,
    pub essay_tasks: Vec<StepPromptExpression>,
    pub tasks: Vec<StepPromptExpression>,
    pub description: Vec<StepPromptExpression>,
}
