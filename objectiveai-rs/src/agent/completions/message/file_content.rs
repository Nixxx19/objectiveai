/// Extractable file content from a media type.
///
/// `content` is the raw payload (e.g. base64-encoded data).
/// `extension` is the file extension to use (e.g. `"png"`, `"wav"`).
pub struct FileContent<'s> {
    pub content: &'s str,
    pub extension: &'s str,
}

/// Parses a data URL, returning `(full_mime, base64_payload)`.
///
/// Expects the format `data:{type}/{subtype};base64,{payload}`.
/// Returns `None` if the URL is not a valid base64 data URL.
pub(crate) fn parse_data_url(url: &str) -> Option<(&str, &str)> {
    let rest = url.strip_prefix("data:")?;
    let (mime, payload) = rest.split_once(";base64,")?;
    Some((mime, payload))
}

/// Maps a full MIME type to a file extension using `mime2ext`.
///
/// Falls back to `"bin"` if the MIME type is not recognized.
pub(crate) fn mime_to_ext(mime: &str) -> &str {
    mime2ext::mime2ext(mime).unwrap_or("bin")
}
