use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
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
