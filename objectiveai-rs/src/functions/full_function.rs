use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, arbitrary::Arbitrary)]
#[serde(untagged)]
#[schemars(rename = "functions.FullRemoteFunction")]
pub enum FullRemoteFunction {
    Alpha(AlphaRemoteFunction),
    Standard(super::RemoteFunction),
}

impl FullRemoteFunction {
    pub fn transpile(self) -> super::RemoteFunction {
        match self {
            FullRemoteFunction::Alpha(function) => function.transpile(),
            FullRemoteFunction::Standard(function) => function,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(rename = "functions.FullInlineFunction")]
pub enum FullInlineFunction {
    Alpha(AlphaInlineFunction),
    Standard(super::InlineFunction),
}

impl FullInlineFunction {
    pub fn transpile(self) -> super::InlineFunction {
        match self {
            FullInlineFunction::Alpha(function) => function.transpile(),
            FullInlineFunction::Standard(function) => function,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, arbitrary::Arbitrary)]
#[serde(untagged)]
#[schemars(rename = "functions.AlphaRemoteFunction")]
pub enum AlphaRemoteFunction {
    Scalar(super::alpha_scalar::RemoteFunction),
    Vector(super::alpha_vector::RemoteFunction),
}

impl AlphaRemoteFunction {
    pub fn transpile(self) -> super::RemoteFunction {
        match self {
            AlphaRemoteFunction::Scalar(function) => function.transpile(),
            AlphaRemoteFunction::Vector(function) => function.transpile(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(rename = "functions.AlphaInlineFunction")]
pub enum AlphaInlineFunction {
    Scalar(super::alpha_scalar::InlineFunction),
    Vector(super::alpha_vector::InlineFunction),
}

impl AlphaInlineFunction {
    pub fn transpile(self) -> super::InlineFunction {
        match self {
            AlphaInlineFunction::Scalar(function) => function.transpile(),
            AlphaInlineFunction::Vector(function) => function.transpile(),
        }
    }
}

/// A full function, either remote or inline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(rename = "functions.FullFunction")]
pub enum FullFunction {
    Remote(FullRemoteFunction),
    Inline(FullInlineFunction),
}

/// A function specification that is either a full inline function definition
/// or a remote path reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(rename = "functions.FullInlineFunctionOrRemoteCommitOptional")]
pub enum FullInlineFunctionOrRemoteCommitOptional {
    Inline(FullInlineFunction),
    Remote(crate::RemotePathCommitOptional),
}
