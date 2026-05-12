//! Plugin discovery on the local filesystem.
//!
//! Plugins live at `<base_dir>/plugins/<name>` (or `<name>.exe` on
//! Windows). The cli's external-subcommand dispatch uses
//! [`Client::resolve_plugin`] to turn a user-supplied plugin name
//! into an executable path.

use std::path::{Path, PathBuf};

use super::super::Client;
use super::{Manifest, ManifestWithNameAndSource};

/// Two-step parse: try `ManifestWithNameAndSource` first (installed
/// plugins persist `name` + `source`), then fall back to a bare
/// `Manifest` with `name = file_stem` and `source = absolute_path`
/// (hand-edited / pre-existing manifests). Returns `None` on missing
/// / unreadable / malformed files.
async fn parse_manifest_file(path: &Path) -> Option<ManifestWithNameAndSource> {
    let bytes = tokio::fs::read(path).await.ok()?;
    if let Ok(full) = serde_json::from_slice::<ManifestWithNameAndSource>(&bytes) {
        return Some(full);
    }
    let manifest: Manifest = serde_json::from_slice(&bytes).ok()?;
    let name = path.file_stem()?.to_str()?.to_string();
    let source = path.to_string_lossy().into_owned();
    Some(ManifestWithNameAndSource { name, manifest, source })
}

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

    /// Look up a single plugin manifest by name. Reads
    /// `<base_dir>/plugins/<name>.json`. If the file persists `name`
    /// and `source` (as installed plugins do), they're returned
    /// verbatim; otherwise the wrapper is synthesized with
    /// `name = <name>` and `source = absolute_path`. Returns `None`
    /// if the file is missing, unreadable, or malformed.
    pub async fn get_plugin(&self, name: &str) -> Option<ManifestWithNameAndSource> {
        let path = self.plugins_dir().join(format!("{name}.json"));
        parse_manifest_file(&path).await
    }

    /// Enumerate plugin manifests in the plugins directory. Reads each
    /// `.json` file in `<base_dir>/plugins/`, deserializes it as a
    /// [`Manifest`], and pairs it with the file's stem (`name`) and
    /// absolute path (`source`). Every failure mode — missing dir,
    /// unreadable file, malformed JSON, missing required field — is
    /// silently skipped; the return type is plain `Vec` rather than
    /// `Result` to reflect that.
    ///
    /// Results are sorted by manifest mtime descending (most recently
    /// modified first), then `skip(offset).take(limit)` is applied —
    /// matching the convention of the logs list endpoints. Pass
    /// `(0, usize::MAX)` for an unbounded list.
    ///
    /// The directory scan is sequential (intrinsic to `read_dir`) but
    /// per-file read+parse runs concurrently via
    /// [`futures::future::join_all`].
    pub async fn list_plugins(&self, offset: usize, limit: usize) -> Vec<ManifestWithNameAndSource> {
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
            let bundle = parse_manifest_file(&p).await?;
            let modified = tokio::fs::metadata(&p)
                .await
                .ok()?
                .modified()
                .ok()?
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .ok()?
                .as_secs();
            Some((modified, bundle))
        });
        let mut entries: Vec<(u64, ManifestWithNameAndSource)> = futures::future::join_all(futures)
            .await
            .into_iter()
            .flatten()
            .collect();
        entries.sort_by(|a, b| b.0.cmp(&a.0));
        let iter = entries.into_iter().map(|(_, m)| m);
        if offset > 0 || limit < usize::MAX {
            iter.skip(offset).take(limit).collect()
        } else {
            iter.collect()
        }
    }
}

