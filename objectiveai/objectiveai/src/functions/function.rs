use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Function {
    Remote(RemoteFunction),
    Inline(InlineFunction),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RemoteFunction {
    #[serde(rename = "scalar.function")]
    Scalar {
        description: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        changelog: Option<String>,
        input_schema: super::expression::InputSchema,
        #[serde(skip_serializing_if = "Option::is_none")]
        input_maps: Option<super::expression::InputMaps>, // receives input
        tasks: Vec<super::TaskExpression>, // receives input + maps
        output: super::expression::Expression, // receives input + tasks
    },
    #[serde(rename = "vector.function")]
    Vector {
        description: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        changelog: Option<String>,
        input_schema: super::expression::InputSchema,
        #[serde(skip_serializing_if = "Option::is_none")]
        input_maps: Option<super::expression::InputMaps>, // receives input
        tasks: Vec<super::TaskExpression>, // receives input + maps
        output: super::expression::Expression, // receives input + tasks
        output_length: super::expression::WithExpression<u64>, // receives input
    },
}

impl RemoteFunction {
    pub fn description(&self) -> &str {
        match self {
            RemoteFunction::Scalar { description, .. } => description,
            RemoteFunction::Vector { description, .. } => description,
        }
    }

    pub fn changelog(&self) -> Option<&str> {
        match self {
            RemoteFunction::Scalar { changelog, .. } => changelog.as_deref(),
            RemoteFunction::Vector { changelog, .. } => changelog.as_deref(),
        }
    }

    pub fn input_schema(&self) -> &super::expression::InputSchema {
        match self {
            RemoteFunction::Scalar { input_schema, .. } => input_schema,
            RemoteFunction::Vector { input_schema, .. } => input_schema,
        }
    }

    pub fn input_maps(&self) -> Option<&super::expression::InputMaps> {
        match self {
            RemoteFunction::Scalar { input_maps, .. } => input_maps.as_ref(),
            RemoteFunction::Vector { input_maps, .. } => input_maps.as_ref(),
        }
    }

    pub fn tasks(&self) -> &[super::TaskExpression] {
        match self {
            RemoteFunction::Scalar { tasks, .. } => tasks,
            RemoteFunction::Vector { tasks, .. } => tasks,
        }
    }

    pub fn output(&self) -> &super::expression::Expression {
        match self {
            RemoteFunction::Scalar { output, .. } => output,
            RemoteFunction::Vector { output, .. } => output,
        }
    }

    pub fn output_length(
        &self,
    ) -> Option<&super::expression::WithExpression<u64>> {
        match self {
            RemoteFunction::Scalar { .. } => None,
            RemoteFunction::Vector { output_length, .. } => Some(output_length),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InlineFunction {
    #[serde(rename = "scalar.function")]
    Scalar {
        #[serde(skip_serializing_if = "Option::is_none")]
        input_maps: Option<super::expression::InputMaps>,
        tasks: Vec<super::TaskExpression>,
        output: super::expression::Expression,
    },
    #[serde(rename = "vector.function")]
    Vector {
        #[serde(skip_serializing_if = "Option::is_none")]
        input_maps: Option<super::expression::InputMaps>,
        tasks: Vec<super::TaskExpression>,
        output: super::expression::Expression,
    },
}

impl InlineFunction {
    pub fn input_maps(&self) -> Option<&super::expression::InputMaps> {
        match self {
            InlineFunction::Scalar { input_maps, .. } => input_maps.as_ref(),
            InlineFunction::Vector { input_maps, .. } => input_maps.as_ref(),
        }
    }

    pub fn tasks(&self) -> &[super::TaskExpression] {
        match self {
            InlineFunction::Scalar { tasks, .. } => tasks,
            InlineFunction::Vector { tasks, .. } => tasks,
        }
    }

    pub fn output(&self) -> &super::expression::Expression {
        match self {
            InlineFunction::Scalar { output, .. } => output,
            InlineFunction::Vector { output, .. } => output,
        }
    }
}
