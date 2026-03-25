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
    pub fn remotes(&self) -> impl Iterator<Item = &crate::RemotePath> {
        enum Remotes<A, B> { A(A), B(B) }
        impl<'a, A: Iterator<Item = &'a crate::RemotePath>, B: Iterator<Item = &'a crate::RemotePath>> Iterator for Remotes<A, B> {
            type Item = &'a crate::RemotePath;
            fn next(&mut self) -> Option<Self::Item> {
                match self { Remotes::A(a) => a.next(), Remotes::B(b) => b.next() }
            }
        }
        match self {
            FullRemoteFunction::Alpha(f) => Remotes::A(f.remotes()),
            FullRemoteFunction::Standard(f) => Remotes::B(f.remotes()),
        }
    }

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
    pub fn remotes(&self) -> impl Iterator<Item = &crate::RemotePath> {
        enum Remotes<A, B> { A(A), B(B) }
        impl<'a, A: Iterator<Item = &'a crate::RemotePath>, B: Iterator<Item = &'a crate::RemotePath>> Iterator for Remotes<A, B> {
            type Item = &'a crate::RemotePath;
            fn next(&mut self) -> Option<Self::Item> {
                match self { Remotes::A(a) => a.next(), Remotes::B(b) => b.next() }
            }
        }
        match self {
            FullInlineFunction::Alpha(f) => Remotes::A(f.remotes()),
            FullInlineFunction::Standard(f) => Remotes::B(f.remotes()),
        }
    }

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
    pub fn remotes(&self) -> impl Iterator<Item = &crate::RemotePath> {
        enum Remotes<A, B> { A(A), B(B) }
        impl<'a, A: Iterator<Item = &'a crate::RemotePath>, B: Iterator<Item = &'a crate::RemotePath>> Iterator for Remotes<A, B> {
            type Item = &'a crate::RemotePath;
            fn next(&mut self) -> Option<Self::Item> {
                match self { Remotes::A(a) => a.next(), Remotes::B(b) => b.next() }
            }
        }
        match self {
            AlphaRemoteFunction::Scalar(f) => Remotes::A(f.remotes()),
            AlphaRemoteFunction::Vector(f) => Remotes::B(f.remotes()),
        }
    }

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
    pub fn remotes(&self) -> impl Iterator<Item = &crate::RemotePath> {
        enum Remotes<A, B> { A(A), B(B) }
        impl<'a, A: Iterator<Item = &'a crate::RemotePath>, B: Iterator<Item = &'a crate::RemotePath>> Iterator for Remotes<A, B> {
            type Item = &'a crate::RemotePath;
            fn next(&mut self) -> Option<Self::Item> {
                match self { Remotes::A(a) => a.next(), Remotes::B(b) => b.next() }
            }
        }
        match self {
            AlphaInlineFunction::Scalar(f) => Remotes::A(f.remotes()),
            AlphaInlineFunction::Vector(f) => Remotes::B(f.remotes()),
        }
    }

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

impl FullFunction {
    pub fn remotes(&self) -> impl Iterator<Item = &crate::RemotePath> {
        enum Remotes<A, B> { A(A), B(B) }
        impl<'a, A: Iterator<Item = &'a crate::RemotePath>, B: Iterator<Item = &'a crate::RemotePath>> Iterator for Remotes<A, B> {
            type Item = &'a crate::RemotePath;
            fn next(&mut self) -> Option<Self::Item> {
                match self { Remotes::A(a) => a.next(), Remotes::B(b) => b.next() }
            }
        }
        match self {
            FullFunction::Remote(f) => Remotes::A(f.remotes()),
            FullFunction::Inline(f) => Remotes::B(f.remotes()),
        }
    }
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
