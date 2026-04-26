use std::fs;
use std::path::Path;
use std::time::Instant;

use crate::util;

#[derive(Debug, serde::Serialize)]
pub struct GlobSearchOutput {
    #[serde(rename = "durationMs")]
    pub duration_ms: u128,
    #[serde(rename = "numFiles")]
    pub num_files: usize,
    pub filenames: Vec<String>,
    pub truncated: bool,
}

pub fn glob_search(pattern: &str, path: Option<&str>) -> Result<String, String> {
    let started = Instant::now();

    let base_dir = match path {
        Some(p) => util::normalize_path(p).map_err(|e| format!("Invalid path: {e}"))?,
        None => std::env::current_dir().map_err(|e| format!("Failed to get CWD: {e}"))?,
    };

    let search_pattern = if Path::new(pattern).is_absolute() {
        pattern.to_owned()
    } else {
        base_dir.join(pattern).to_string_lossy().into_owned()
    };

    let entries = glob::glob(&search_pattern)
        .map_err(|e| format!("Invalid glob pattern: {e}"))?;

    let mut matches = Vec::new();
    for entry in entries.flatten() {
        if entry.is_file() {
            matches.push(entry);
        }
    }

    // Sort by modification time, oldest first (ascending mtime)
    matches.sort_by(|a, b| {
        let a_mtime = fs::metadata(a).and_then(|m| m.modified()).ok();
        let b_mtime = fs::metadata(b).and_then(|m| m.modified()).ok();
        a_mtime.cmp(&b_mtime)
    });

    let truncated = matches.len() > 100;
    let filenames: Vec<String> = matches
        .into_iter()
        .take(100)
        .map(|p| p.to_string_lossy().into_owned())
        .collect();

    let output = GlobSearchOutput {
        duration_ms: started.elapsed().as_millis(),
        num_files: filenames.len(),
        filenames,
        truncated,
    };

    serde_json::to_string_pretty(&output)
        .map_err(|e| format!("Failed to serialize output: {e}"))
}
