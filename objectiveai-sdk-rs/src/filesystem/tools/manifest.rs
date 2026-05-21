use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Declarative metadata a local tool ships with. The wire shape is
/// JSON: `<base_dir>/tools/<name>.json`. Companion executable lives
/// alongside in `<base_dir>/tools/<exec>`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "filesystem.tools.Manifest")]
pub struct Manifest {
    /// One-line description of what the tool does. Surfaced to
    /// agents that consume the tool.
    pub description: String,

    /// Filename of the executable to invoke, resolved relative to
    /// `<base_dir>/tools/`. The author is responsible for including
    /// any platform-specific extension (`.exe`, `.sh`, `.bat`, …) —
    /// the SDK doesn't synthesise one.
    pub exec: String,
}

/// A [`Manifest`] enriched with the tool's identifying `name` and the
/// `source` it was loaded from. Same shape and ordering convention
/// as `filesystem::plugins::ManifestWithNameAndSource`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "filesystem.tools.ManifestWithNameAndSource")]
pub struct ManifestWithNameAndSource {
    /// The tool's identifier — the filename it lives under in the
    /// tools directory (e.g. `hello` for `~/.objectiveai/tools/hello.json`).
    pub name: String,
    #[serde(flatten)]
    pub manifest: Manifest,
    /// Where this manifest came from — typically an absolute
    /// filesystem path. Free-form string; the host just displays it.
    pub source: String,
}
