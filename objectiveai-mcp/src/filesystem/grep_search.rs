use std::cmp::Reverse;
use std::fs;
use std::path::{Path, PathBuf};

use glob::Pattern;
use regex::RegexBuilder;
use walkdir::WalkDir;

use super::util;

#[derive(Debug, serde::Serialize)]
pub struct GrepSearchOutput {
    pub mode: Option<String>,
    #[serde(rename = "numFiles")]
    pub num_files: usize,
    pub filenames: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(rename = "numLines", skip_serializing_if = "Option::is_none")]
    pub num_lines: Option<usize>,
    #[serde(rename = "numMatches", skip_serializing_if = "Option::is_none")]
    pub num_matches: Option<usize>,
    #[serde(rename = "appliedLimit", skip_serializing_if = "Option::is_none")]
    pub applied_limit: Option<usize>,
    #[serde(rename = "appliedOffset", skip_serializing_if = "Option::is_none")]
    pub applied_offset: Option<usize>,
}

pub struct GrepSearchInput {
    pub pattern: String,
    pub path: Option<String>,
    pub glob: Option<String>,
    pub output_mode: Option<String>,
    pub before: Option<usize>,
    pub after: Option<usize>,
    pub context_short: Option<usize>,
    pub context: Option<usize>,
    pub line_numbers: Option<bool>,
    pub case_insensitive: Option<bool>,
    pub file_type: Option<String>,
    pub head_limit: Option<usize>,
    pub offset: Option<usize>,
    pub multiline: Option<bool>,
}

pub fn grep_search(input: &GrepSearchInput) -> Result<String, String> {
    let base_path = match &input.path {
        Some(p) => util::normalize_path(p).map_err(|e| format!("Invalid path: {e}"))?,
        None => std::env::current_dir().map_err(|e| format!("Failed to get CWD: {e}"))?,
    };

    let regex = RegexBuilder::new(&input.pattern)
        .case_insensitive(input.case_insensitive.unwrap_or(false))
        .dot_matches_new_line(input.multiline.unwrap_or(false))
        .build()
        .map_err(|e| format!("Invalid regex pattern: {e}"))?;

    let glob_filter = input
        .glob
        .as_deref()
        .map(Pattern::new)
        .transpose()
        .map_err(|e| format!("Invalid glob filter: {e}"))?;

    let file_type = input.file_type.as_deref();
    let output_mode = input
        .output_mode
        .clone()
        .unwrap_or_else(|| "files_with_matches".into());
    let context = input.context.or(input.context_short).unwrap_or(0);

    let to_relative = |p: &Path| -> String {
        pathdiff::diff_paths(p, &base_path)
            .unwrap_or_else(|| p.to_path_buf())
            .to_string_lossy()
            .into_owned()
    };

    let mut filenames = Vec::new();
    let mut content_lines = Vec::new();
    let mut total_matches = 0usize;

    // For files_with_matches mode, collect (path, mtime) pairs for sorting
    let mut file_mtimes: Vec<(String, Option<std::time::SystemTime>)> = Vec::new();

    for file_path in collect_search_files(&base_path).map_err(|e| format!("Search failed: {e}"))? {
        if !matches_filters(&file_path, glob_filter.as_ref(), file_type) {
            continue;
        }

        let Ok(file_contents) = fs::read_to_string(&file_path) else {
            continue;
        };

        let rel_path = to_relative(&file_path);

        if output_mode == "count" {
            let count = regex.find_iter(&file_contents).count();
            if count > 0 {
                filenames.push(rel_path);
                total_matches += count;
            }
            continue;
        }

        let lines: Vec<&str> = file_contents.lines().collect();
        let mut matched_lines = Vec::new();
        for (index, line) in lines.iter().enumerate() {
            if regex.is_match(line) {
                total_matches += 1;
                matched_lines.push(index);
            }
        }

        if matched_lines.is_empty() {
            continue;
        }

        if output_mode == "files_with_matches" {
            let mtime = fs::metadata(&file_path).and_then(|m| m.modified()).ok();
            file_mtimes.push((rel_path, mtime));
            continue;
        }
        filenames.push(rel_path.clone());

        if output_mode == "content" {
            for index in matched_lines {
                let start = index.saturating_sub(input.before.unwrap_or(context));
                let end = (index + input.after.unwrap_or(context) + 1).min(lines.len());
                for (current, line) in lines.iter().enumerate().take(end).skip(start) {
                    let prefix = if input.line_numbers.unwrap_or(true) {
                        format!("{rel_path}:{}:", current + 1)
                    } else {
                        format!("{rel_path}:")
                    };
                    let truncated_line = if line.len() > 500 { &line[..500] } else { line };
                    content_lines.push(format!("{prefix}{truncated_line}"));
                }
            }
        }
    }

    if output_mode == "content" {
        let (lines, limit, offset) = apply_limit(content_lines, input.head_limit, input.offset);
        let output = GrepSearchOutput {
            mode: Some(output_mode),
            num_files: filenames.len(),
            filenames,
            num_lines: Some(lines.len()),
            content: Some(lines.join("\n")),
            num_matches: None,
            applied_limit: limit,
            applied_offset: offset,
        };
        return serde_json::to_string_pretty(&output)
            .map_err(|e| format!("Failed to serialize output: {e}"));
    }

    // For files_with_matches, sort by mtime (newest first) then extract names
    if output_mode == "files_with_matches" {
        file_mtimes.sort_by_key(|(_, mtime)| mtime.map(Reverse));
        filenames = file_mtimes.into_iter().map(|(name, _)| name).collect();
    }

    let (filenames, applied_limit, applied_offset) =
        apply_limit(filenames, input.head_limit, input.offset);

    let output = GrepSearchOutput {
        mode: Some(output_mode.clone()),
        num_files: filenames.len(),
        filenames,
        content: None,
        num_lines: None,
        num_matches: (output_mode == "count").then_some(total_matches),
        applied_limit,
        applied_offset,
    };

    serde_json::to_string_pretty(&output)
        .map_err(|e| format!("Failed to serialize output: {e}"))
}