#[cfg(feature = "http")]
impl Client {
    /// Install a plugin from a GitHub repository.
    ///
    /// 1. Fetches `objectiveai.json` from `raw.githubusercontent.com`
    ///    at the supplied `commit_sha` (or the default branch via
    ///    `HEAD` when none).
    /// 2. Parses it as a [`Manifest`].
    /// 3. Looks up the current platform in `manifest.binaries`. If
    ///    absent (or this host's platform isn't recognized by
    ///    [`super::Platform::current`]), returns `Ok(false)` — the
    ///    plugin simply doesn't support this host.
    /// 4. Downloads the matching release asset from
    ///    `https://github.com/<owner>/<repository>/releases/download/v<version>/<asset>`.
    /// 5. Writes it to `<base_dir>/plugins/<repository>/plugin`
    ///    (`plugin.exe` on Windows). Sets mode `0o755` on Unix so the
    ///    binary is executable.
    ///
    /// `headers` is an optional `IndexMap<String, String>` that gets
    /// attached to both HTTP requests (e.g. `Authorization` for
    /// private repos / higher rate limits). The cli always passes
    /// `None`.
    ///
    /// Failures past step 3 are returned as
    /// [`super::InstallError`] wrapped by
    /// [`super::super::Error::Install`].
    pub async fn install_plugin(
        &self,
        owner: &str,
        repository: &str,
        commit_sha: Option<&str>,
        headers: Option<&indexmap::IndexMap<String, String>>,
    ) -> Result<bool, super::super::Error> {
        let manifest = self
            .fetch_plugin_manifest(owner, repository, commit_sha, headers)
            .await?;
        let source = raw_manifest_url(owner, repository, commit_sha);
        self.install_plugin_from_manifest(owner, repository, &manifest, &source, headers)
            .await
    }

    /// Step 1 of `install_plugin`: fetch `<owner>/<repo>/<ref>/objectiveai.json`
    /// from `raw.githubusercontent.com` and parse it as a [`Manifest`].
    /// Exposed publicly so callers can inspect the manifest before
    /// committing to an install (e.g. for whitelist checks).
    pub async fn fetch_plugin_manifest(
        &self,
        owner: &str,
        repository: &str,
        commit_sha: Option<&str>,
        headers: Option<&indexmap::IndexMap<String, String>>,
    ) -> Result<Manifest, super::super::Error> {
        self.fetch_plugin_manifest_impl(
            "https://raw.githubusercontent.com",
            owner,
            repository,
            commit_sha,
            headers,
        )
        .await
    }

    /// Step 2 of `install_plugin`: given an already-parsed manifest,
    /// pick the binary for the current platform (`Ok(false)` if
    /// absent), download it from the corresponding release asset,
    /// and write it to `<plugins_dir>/<repository>/plugin[.exe]`.
    pub async fn install_plugin_from_manifest(
        &self,
        owner: &str,
        repository: &str,
        manifest: &Manifest,
        source: &str,
        headers: Option<&indexmap::IndexMap<String, String>>,
    ) -> Result<bool, super::super::Error> {
        self.install_from_manifest_impl(
            "https://github.com",
            owner,
            repository,
            manifest,
            source,
            headers,
        )
        .await
    }

    /// Test-only entry point that exposes the raw / releases URL
    /// bases so in-process mock servers can intercept the requests.
    /// Threads both URLs through the same fetch + install_from path
    /// used by production.
    #[cfg(test)]
    pub(super) async fn install_plugin_at(
        &self,
        raw_base: &str,
        releases_base: &str,
        owner: &str,
        repository: &str,
        commit_sha: Option<&str>,
        headers: Option<&indexmap::IndexMap<String, String>>,
    ) -> Result<bool, super::super::Error> {
        let manifest = self
            .fetch_plugin_manifest_impl(raw_base, owner, repository, commit_sha, headers)
            .await?;
        let reference = commit_sha.unwrap_or("HEAD");
        let source = format!("{raw_base}/{owner}/{repository}/{reference}/objectiveai.json");
        self.install_from_manifest_impl(
            releases_base,
            owner,
            repository,
            &manifest,
            &source,
            headers,
        )
        .await
    }

