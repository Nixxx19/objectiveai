use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Cached state for a file that has been read.
#[derive(Debug, Clone)]
pub struct FileStateEntry {
    /// Normalized file content (CRLF→LF).
    pub content: String,
    /// File modification time in milliseconds since epoch.
    pub timestamp: u64,
    /// Line offset used when reading (None = full read).
    pub offset: Option<usize>,
    /// Line limit used when reading (None = full read).
    pub limit: Option<usize>,
}

impl FileStateEntry {
    /// Returns true if this entry represents a partial view of the file
    /// (i.e., read with offset or limit). Partial views block writes/edits.
    pub fn is_partial_view(&self) -> bool {
        self.offset.is_some() || self.limit.is_some()
    }
}

/// Per-session cache tracking files that have been read.
/// Write and edit operations require a file to be in this cache
/// (and not be a partial view) before they can proceed.
#[derive(Debug, Clone)]
pub struct FileStateCache {
    inner: Arc<RwLock<HashMap<String, FileStateEntry>>>,
}

impl FileStateCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the cached state for a file path.
    pub fn get(&self, path: &str) -> Option<FileStateEntry> {
        self.inner.read().unwrap().get(path).cloned()
    }

    /// Set the cached state for a file path.
    pub fn set(&self, path: String, entry: FileStateEntry) {
        self.inner.write().unwrap().insert(path, entry);
    }

    /// Clear all cached state.
    pub fn clear(&self) {
        self.inner.write().unwrap().clear();
    }
}