const VCS_DIRS: &[&str] = &[".git", ".svn", ".hg", ".bzr", ".jj", ".sl"];

fn is_vcs_dir(entry: &walkdir::DirEntry) -> bool {
    if entry.file_type().is_dir() {
        let name = entry.file_name().to_string_lossy();
        return VCS_DIRS.iter().any(|&vcs| name == vcs);
    }
    false
}

fn collect_search_files(base_path: &Path) -> std::io::Result<Vec<PathBuf>> {
    if base_path.is_file() {
        return Ok(vec![base_path.to_path_buf()]);
    }

    let mut files = Vec::new();
    let walker = WalkDir::new(base_path).into_iter().filter_entry(|e| !is_vcs_dir(e));
    for entry in walker {
        let entry = entry.map_err(|e| std::io::Error::other(e.to_string()))?;
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }
    Ok(files)
}

fn matches_filters(path: &Path, glob_filter: Option<&Pattern>, file_type: Option<&str>) -> bool {
    if let Some(glob_filter) = glob_filter {
        let path_string = path.to_string_lossy();
        if !glob_filter.matches(&path_string) && !glob_filter.matches_path(path) {
            return false;
        }
    }

    if let Some(file_type) = file_type {
        let extension = path.extension().and_then(|e| e.to_str());
        if extension != Some(file_type) {
            return false;
        }
    }

    true
}

fn apply_limit<T>(
    items: Vec<T>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> (Vec<T>, Option<usize>, Option<usize>) {
    let offset_value = offset.unwrap_or(0);
    let mut items: Vec<T> = items.into_iter().skip(offset_value).collect();
    let explicit_limit = limit.unwrap_or(250);
    if explicit_limit == 0 {
        return (items, None, (offset_value > 0).then_some(offset_value));
    }

    let truncated = items.len() > explicit_limit;
    items.truncate(explicit_limit);
    (
        items,
        truncated.then_some(explicit_limit),
        (offset_value > 0).then_some(offset_value),
    )
}
