use thiserror::Error;

#[derive(Debug, Error)]
pub enum StdioParseError {
    /// Line is empty or whitespace-only.
    #[error("empty line")]
    Empty,
    /// Line doesn't start with `{` or is otherwise not a JSON object.
    #[error("malformed json envelope: expected '{{' at offset {0}")]
    NotAnObject(usize),
    /// A field name (string literal) couldn't be read at the given
    /// offset — missing quotes, immediate EOF, etc.
    #[error("malformed field name at offset {0}")]
    MalformedFieldName(usize),
    /// Reached end of line while still inside a string literal.
    #[error("unterminated string at offset {0}")]
    UnterminatedString(usize),
    /// Bytes after a field name are not `:` (with optional surrounding
    /// whitespace).
    #[error("expected ':' at offset {0}")]
    MissingColon(usize),
    /// First field name is not `type`.
    #[error("expected `type` as the first field")]
    MissingTypeField,
    /// First-field value (`type`) is not a JSON string.
    #[error("`type` field is not a string")]
    TypeNotString,
    /// Bytes between fields are not `,` (with optional surrounding
    /// whitespace).
    #[error("expected ',' between fields at offset {0}")]
    MissingComma(usize),
    /// Second field is `id` but its value is not a JSON string.
    #[error("`id` field is not a string")]
    IdNotString,
    /// Schema requires `id` as the second field but the line had none.
    #[error("missing `id` as second field")]
    MissingIdField,
    /// `serde_json::from_str` failed on a line whose id matched.
    #[error("deserialize: {0}")]
    Deserialize(#[from] serde_json::Error),
}
