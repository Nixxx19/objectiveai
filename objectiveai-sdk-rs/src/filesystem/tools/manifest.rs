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

    /// Version string. Free-form; the host displays whatever's here
    /// (semver convention recommended but not enforced).
    pub version: String,

    /// GitHub-style owner (user or org) of the tool's source repo.
    /// Free-form; tools have no installer, so this is purely
    /// author-supplied metadata — nothing overrides it the way the
    /// plugin installer overrides the plugin manifest's owner.
    pub owner: String,

    /// Filename of the executable to invoke, resolved relative to
    /// `<base_dir>/tools/`. The author is responsible for including
    /// any platform-specific extension (`.exe`, `.sh`, `.bat`, …) —
    /// the SDK doesn't synthesise one.
    pub exec: String,
}

impl Manifest {
    /// LLM-visible tool name: `{owner}-{name}-{version}` with every
    /// `.` substituted to `-`, mirroring
    /// `filesystem::plugins::Manifest::tool_name`. The substitution
    /// keeps the result Anthropic-tool-name-regex safe
    /// (`^[a-zA-Z0-9_-]{1,128}$`) even when the version field carries
    /// semver dots (`1.2.3` -> `1-2-3`).
    pub fn tool_name(&self, name: &str) -> String {
        format!("{}-{}-{}", self.owner, name, self.version).replace('.', "-")
    }
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

impl ManifestWithNameAndSource {
    /// LLM-visible tool name. See [`Manifest::tool_name`] — this
    /// helper supplies the `name` field automatically.
    pub fn tool_name(&self) -> String {
        self.manifest.tool_name(&self.name)
    }
}
