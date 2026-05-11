use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
