use super::state::{FileStateCache, FileStateEntry};
use super::util;

#[derive(Debug, serde::Serialize)]
pub struct EditFileOutput {
    #[serde(rename = "filePath")]
    pub file_path: String,
    #[serde(rename = "oldString")]
    pub old_string: String,
    #[serde(rename = "newString")]
    pub new_string: String,
    #[serde(rename = "originalFile")]
    pub original_file: String,
    #[serde(rename = "structuredPatch")]
    pub structured_patch: Vec<util::StructuredPatchHunk>,
    #[serde(rename = "replaceAll")]
    pub replace_all: bool,
}

pub fn edit_file(
    file_state: &FileStateCache,
    path: &str,
    old_string: &str,
    new_string: &str,
    replace_all: bool,
) -> Result<String, String> {
    // Error code 1: no-op edit
    if old_string == new_string {
        return Err("old_string and new_string must be different".into());
    }

    let absolute_path = util::normalize_path(path)
        .map_err(|e| format!("Failed to resolve path: {e}"))?;
    let absolute_path_str = absolute_path.to_string_lossy().to_string();

    // Must-read check (error code 6)
    let cached = file_state.get(&absolute_path_str);
    match &cached {
        None => {
            return Err("File has not been read yet. Read it first before writing to it.".into());
        }
        Some(entry) if entry.is_partial_view() => {
            return Err("File has not been read yet. Read it first before writing to it.".into());
        }
        Some(_) => {}
    }
    let cached = cached.unwrap();

    // Read current file content
    let original_file = std::fs::read_to_string(&absolute_path)
        .map_err(|e| format!("Failed to read file: {e}"))?;
    let original_file = util::normalize_line_endings(&original_file);

    // Staleness check (error code 7)
    let current_mtime = util::get_file_mtime_ms(&absolute_path)
        .map_err(|e| format!("Failed to get file mtime: {e}"))?;
    if current_mtime > cached.timestamp {
        // Windows content-comparison fallback for full reads
        let is_full_read = cached.offset.is_none() && cached.limit.is_none();
        if !(is_full_read && original_file == cached.content) {
            return Err(
                "File has been modified since read, either by the user or by a linter. Read it again before attempting to write it.".into()
            );
        }
    }

    // Find old_string in file with quote normalization fallback
    let actual_old_string = if original_file.contains(old_string) {
        old_string.to_owned()
    } else {
        // Try curly quote normalization
        match util::find_match_with_quote_normalization(&original_file, old_string) {
            Some(matched) => matched.to_owned(),
            None => {
                // Error code 8: match not found
                return Err("old_string not found in file".into());
            }
        }
    };

    // Count matches
    let matches = original_file.matches(&actual_old_string).count();

    // Error code 9: ambiguous match
    if matches > 1 && !replace_all {
        return Err(format!(
            "Found {matches} matches of the string to replace, but replace_all is false. \
             Either provide a larger string with more surrounding context to make it unique, \
             or use replace_all to change every instance."
        ));
    }

    // Apply the edit
    let updated = util::apply_edit(&original_file, &actual_old_string, new_string, replace_all);

    // Write the updated file
    std::fs::write(&absolute_path, &updated)
        .map_err(|e| format!("Failed to write file: {e}"))?;

    // Update readFileState
    let mtime_ms = util::get_file_mtime_ms(&absolute_path)
        .map_err(|e| format!("Failed to get file mtime: {e}"))?;
    file_state.set(absolute_path_str.clone(), FileStateEntry {
        content: updated.clone(),
        timestamp: mtime_ms,
        offset: None,
        limit: None,
        is_partial_view: false,
    });

    let patch = util::make_patch(&original_file, &updated);

    let output = EditFileOutput {
        file_path: absolute_path_str,
        old_string: actual_old_string,
        new_string: new_string.to_owned(),
        original_file,
        structured_patch: patch,
        replace_all,
    };

    serde_json::to_string_pretty(&output)
        .map_err(|e| format!("Failed to serialize output: {e}"))
}
