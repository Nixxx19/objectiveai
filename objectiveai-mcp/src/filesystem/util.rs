use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

/// Normalize a path to an absolute canonical path.
pub fn normalize_path(path: &str) -> io::Result<PathBuf> {
    let candidate = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?.join(path)
    };
    candidate.canonicalize()
}

/// Normalize a path, allowing the file to not exist yet (for writes).
pub fn normalize_path_allow_missing(path: &str) -> io::Result<PathBuf> {
    let candidate = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?.join(path)
    };

    if let Ok(canonical) = candidate.canonicalize() {
        return Ok(canonical);
    }

    if let Some(parent) = candidate.parent() {
        let canonical_parent = parent
            .canonicalize()
            .unwrap_or_else(|_| parent.to_path_buf());
        if let Some(name) = candidate.file_name() {
            return Ok(canonical_parent.join(name));
        }
    }

    Ok(candidate)
}

/// Get file modification time in milliseconds since epoch.
pub fn get_file_mtime_ms(path: &Path) -> io::Result<u64> {
    let metadata = fs::metadata(path)?;
    let modified = metadata.modified()?;
    Ok(modified
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64)
}

/// Normalize line endings: CRLF → LF.
pub fn normalize_line_endings(content: &str) -> String {
    content.replace("\r\n", "\n")
}

/// Blocked device paths that should not be read.
const BLOCKED_DEVICES: &[&str] = &[
    "/dev/zero",
    "/dev/random",
    "/dev/urandom",
    "/dev/full",
    "/dev/stdin",
    "/dev/tty",
    "/dev/console",
    "/dev/stdout",
    "/dev/stderr",
    "/dev/fd/0",
    "/dev/fd/1",
    "/dev/fd/2",
];

/// Check if a path is a blocked device.
pub fn is_blocked_device(path: &str) -> bool {
    BLOCKED_DEVICES.iter().any(|d| path == *d)
        || path.starts_with("/proc/") && path.contains("/fd/")
}

/// Simple structured patch hunk for diff output.
#[derive(Debug, Clone, serde::Serialize)]
pub struct StructuredPatchHunk {
    #[serde(rename = "oldStart")]
    pub old_start: usize,
    #[serde(rename = "oldLines")]
    pub old_lines: usize,
    #[serde(rename = "newStart")]
    pub new_start: usize,
    #[serde(rename = "newLines")]
    pub new_lines: usize,
    pub lines: Vec<String>,
}

/// Generate a structured patch between two strings using line-level diff.
pub fn make_patch(original: &str, updated: &str) -> Vec<StructuredPatchHunk> {
    let old_lines: Vec<&str> = original.lines().collect();
    let new_lines: Vec<&str> = updated.lines().collect();

    if old_lines == new_lines {
        return Vec::new();
    }

    // Find first differing line
    let common_prefix = old_lines
        .iter()
        .zip(new_lines.iter())
        .take_while(|(a, b)| a == b)
        .count();

    // Find last differing line (from end)
    let common_suffix = old_lines
        .iter()
        .rev()
        .zip(new_lines.iter().rev())
        .take_while(|(a, b)| a == b)
        .count()
        .min(old_lines.len() - common_prefix)
        .min(new_lines.len() - common_prefix);

    let old_changed_end = old_lines.len() - common_suffix;
    let new_changed_end = new_lines.len() - common_suffix;

    // Build hunk with up to 3 lines of context
    let ctx = 3;
    let hunk_old_start = common_prefix.saturating_sub(ctx);
    let hunk_old_end = (old_changed_end + ctx).min(old_lines.len());
    let hunk_new_start = common_prefix.saturating_sub(ctx);
    let hunk_new_end = (new_changed_end + ctx).min(new_lines.len());

    let mut lines = Vec::new();

    // Context before
    for i in hunk_old_start..common_prefix {
        lines.push(format!(" {}", old_lines[i]));
    }
    // Removed lines
    for i in common_prefix..old_changed_end {
        lines.push(format!("-{}", old_lines[i]));
    }
    // Added lines
    for i in common_prefix..new_changed_end {
        lines.push(format!("+{}", new_lines[i]));
    }
    // Context after
    for i in old_changed_end..hunk_old_end {
        lines.push(format!(" {}", old_lines[i]));
    }

    vec![StructuredPatchHunk {
        old_start: hunk_old_start + 1, // 1-indexed
        old_lines: hunk_old_end - hunk_old_start,
        new_start: hunk_new_start + 1, // 1-indexed
        new_lines: hunk_new_end - hunk_new_start,
        lines,
    }]
}

/// Curly quote characters for normalization.
const LEFT_SINGLE_CURLY: char = '\u{2018}';
const RIGHT_SINGLE_CURLY: char = '\u{2019}';
const LEFT_DOUBLE_CURLY: char = '\u{201C}';
const RIGHT_DOUBLE_CURLY: char = '\u{201D}';

/// Try to find old_string in content, falling back to curly quote normalization.
/// Returns the actual matched string from the file content.
pub fn find_match_with_quote_normalization<'a>(content: &'a str, search: &str) -> Option<&'a str> {
    // Try exact match first
    if let Some(idx) = content.find(search) {
        return Some(&content[idx..idx + search.len()]);
    }

    // Normalize curly quotes in the search string to straight quotes
    let normalized_search = search
        .replace(LEFT_SINGLE_CURLY, "'")
        .replace(RIGHT_SINGLE_CURLY, "'")
        .replace(LEFT_DOUBLE_CURLY, "\"")
        .replace(RIGHT_DOUBLE_CURLY, "\"");

    if normalized_search == search {
        return None; // No curly quotes to normalize
    }

    // Try finding the normalized search directly in the original content.
    // This handles the case where the search string had curly quotes but
    // the file content has straight quotes.
    if let Some(idx) = content.find(&normalized_search) {
        return Some(&content[idx..idx + normalized_search.len()]);
    }

    // Try the reverse: normalize the content and search with the normalized
    // search string. Walk the original content char-by-char to find the
    // matching span, avoiding byte-length mismatches between original and
    // normalized content.
    let normalized_content = content
        .replace(LEFT_SINGLE_CURLY, "'")
        .replace(RIGHT_SINGLE_CURLY, "'")
        .replace(LEFT_DOUBLE_CURLY, "\"")
        .replace(RIGHT_DOUBLE_CURLY, "\"");

    if let Some(norm_byte_idx) = normalized_content.find(&normalized_search) {
        // Count the number of characters before the match in normalized content
        let char_offset = normalized_content[..norm_byte_idx].chars().count();
        let match_char_len = normalized_search.chars().count();

        // Map character offset back to byte offset in the original content
        let orig_byte_start = content
            .char_indices()
            .nth(char_offset)
            .map(|(i, _)| i)?;
        let orig_byte_end = content
            .char_indices()
            .nth(char_offset + match_char_len)
            .map(|(i, _)| i)
            .unwrap_or(content.len());

        Some(&content[orig_byte_start..orig_byte_end])
    } else {
        None
    }
}

/// Apply an edit to file content.
pub fn apply_edit(
    original: &str,
    old_string: &str,
    new_string: &str,
    replace_all: bool,
) -> String {
    if replace_all {
        original.replace(old_string, new_string)
    } else if new_string.is_empty() {
        // Empty new_string: strip trailing newline if present
        let with_newline = format!("{old_string}\n");
        if original.contains(&with_newline) {
            original.replacen(&with_newline, "", 1)
        } else {
            original.replacen(old_string, "", 1)
        }
    } else {
        original.replacen(old_string, new_string, 1)
    }
}
