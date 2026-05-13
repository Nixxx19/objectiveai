use crate::error;

/// An error from a single agent completion inside a [`VectorCompletionChunk`](super::VectorCompletionChunk).
///
/// Yielded by [`VectorCompletionChunk::inner_errors`](super::VectorCompletionChunk::inner_errors).
/// Identifies the failing completion by its `index` (matching
/// [`AgentCompletionChunk::index`](super::AgentCompletionChunk::index)) and
/// carries a borrow of the underlying [`ResponseError`](error::ResponseError).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InnerError<'a> {
    /// Index of the failing completion (matches `AgentCompletionChunk::index`).
    pub index: u64,
    /// The underlying error from the agent completion.
    pub error: &'a error::ResponseError,
}
