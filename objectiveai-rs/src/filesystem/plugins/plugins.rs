//! Plugin discovery on the local filesystem.
//!
//! Plugins live at `<base_dir>/plugins/<name>` (or `<name>.exe` on
//! Windows). The cli's external-subcommand dispatch uses
//! [`Client::resolve_plugin`] to turn a user-supplied plugin name
//! into an executable path.

use std::path::PathBuf;

use super::super::Client;

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
}
