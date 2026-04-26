use super::id_prefix::IdPrefix;
use super::StdioParseError;

/// Skip the JSON string literal that starts at `bytes[i]` (which must
/// be the opening `"`). Returns the byte index *immediately after* the
/// closing `"`, or an error if the literal is unterminated. JSON
/// escape pairs (`\"`, `\\`, etc.) are stepped over without being
/// interpreted.
fn skip_string_literal(bytes: &[u8], mut i: usize) -> Result<usize, StdioParseError> {
    let len = bytes.len();
    debug_assert!(i < len && bytes[i] == b'"');
    let start = i;
    i += 1;
    while i < len && bytes[i] != b'"' {
        if bytes[i] == b'\\' && i + 1 < len {
            i += 2;
        } else {
            i += 1;
        }
    }
    if i >= len {
        return Err(StdioParseError::UnterminatedString(start + 1));
    }
    Ok(i + 1)
}

/// Skip ASCII whitespace; return the new index.
fn skip_ws(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

/// Scan `line` looking for `"type":"<v>","id":"<id>"` at the very
/// front of the JSON object (allowing only whitespace before/after
/// the `{` and around separators). The `type` value is parsed but
/// not interpreted — `serde_json` handles the discriminator on the
/// full deserialize. Does **not** unescape JSON inside the id value;
/// caller-supplied ids are constrained to plain ASCII.
pub(super) fn scan_id_prefix(line: &str) -> Result<IdPrefix<'_>, StdioParseError> {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = skip_ws(bytes, 0);
    if i == len {
        return Err(StdioParseError::Empty);
    }

    // Opening `{`.
    if bytes[i] != b'{' {
        return Err(StdioParseError::NotAnObject(i));
    }
    i += 1;
    i = skip_ws(bytes, i);

    // Empty object — no `type`, no `id`. Treat the same as a fatal
    // (caller's full deserialize will reject it cleanly if invalid).
    if i < len && bytes[i] == b'}' {
        return Err(StdioParseError::MissingTypeField);
    }

    // First field name — must be `"type"`.
    if i >= len || bytes[i] != b'"' {
        return Err(StdioParseError::MalformedFieldName(i));
    }
    let key_start = i + 1;
    i = skip_string_literal(bytes, i)?;
    let first_key = &line[key_start..i - 1];
    if first_key != "type" {
        return Err(StdioParseError::MissingTypeField);
    }

    // `:` after `type`.
    i = skip_ws(bytes, i);
    if i >= len || bytes[i] != b':' {
        return Err(StdioParseError::MissingColon(i));
    }
    i += 1;
    i = skip_ws(bytes, i);

    // `type`'s value must be a string. We don't care what it is — we
    // just need to step past it to reach the next field.
    if i >= len || bytes[i] != b'"' {
        return Err(StdioParseError::TypeNotString);
    }
    i = skip_string_literal(bytes, i)?;
    i = skip_ws(bytes, i);

    // After `"type":"<v>"` we expect either `,` (more fields) or `}`
    // (the object ends here — no `id`, treat as a no-id line).
    if i < len && bytes[i] == b'}' {
        return Ok(IdPrefix::NoId);
    }
    if i >= len || bytes[i] != b',' {
        return Err(StdioParseError::MissingComma(i));
    }
    i += 1;
    i = skip_ws(bytes, i);

    // Second field name. If it's `"id"`, read the id value.
    // Otherwise this is a no-id line (e.g. `fatal` whose second
    // field is `message`).
    if i >= len || bytes[i] != b'"' {
        return Err(StdioParseError::MalformedFieldName(i));
    }
    let key_start = i + 1;
    i = skip_string_literal(bytes, i)?;
    let second_key = &line[key_start..i - 1];
    if second_key != "id" {
        return Ok(IdPrefix::NoId);
    }

    // `:` after `id`.
    i = skip_ws(bytes, i);
    if i >= len || bytes[i] != b':' {
        return Err(StdioParseError::MissingColon(i));
    }
    i += 1;
    i = skip_ws(bytes, i);

    // `id` value must be a string.
    if i >= len || bytes[i] != b'"' {
        return Err(StdioParseError::IdNotString);
    }
    let val_start = i + 1;
    let after = skip_string_literal(bytes, i)?;
    Ok(IdPrefix::Id(&line[val_start..after - 1]))
}

/// Quickly recover the request `id` from a single output line without
/// running `serde_json` on the whole payload.
///
/// Returns:
///
/// - `Ok(Some(id))` — line is well-formed and its second field is
///   `id` (the per-request convention). The returned `&str` borrows
///   from `line`.
/// - `Ok(None)` — line is well-formed but has no `id` second field
///   (the untagged `fatal` carve-out).
/// - `Err(_)` — line is malformed.
///
/// The dispatcher uses this to route lines to the right per-request
/// channel before paying for a full deserialize.
pub fn extract_id(line: &str) -> Result<Option<&str>, StdioParseError> {
    match scan_id_prefix(line)? {
        IdPrefix::Id(id) => Ok(Some(id)),
        IdPrefix::NoId => Ok(None),
    }
}
