use super::state::{FileStateCache, FileStateEntry};
use super::util;

const MAX_READ_SIZE_BYTES: u64 = 10 * 1024 * 1024; // 10 MB

const BINARY_EXTENSIONS: &[&str] = &[
    // Images
    "png", "jpg", "jpeg", "gif", "bmp", "ico", "tiff", "tif", "webp", "svg", "avif", "heic",
    "heif", // Video
    "mp4", "avi", "mov", "wmv", "flv", "mkv", "webm", "m4v", "mpg", "mpeg",
    // Audio
    "mp3", "wav", "flac", "aac", "ogg", "wma", "m4a", "opus",
    // Archives
    "zip", "tar", "gz", "bz2", "xz", "7z", "rar", "zst", "lz4",
    // Executables/Libraries
    "exe", "dll", "so", "dylib", "o", "a", "lib", "obj", "class", "pyc", "pyo",
    // Documents (binary)
    "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods", "odp",
    // Databases
    "db", "sqlite", "sqlite3", "mdb",
    // Fonts
    "ttf", "otf", "woff", "woff2", "eot",
    // Other binary
    "bin", "dat", "iso", "img", "dmg", "wasm", "deb", "rpm",
];

fn has_binary_extension(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| BINARY_EXTENSIONS.iter().any(|&b| b.eq_ignore_ascii_case(e)))
        .unwrap_or(false)
}

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
        return Err(format!("Cannot read '{path}': this device file would block or produce infinite output."));
    }

    let absolute_path = util::normalize_path(path)
        .map_err(|e| format!("Failed to resolve path: {e}"))?;
    let absolute_path_str = absolute_path.to_string_lossy().to_string();

    if has_binary_extension(&absolute_path) {
        return Err(format!(
            "Cannot read binary file '{}'. Binary files are not supported.",
            path
        ));
    }

    // Check file size before reading
    let metadata = std::fs::metadata(&absolute_path)
        .map_err(|e| format!("Failed to read file metadata: {e}"))?;
    let file_size = metadata.len();
    if file_size > MAX_READ_SIZE_BYTES {
        return Err(format!(
            "File is too large to read ({file_size} bytes, max 10MB). \
             Consider reading specific line ranges with offset and limit."
        ));
    }

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
        is_partial_view: false,
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
