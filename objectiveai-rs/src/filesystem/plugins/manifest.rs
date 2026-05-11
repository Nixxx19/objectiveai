use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Platform;

/// Declarative metadata a plugin ships with itself. The wire shape is
/// JSON; the on-disk convention (sibling file, embedded resource,
/// `--manifest` flag, …) is deliberately out of scope of this struct
/// and will be settled in a follow-up.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "filesystem.plugins.Manifest")]
pub struct Manifest {
    /// One-line description of what the plugin does. Surfaced in
    /// listings and the plugin's `--help`-equivalent UI.
    pub description: String,

    /// Version string. Semver convention is recommended but not
    /// enforced — the host just displays whatever's here.
    pub version: String,

    /// Author or authors of the plugin. Free-form string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Homepage or repository URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,

    /// SPDX license identifier (or any string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Release-asset filename per platform — what the cli should
    /// download from the GitHub release tagged `v<version>` to install
    /// the plugin's binary on each platform. Values are filenames
    /// (e.g. `psyops-linux-x86_64`, `psyops-windows-x86_64.exe`), NOT
    /// URLs; the URL is composed from the repository + tag + asset
    /// name elsewhere.
    ///
    /// **Every platform key is optional.** Declare entries only for
    /// the platforms this plugin actually ships a binary for; absent
    /// platforms are simply not supported by this release. A plugin
    /// shipping only Linux x86_64 declares one entry; a plugin
    /// shipping all six declares six. Empty map ↔ field omitted in
    /// the wire shape.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub binaries: IndexMap<Platform, String>,
}

/// A [`Manifest`] enriched with the plugin's identifying `name` and
/// the `source` it was loaded from. Used when listing or describing
/// installed plugins, where the bare manifest fields are not enough
/// to identify which plugin they belong to or where they came from.
///
/// `name` sits before the manifest body; `source` sits after. The
/// `manifest` field is `#[serde(flatten)]`-ed so the wire shape is
/// one flat JSON object — `serde_json`'s `preserve_order` feature
/// keeps the declared field order, so consumers see `name` first
/// and `source` last.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "filesystem.plugins.ManifestWithNameAndSource")]
pub struct ManifestWithNameAndSource {
    /// The plugin's identifier — the filename it lives under in the
    /// plugins directory (e.g. `psyops` for `~/.objectiveai/plugins/psyops`).
    pub name: String,
    #[serde(flatten)]
    pub manifest: Manifest,
    /// Where this manifest came from — e.g. an absolute filesystem path,
    /// a URL, or a registry reference. Free-form string; the host
    /// just displays it.
    pub source: String,
}
