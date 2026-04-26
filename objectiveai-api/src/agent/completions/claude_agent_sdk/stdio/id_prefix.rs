/// Outcome of the prefix scan performed by `super::scanner::scan_id_prefix`.
///
/// Internal to the `stdio` module — `pub(super)` so sibling files
/// (`stdio_output`, `stdio_error`, `scanner`) can match on it, but
/// not re-exported at the crate level.
pub(super) enum IdPrefix<'a> {
    /// Second field is `id` with the given string value (raw bytes
    /// between the surrounding quotes — escapes intentionally not
    /// resolved; ids are caller-supplied and expected to be plain
    /// ASCII without escape sequences).
    Id(&'a str),
    /// First field is `type` and the JSON is well-formed past it, but
    /// the second field's name is not `id` (or the object has only one
    /// field). Used to recognize untagged `fatal` lines on stderr.
    NoId,
}
