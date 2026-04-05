use super::state::{FileStateCache, FileStateEntry};
use super::util;
use std::path::Path;

const MAX_READ_SIZE_BYTES: u64 = 10 * 1024 * 1024; // 10 MB
const MAX_IMAGE_FILE_SIZE: u64 = 20 * 1024 * 1024; // 20 MB

const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp"];

// Binary extensions -- does NOT include image extensions (handled separately)
const BINARY_EXTENSIONS: &[&str] = &[
    // Images that we DON'T support reading (non-web formats)
    "bmp", "ico", "tiff", "tif", "avif", "heic", "heif",
    // Video
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

fn has_extension_in(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| extensions.iter().any(|&b| b.eq_ignore_ascii_case(e)))
        .unwrap_or(false)
}

/// Detect image format from magic bytes.
fn detect_image_format(bytes: &[u8]) -> &'static str {
    if bytes.len() >= 4 && bytes[..4] == [0x89, 0x50, 0x4E, 0x47] {
        "image/png"
    } else if bytes.len() >= 3 && bytes[..3] == [0xFF, 0xD8, 0xFF] {
        "image/jpeg"
    } else if bytes.len() >= 4 && &bytes[..4] == b"GIF8" {
        "image/gif"
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        "image/webp"
    } else {
        "image/png" // default fallback
    }
}

/// Output variants from read_file.
pub enum ReadOutput {
    /// Text file content (JSON serialized)
    Text(String),
    /// Image file (base64 data + media type)
    Image { base64: String, media_type: String },
    /// Notebook cells (text + embedded images)
    Notebook(Vec<super::notebook::NotebookBlock>),
    /// File unchanged since last read (dedup stub)
    FileUnchanged(String),
    /// PDF not supported (error message)
    Pdf(String),
}

#[derive(Debug, serde::Serialize)]
struct TextFilePayload {
    #[serde(rename = "filePath")]
    file_path: String,
    content: String,
    #[serde(rename = "numLines")]
    num_lines: usize,
    #[serde(rename = "startLine")]
    start_line: usize,
    #[serde(rename = "totalLines")]
    total_lines: usize,
}

#[derive(Debug, serde::Serialize)]
struct ReadFileJsonOutput {
    #[serde(rename = "type")]
    kind: String,
    file: TextFilePayload,
}

