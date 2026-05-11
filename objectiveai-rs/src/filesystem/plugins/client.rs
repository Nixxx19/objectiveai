//! Plugin discovery on the local filesystem.
//!
//! Plugins live at `<base_dir>/plugins/<name>` (or `<name>.exe` on
//! Windows). The cli's external-subcommand dispatch uses
//! [`Client::resolve_plugin`] to turn a user-supplied plugin name
//! into an executable path.

use std::path::PathBuf;

use super::super::Client;
use super::{Manifest, ManifestWithNameAndSource};

impl Client {
    /// The plugins directory: `<base_dir>/plugins`.
    pub fn plugins_dir(&self) -> PathBuf {
        self.base_dir().join("plugins")
    }

    /// Resolve a plugin name to its executable path. Returns `Some(path)`
    /// when either `<plugins_dir>/<name>` or `<plugins_dir>/<name>.exe`
    /// exists; `None` otherwise. The non-extension form is tried first
    /// to match Unix convention; `.exe` is the Windows fallback (also
    /// harmless to attempt on Unix).
    ///
    /// Uses `tokio::fs::metadata` so it doesn't block the runtime.
    pub async fn resolve_plugin(&self, name: &str) -> Option<PathBuf> {
        let dir = self.plugins_dir();
        let bare = dir.join(name);
        if tokio::fs::metadata(&bare)
            .await
            .map(|m| m.is_file())
            .unwrap_or(false)
        {
            return Some(bare);
        }
        let exe = dir.join(format!("{name}.exe"));
        if tokio::fs::metadata(&exe)
            .await
            .map(|m| m.is_file())
            .unwrap_or(false)
        {
            return Some(exe);
        }
        None
    }

    /// Enumerate every plugin manifest in the plugins directory. Reads
    /// each `.json` file in `<base_dir>/plugins/`, deserializes it as a
    /// [`Manifest`], and pairs it with the file's stem (`name`) and
    /// absolute path (`source`). Every failure mode — missing dir,
    /// unreadable file, malformed JSON, missing required field — is
    /// silently skipped; the return type is plain `Vec` rather than
    /// `Result` to reflect that.
    ///
    /// The directory scan is sequential (intrinsic to `read_dir`) but
    /// per-file read+parse runs concurrently via
    /// [`futures::future::join_all`]. Order of the returned vec is
    /// unspecified — sort at the call site if a stable order matters.
    pub async fn list_plugins(&self) -> Vec<ManifestWithNameAndSource> {
        let dir = self.plugins_dir();
        let Ok(mut read_dir) = tokio::fs::read_dir(&dir).await else {
            return Vec::new();
        };
        let mut paths: Vec<PathBuf> = Vec::new();
        while let Ok(Some(entry)) = read_dir.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                paths.push(path);
            }
        }
        let futures = paths.into_iter().map(|p| async move {
            let bytes = tokio::fs::read(&p).await.ok()?;
            let manifest: Manifest = serde_json::from_slice(&bytes).ok()?;
            let name = p.file_stem()?.to_str()?.to_string();
            let source = p.to_string_lossy().into_owned();
            Some(ManifestWithNameAndSource { name, manifest, source })
        });
        futures::future::join_all(futures)
            .await
            .into_iter()
            .flatten()
            .collect()
    }
}