    /// Test-only fetch-only entry point, mirrors `install_plugin_at`.
    #[cfg(test)]
    pub(super) async fn fetch_plugin_manifest_at(
        &self,
        raw_base: &str,
        owner: &str,
        repository: &str,
        commit_sha: Option<&str>,
        headers: Option<&indexmap::IndexMap<String, String>>,
    ) -> Result<Manifest, super::super::Error> {
        self.fetch_plugin_manifest_impl(raw_base, owner, repository, commit_sha, headers)
            .await
    }

    async fn fetch_plugin_manifest_impl(
        &self,
        raw_base: &str,
        owner: &str,
        repository: &str,
        commit_sha: Option<&str>,
        headers: Option<&indexmap::IndexMap<String, String>>,
    ) -> Result<Manifest, super::super::Error> {
        let http = reqwest::Client::new();
        let header_map = build_headers(headers)?;
        let reference = commit_sha.unwrap_or("HEAD");
        let manifest_url =
            format!("{raw_base}/{owner}/{repository}/{reference}/objectiveai.json");
        let resp = http
            .get(&manifest_url)
            .headers(header_map)
            .send()
            .await
            .map_err(super::InstallError::ManifestRequest)?;
        let status = resp.status();
        let bytes = resp
            .bytes()
            .await
            .map_err(super::InstallError::ManifestResponse)?;
        if !status.is_success() {
            return Err(super::InstallError::ManifestBadStatus {
                code: status,
                url: manifest_url,
                body: String::from_utf8_lossy(&bytes).into_owned(),
            }
            .into());
        }
        let mut de = serde_json::Deserializer::from_slice(&bytes);
        let manifest: Manifest = serde_path_to_error::deserialize(&mut de)
            .map_err(super::InstallError::ManifestParse)?;
        Ok(manifest)
    }

    async fn install_from_manifest_impl(
        &self,
        releases_base: &str,
        owner: &str,
        repository: &str,
        manifest: &Manifest,
        source: &str,
        headers: Option<&indexmap::IndexMap<String, String>>,
    ) -> Result<bool, super::super::Error> {
        let http = reqwest::Client::new();
        let header_map = build_headers(headers)?;

        // 1. Match platform
        let Some(platform) = super::Platform::current() else {
            return Ok(false);
        };
        let Some(binary_name) = manifest.binaries.get(&platform) else {
            return Ok(false);
        };

        // 2. Fetch binary
        let binary_url = format!(
            "{releases_base}/{owner}/{repository}/releases/download/v{version}/{binary_name}",
            version = manifest.version,
        );
        let resp = http
            .get(&binary_url)
            .headers(header_map)
            .send()
            .await
            .map_err(super::InstallError::BinaryRequest)?;
        let status = resp.status();
        if !status.is_success() {
            return Err(super::InstallError::BinaryBadStatus {
                code: status,
                url: binary_url,
            }
            .into());
        }
        let bin_bytes = resp
            .bytes()
            .await
            .map_err(super::InstallError::BinaryResponse)?;

        // 3. Write to <plugins_dir>/<repository>/plugin[.exe]
        let plugin_dir = self.plugins_dir().join(repository);
        tokio::fs::create_dir_all(&plugin_dir)
            .await
            .map_err(|e| super::InstallError::PluginDirCreate(plugin_dir.clone(), e))?;
        let binary_filename = if cfg!(windows) { "plugin.exe" } else { "plugin" };
        let binary_path = plugin_dir.join(binary_filename);
        tokio::fs::write(&binary_path, &bin_bytes)
            .await
            .map_err(|e| super::InstallError::BinaryWrite(binary_path.clone(), e))?;

        // 4. chmod +x on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o755);
            tokio::fs::set_permissions(&binary_path, perms)
                .await
                .map_err(|e| super::InstallError::Chmod(binary_path.clone(), e))?;
        }

