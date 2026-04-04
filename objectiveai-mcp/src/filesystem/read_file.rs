use super::state::{FileStateCache, FileStateEntry};
use super::util;

#[derive(Debug, serde::Serialize)]
pub struct TextFilePayload {
    #[serde(rename = "filePath")]
    pub file_path: String,
    pub content: String,
    #[serde(rename = "numLines")]
    pub num_lines: usize,
    #[serde(rename = "startLine")]
    pub start_line: usize,
    #[serde(rename = "totalLines")]
    pub total_lines: usize,
}

#[derive(Debug, serde::Serialize)]
pub struct ReadFileOutput {
    #[serde(rename = "type")]
    pub kind: String,
    pub file: TextFilePayload,
}

pub fn read_file(
    file_state: &FileStateCache,
    path: &str,
    offset: Option<usize>,
    limit: Option<usize>,
) -> Result<String, String> {
    // Check for blocked devices
    if util::is_blocked_device(path) {
        return Err(format!("Reading from {path} is not allowed"));
    }

    let absolute_path = util::normalize_path(path)
        .map_err(|e| format!("Failed to resolve path: {e}"))?;
    let absolute_path_str = absolute_path.to_string_lossy().to_string();

    let raw_content = std::fs::read_to_string(&absolute_path)
        .map_err(|e| format!("Failed to read file: {e}"))?;

    let content = util::normalize_line_endings(&raw_content);
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    // Convert offset: user provides 1-indexed, internally 0-indexed
    // offset 0 or None both mean start from beginning
    let start_index = match offset {
        Some(0) | None => 0,
        Some(n) => (n.saturating_sub(1)).min(total_lines),
    };

    let end_index = match limit {
        Some(l) => start_index.saturating_add(l).min(total_lines),
        None => total_lines,
    };

    let selected = lines[start_index..end_index].join("\n");
    let num_lines = end_index.saturating_sub(start_index);
    let start_line = start_index.saturating_add(1); // 1-indexed for output

    // Get file modification time
    let mtime_ms = util::get_file_mtime_ms(&absolute_path)
        .map_err(|e| format!("Failed to get file mtime: {e}"))?;

    // Update readFileState
    file_state.set(absolute_path_str.clone(), FileStateEntry {
        content: content.clone(),
        timestamp: mtime_ms,
        offset,
        limit,
    });

    let output = ReadFileOutput {
        kind: "text".into(),
        file: TextFilePayload {
            file_path: absolute_path_str,
            content: selected,
            num_lines,
            start_line,
            total_lines,
        },
    };

    serde_json::to_string_pretty(&output)
        .map_err(|e| format!("Failed to serialize output: {e}"))
}