pub fn read_file(
    file_state: &FileStateCache,
    path: &str,
    offset: Option<usize>,
    limit: Option<usize>,
) -> Result<ReadOutput, String> {
    // UNC path security check
    if util::is_unc_path(path) {
        return Err("Cannot read files on UNC paths.".into());
    }

    // Check for blocked devices
    if util::is_blocked_device(path) {
        return Err(format!("Cannot read '{path}': this device file would block or produce infinite output."));
    }

    let absolute_path = util::normalize_path_allow_missing(path)
        .map_err(|e| format!("Failed to resolve path: {e}"))?;
    let absolute_path_str = absolute_path.to_string_lossy().to_string();

    // File-not-found with suggestions
    if !absolute_path.exists() {
        let mut msg = format!(
            "File does not exist. Note: your current working directory is {}.",
            std::env::current_dir()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default()
        );
        if let Some(similar) = util::find_similar_file(&absolute_path) {
            msg.push_str(&format!("\nDid you mean: {similar}"));
        }
        if let Some(suggested) = util::suggest_path_under_cwd(path) {
            msg.push_str(&format!("\nSuggested path: {suggested}"));
        }
        return Err(msg);
    }

    // file_unchanged dedup: if same path, same offset/limit, same mtime -> return stub
    if let Some(cached) = file_state.get(&absolute_path_str) {
        // Only dedup entries that came from a prior Read (offset is Some), not from Edit/Write
        if cached.offset.is_some() && cached.offset == offset && cached.limit == limit {
            if let Ok(current_mtime) = util::get_file_mtime_ms(&absolute_path) {
                if current_mtime == cached.timestamp {
                    return Ok(ReadOutput::FileUnchanged(
                        "File unchanged since last read. The content from the earlier Read tool_result in this conversation is still current \u{2014} refer to that instead of re-reading.".into()
                    ));
                }
            }
        }
    }

    let ext = absolute_path.extension().and_then(|e| e.to_str()).unwrap_or("");

    // Image files -- return as Content::image
    if has_extension_in(&absolute_path, IMAGE_EXTENSIONS) {
        let metadata = std::fs::metadata(&absolute_path)
            .map_err(|e| format!("Failed to read file metadata: {e}"))?;
        if metadata.len() > MAX_IMAGE_FILE_SIZE {
            return Err(format!(
                "Image file is too large ({} bytes, max 20MB).",
                metadata.len()
            ));
        }
        let bytes = std::fs::read(&absolute_path)
            .map_err(|e| format!("Failed to read image file: {e}"))?;
        let media_type = detect_image_format(&bytes).to_string();
        let base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);

        // Update file state (store placeholder content, not the base64)
        let mtime_ms = util::get_file_mtime_ms(&absolute_path)
            .map_err(|e| format!("Failed to get file mtime: {e}"))?;
        file_state.set(absolute_path_str, FileStateEntry {
            content: format!("[image: {} bytes]", bytes.len()),
            timestamp: mtime_ms,
            offset,
            limit,
            is_partial_view: false,
        });

        return Ok(ReadOutput::Image { base64, media_type });
    }

    // Notebook files (.ipynb)
    if ext.eq_ignore_ascii_case("ipynb") {
        let metadata = std::fs::metadata(&absolute_path)
            .map_err(|e| format!("Failed to read file metadata: {e}"))?;
        if metadata.len() > MAX_READ_SIZE_BYTES {
            return Err(format!(
                "Notebook file is too large ({} bytes, max 10MB).",
                metadata.len()
            ));
        }
        let blocks = super::notebook::read_notebook(&absolute_path)?;

        // Update file state
        let raw = std::fs::read_to_string(&absolute_path).unwrap_or_default();
        let mtime_ms = util::get_file_mtime_ms(&absolute_path)
            .map_err(|e| format!("Failed to get file mtime: {e}"))?;
        file_state.set(absolute_path_str, FileStateEntry {
            content: util::normalize_line_endings(&raw),
            timestamp: mtime_ms,
            offset,
            limit,
            is_partial_view: false,
        });

        return Ok(ReadOutput::Notebook(blocks));
    }

    // PDF files -- stub
    if ext.eq_ignore_ascii_case("pdf") {
        return Ok(ReadOutput::Pdf(
            "PDF reading is not yet supported. Use Bash with pdftotext or similar utilities.".into()
        ));
    }

    // Binary file rejection
    if has_extension_in(&absolute_path, BINARY_EXTENSIONS) {
        return Err(format!(
            "This tool cannot read binary files. The file appears to be a binary .{ext} file. \
             Please use appropriate tools for binary file analysis."
        ));
    }

    // Check file size before reading text
    let metadata = std::fs::metadata(&absolute_path)
        .map_err(|e| format!("Failed to read file metadata: {e}"))?;
    if metadata.len() > MAX_READ_SIZE_BYTES {
        return Err(format!(
            "File is too large to read ({} bytes, max 10MB). \
             Consider reading specific line ranges with offset and limit.",
            metadata.len()
        ));
    }

    let raw_content = std::fs::read_to_string(&absolute_path)
        .map_err(|e| format!("Failed to read file: {e}"))?;

    let content = util::normalize_line_endings(&raw_content);
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

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
    let start_line = start_index.saturating_add(1);

    let mtime_ms = util::get_file_mtime_ms(&absolute_path)
        .map_err(|e| format!("Failed to get file mtime: {e}"))?;

    file_state.set(absolute_path_str.clone(), FileStateEntry {
        content: content.clone(),
        timestamp: mtime_ms,
        offset,
        limit,
        is_partial_view: false,
    });

    let output = ReadFileJsonOutput {
        kind: "text".into(),
        file: TextFilePayload {
            file_path: absolute_path_str,
            content: selected,
            num_lines,
            start_line,
            total_lines,
        },
    };

    let json = serde_json::to_string_pretty(&output)
        .map_err(|e| format!("Failed to serialize output: {e}"))?;
    Ok(ReadOutput::Text(json))
}