        // 5. Fetch + extract the viewer UI bundle into
        // <plugins_dir>/<repository>/viewer/ if the manifest declares
        // a `viewer_zip`.
        if let Some(viewer_zip_name) = &manifest.viewer_zip {
            let viewer_url = format!(
                "{releases_base}/{owner}/{repository}/releases/download/v{version}/{viewer_zip_name}",
                version = manifest.version,
            );
            let viewer_header_map = build_headers(headers)?;
            let resp = http
                .get(&viewer_url)
                .headers(viewer_header_map)
                .send()
                .await
                .map_err(super::InstallError::ViewerZipRequest)?;
            let status = resp.status();
            if !status.is_success() {
                return Err(super::InstallError::ViewerZipBadStatus {
                    code: status,
                    url: viewer_url,
                }
                .into());
            }
            let zip_bytes = resp
                .bytes()
                .await
                .map_err(super::InstallError::ViewerZipResponse)?;
            let viewer_dir = plugin_dir.join("viewer");
            tokio::fs::create_dir_all(&viewer_dir)
                .await
                .map_err(|e| super::InstallError::ViewerZipExtract(viewer_dir.clone(), e.to_string()))?;
            let viewer_dir_for_blocking = viewer_dir.clone();
            tokio::task::spawn_blocking(move || {
                let cursor = std::io::Cursor::new(zip_bytes);
                let mut archive = zip::ZipArchive::new(cursor)
                    .map_err(|e| format!("zip archive open: {e}"))?;
                archive
                    .extract(&viewer_dir_for_blocking)
                    .map_err(|e| format!("extract: {e}"))
            })
            .await
            .map_err(|e| super::InstallError::ViewerZipExtract(viewer_dir.clone(), format!("join: {e}")))?
            .map_err(|e| super::InstallError::ViewerZipExtract(viewer_dir.clone(), e))?;
        }

        // 6. Persist the manifest as <plugins_dir>/<repository>.json so
        // list_plugins / get_plugin surface this install.
        let manifest_path = self.plugins_dir().join(format!("{repository}.json"));
        let bundle = ManifestWithNameAndSource {
            name: repository.to_string(),
            manifest: manifest.clone(),
            source: source.to_string(),
        };
        let bytes = serde_json::to_vec_pretty(&bundle)
            .map_err(super::InstallError::ManifestSerialize)?;
        tokio::fs::write(&manifest_path, &bytes)
            .await
            .map_err(|e| super::InstallError::ManifestPersist(manifest_path.clone(), e))?;

        Ok(true)
    }
}

/// Convention: the raw-GitHub URL we'd fetch `objectiveai.json` from
/// for a given (owner, repository, optional commit sha). Defaults to
/// `HEAD` when no commit is supplied. Lifted out so the cli and the
/// SDK's own `install_plugin` wrapper share one source of truth.
pub fn raw_manifest_url(owner: &str, repository: &str, commit_sha: Option<&str>) -> String {
    let reference = commit_sha.unwrap_or("HEAD");
    format!(
        "https://raw.githubusercontent.com/{owner}/{repository}/{reference}/objectiveai.json"
    )
}

#[cfg(feature = "http")]
pub(super) fn build_headers(
    headers: Option<&indexmap::IndexMap<String, String>>,
) -> Result<reqwest::header::HeaderMap, super::InstallError> {
    let mut out = reqwest::header::HeaderMap::new();
    let Some(h) = headers else {
        return Ok(out);
    };
    for (k, v) in h {
        let name = reqwest::header::HeaderName::from_bytes(k.as_bytes()).map_err(|e| {
            super::InstallError::InvalidHeaderName {
                name: k.clone(),
                reason: e.to_string(),
            }
        })?;
        let value = reqwest::header::HeaderValue::from_str(v).map_err(|e| {
            super::InstallError::InvalidHeaderValue {
                name: k.clone(),
                reason: e.to_string(),
            }
        })?;
        out.insert(name, value);
    }
    Ok(out)
}
