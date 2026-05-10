//! Structured JSON Lines output for `objectiveai-cli`.
//!
//! Every line `objectiveai-cli` writes to stdout is one `Output` JSON object.
//! There are two top-level shapes, discriminated by `"type"`:
//!
//! - `error` — a failure or non-fatal advisory, with a log level and a
//!   `fatal` flag. No machine-readable error code: the message is the
//!   message.
//! - `notification` — everything else: command results, config values,
//!   acknowledgements, runtime lifecycle events. Two-tier: the outer
//!   variant chooses a category (`kind`), the inner variant chooses a
//!   specific shape (`subkind`).
//!
//! ## No-arbitrary-JSON policy
//!
//! Output shapes are strongly typed. `serde_json::Value` appears in
//! exactly two places, both explicit escape hatches:
//!
//! 1. [`LogContent::Json`] — log files contain arbitrary API payloads
//!    that we can't constrain at this layer.
//! 2. [`Config::Jq`] — output of a user-supplied `jq` filter, which by
//!    definition can produce any JSON.
//!
//! Anywhere else, structured data is a typed Rust value.

use serde::{Deserialize, Serialize};

// ============================================================================
// === Top-level                                                             ===
// ============================================================================

/// A single line of CLI output.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Output {
    Error(Error),
    Notification(Notification),
}

// ============================================================================
// === Error                                                                 ===
// ============================================================================

/// A failure or advisory written to stdout. `fatal: true` means the
/// process is exiting with a non-zero status; `fatal: false` is a
/// non-blocking warning (e.g. auto-update failed but the requested
/// command still ran).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Error {
    pub level: Level,
    pub fatal: bool,
    pub message: String,
}

/// Severity matching the conventions used by bunyan / pino / `log` crate
/// JSON encoders. `fatal` is encoded separately on [`Error`].
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

// ============================================================================
// === Notification                                                          ===
// ============================================================================

/// Non-error output. Each variant nests a `subkind`-tagged enum that
/// pins down the exact shape.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Notification {
    Resource(Resource),
    List(List),
    Execution(Execution),
    Log(Log),
    Schema(Schema),
    Instructions(Instructions),
    Config(Config),
    Ack(Ack),
    Process(Process),
}

// ----------------------------------------------------------------------------
// --- Resource (single typed resource from a `*/get` endpoint)             ---
// ----------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "subkind", rename_all = "snake_case")]
pub enum Resource {
    /// Emitted by `agents get`.
    Agent(Box<objectiveai::agent::response::GetAgentResponse>),
    /// Emitted by `swarms get`.
    Swarm(Box<objectiveai::swarm::response::GetSwarmResponse>),
    /// Emitted by `functions get`.
    Function(Box<objectiveai::functions::response::GetFunctionResponse>),
    /// Emitted by `functions profiles get`.
    Profile(Box<objectiveai::functions::profiles::response::GetProfileResponse>),
    /// Emitted by `functions profiles pairs get`. The CLI fetches both
    /// halves and returns them together; we mirror that composite shape.
    Pair(Box<Pair>),
    /// Emitted by `functions inventions state get`.
    InventionState(
        Box<objectiveai::functions::inventions::state::response::GetFunctionInventionStateResponse>,
    ),
}

/// Function + profile composite returned by `functions profiles pairs get`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pair {
    pub function: objectiveai::functions::response::GetFunctionResponse,
    pub profile: objectiveai::functions::profiles::response::GetProfileResponse,
}

// ----------------------------------------------------------------------------
// --- List (listings of resources, logs, schemas, favorites)               ---
// ----------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "subkind", rename_all = "snake_case")]
pub enum List {
    /// Emitted by `agents list`.
    Agents {
        source: ListSource,
        items: Vec<ListItem>,
    },
    /// Emitted by `swarms list`.
    Swarms {
        source: ListSource,
        items: Vec<ListItem>,
    },
    /// Emitted by `functions list`.
    Functions {
        source: ListSource,
        items: Vec<ListItem>,
    },
    /// Emitted by `functions profiles list`.
    Profiles {
        source: ListSource,
        items: Vec<ListItem>,
    },
    /// Emitted by `functions profiles pairs list`.
    Pairs {
        source: ListSource,
        items: Vec<PairListItem>,
    },
    /// Emitted by `<scope> logs list` and the global `logs list`.
    Logs {
        items: Vec<objectiveai::filesystem::logs::ListItem>,
    },
    /// Emitted by `schemas list` and by the per-category `schemas <category> list`.
    Schemas { names: Vec<String> },
    /// Emitted by `<resource> favorites get`. The `resource` field tells
    /// the consumer which favorite list this is.
    Favorites {
        resource: FavoriteResource,
        items: Vec<objectiveai::filesystem::config::Favorite>,
    },
    /// Emitted by `functions profiles pairs favorites get`. Kept as a
    /// distinct variant because pair favorites have a different shape
    /// (two paths instead of one).
    PairFavorites {
        items: Vec<objectiveai::filesystem::config::PairFavorite>,
    },
}

/// Where a listing was fetched from. Mirrors `objectiveai_cli::list::Source`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ListSource {
    Filesystem,
    Favorites,
    Objectiveai,
    Mock,
    All,
}

/// One entry in a non-pair listing — either a favorite reference or a
/// resolved remote path. Untagged so the wire shape is whichever
/// underlying object matches.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ListItem {
    Favorite(objectiveai::filesystem::config::Favorite),
    Path(objectiveai::RemotePath),
}

/// One entry in a function-profile pair listing.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PairListItem {
    Favorite(objectiveai::filesystem::config::PairFavorite),
    Item(objectiveai::functions::response::ListFunctionProfilePairItem),
}

/// Which favorite collection a Favorites notification refers to.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FavoriteResource {
    Agent,
    Swarm,
    Function,
    Profile,
}

// ----------------------------------------------------------------------------
// --- Execution (terminal results of `*/create` endpoints)                 ---
// ----------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "subkind", rename_all = "snake_case")]
pub enum Execution {
    /// Emitted by `functions executions create`.
    Function(Box<objectiveai::functions::executions::response::unary::FunctionExecution>),
    /// Emitted by `laboratories executions create`.
    Laboratory(Box<objectiveai::laboratories::executions::response::unary::LaboratoryExecution>),
    /// Emitted by `functions inventions create`.
    InventionCreate(Box<objectiveai::functions::inventions::response::unary::FunctionInvention>),
    /// Emitted by `functions inventions recursive create`.
    InventionRecursiveCreate(
        Box<
            objectiveai::functions::inventions::recursive::response::unary::FunctionInventionRecursive,
        >,
    ),
    /// Emitted by `vector completions`.
    VectorCompletion(Box<objectiveai::vector::completions::response::unary::VectorCompletion>),
}

// ----------------------------------------------------------------------------
// --- Log                                                                   ---
// ----------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "subkind", rename_all = "snake_case")]
pub enum Log {
    /// Emitted by `<scope> logs get` and `<scope> logs subscribe` when
    /// the requested log resolves.
    Content { content: LogContent },
    /// Emitted by `<scope> logs clear` and global `logs clear`.
    Cleared { count: u64 },
    /// Emitted by streaming `create` commands once a log id has been
    /// allocated and the log file is available — replaces the bare
    /// `println!("Logs ID: ...")` in `objectiveai-cli/src/log_line.rs`.
    StreamReady { id: String },
    /// Emitted by `<scope> logs subscribe` if no matching log appears
    /// before the timeout.
    SubscribeTimedOut,
}

/// Contents of a log file. The `objectiveai::filesystem::logs::LogContent`
/// upstream has no serde derives, so we mirror its shape here with our
/// own wire form.
///
/// `LogContent::Json` carries the unmodified API response, which is
/// arbitrary structured JSON — one of the two intentional uses of
/// `serde_json::Value` in this module.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "encoding", rename_all = "snake_case")]
pub enum LogContent {
    Json { value: serde_json::Value },
    /// `data:{mime};base64,{payload}` string for binary log content.
    DataUrl { url: String },
}

// ----------------------------------------------------------------------------
// --- Schema                                                                ---
// ----------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "subkind", rename_all = "snake_case")]
pub enum Schema {
    /// Emitted by every `schemas <category> <type> get` command. The
    /// schema is a real JSON Schema object, not a stringified blob.
    Get { name: String, schema: schemars::Schema },
}

// ----------------------------------------------------------------------------
// --- Instructions                                                          ---
// ----------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "subkind", rename_all = "snake_case")]
pub enum Instructions {
    /// Emitted by `<scope> instructions get` and the global `instructions get`.
    Get { id: Option<String>, text: String },
    /// Emitted by `instructions list`.
    List { ids: Vec<String> },
}

// ----------------------------------------------------------------------------
// --- Config                                                                ---
// ----------------------------------------------------------------------------

/// Output of a config getter. One variant per logical config family;
/// the payload is the typed value as it lives in the on-disk config
/// (no `Option<&str>` smuggling, no stringification).
///
/// `Jq` is the one untyped escape hatch: a user-supplied jq filter
/// over the config can return literally any JSON shape, including
/// mixed-type arrays, so it must accept `Vec<serde_json::Value>`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "subkind", rename_all = "snake_case")]
pub enum Config {
    /// Emitted by `api mode config get`.
    ApiMode {
        value: objectiveai::filesystem::config::ApiMode,
    },
    /// Emitted by `api remote config get`.
    ApiRemote {
        value: objectiveai::filesystem::config::ApiRemoteConfig,
    },
    /// Emitted by `api headers config get` (the full headers bag).
    ApiHeaders {
        value: objectiveai::filesystem::config::ApiHeadersConfig,
    },
    /// Emitted by `api headers x_objectiveai_authorization config get`.
    ApiHeaderXObjectiveaiAuthorization { value: Option<String> },
    /// Emitted by `api headers x_openrouter_authorization config get`.
    ApiHeaderXOpenrouterAuthorization { value: Option<String> },
    /// Emitted by `api headers x_github_authorization config get`.
    ApiHeaderXGithubAuthorization { value: Option<String> },
    /// Emitted by `api headers x_mcp_authorization config get`. MCP
    /// authorization is keyed per server, hence the map.
    ApiHeaderXMcpAuthorization {
        value: Option<indexmap::IndexMap<String, String>>,
    },
    /// Emitted by `api headers x_viewer_signature config get`.
    ApiHeaderXViewerSignature { value: Option<String> },
    /// Emitted by `api headers x_viewer_address config get`.
    ApiHeaderXViewerAddress { value: Option<String> },
    /// Emitted by `api headers user_agent config get`.
    ApiHeaderUserAgent { value: Option<String> },
    /// Emitted by `api headers http_referer config get`.
    ApiHeaderHttpReferer { value: Option<String> },
    /// Emitted by `api headers x_title config get`.
    ApiHeaderXTitle { value: Option<String> },
    /// Emitted by `api headers x_commit_author_name config get`.
    ApiHeaderXCommitAuthorName { value: Option<String> },
    /// Emitted by `api headers x_commit_author_email config get`.
    ApiHeaderXCommitAuthorEmail { value: Option<String> },
    /// Emitted by `agents config get`.
    Agents {
        value: objectiveai::filesystem::config::AgentsConfig,
    },
    /// Emitted by `swarms config get`.
    Swarms {
        value: objectiveai::filesystem::config::SwarmsConfig,
    },
    /// Emitted by `functions config get`.
    Functions {
        value: objectiveai::filesystem::config::FunctionsConfig,
    },
    /// Emitted by `functions profiles config get`.
    FunctionsProfiles {
        value: objectiveai::filesystem::config::FunctionsProfilesConfig,
    },
    /// Emitted by `functions profiles pairs config get`.
    FunctionsProfilesPairs {
        value: objectiveai::filesystem::config::FunctionsProfilesPairsConfig,
    },
    /// Emitted by `viewer mode config get`.
    ViewerMode {
        value: objectiveai::filesystem::config::ViewerMode,
    },
    /// Emitted by any config getter when the user passes a `--filter`
    /// jq expression. The filter is user-supplied and may return any
    /// JSON shape, so this is the one place untyped values appear.
    Jq { results: Vec<serde_json::Value> },
}

// ----------------------------------------------------------------------------
// --- Ack (silent successes — side-effect commands that previously emitted "ok")
// ----------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "subkind", rename_all = "snake_case")]
pub enum Ack {
    /// Emitted by every `* config set` command.
    ConfigSet { key: String },
    /// Emitted by `<resource> favorites add`.
    FavoriteAdded {
        resource: FavoriteResource,
        path: objectiveai::RemotePathCommitOptional,
    },
    /// Emitted by `<resource> favorites del`.
    FavoriteRemoved {
        resource: FavoriteResource,
        name: String,
    },
    /// Emitted by `<resource> favorites edit`.
    FavoriteEdited {
        resource: FavoriteResource,
        name: String,
    },
    /// Emitted by `functions profiles pairs favorites add`.
    PairFavoriteAdded {
        function: objectiveai::RemotePathCommitOptional,
        profile: objectiveai::RemotePathCommitOptional,
    },
    /// Emitted by `functions profiles pairs favorites del`.
    PairFavoriteRemoved { name: String },
    /// Emitted by `functions profiles pairs favorites edit`.
    PairFavoriteEdited { name: String },
    /// Emitted by `instructions clear` and the per-scope `instructions clear`.
    InstructionsCleared,
    /// Emitted by `agents publish` (and any future `<resource> publish`).
    /// The SHA identifies the resulting commit on the local filesystem repo.
    Published { sha: String },
}

// ----------------------------------------------------------------------------
// --- Process (runtime lifecycle — replaces stray println!s)               ---
// ----------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "subkind", rename_all = "snake_case")]
pub enum Process {
    /// Emitted by the parent process during `--detach`, replacing the
    /// bare `println!("PID: {pid}")` in `api/detach.rs`.
    Detached { pid: u32 },
    /// Emitted by the auto-updater when a new release is detected.
    UpdateAvailable { version: String },
    /// Emitted by the auto-updater after a new binary has been installed.
    /// Update *failures* travel through [`Error`] with `fatal: false`.
    UpdateInstalled { version: String },
}

// ============================================================================
// === Tests                                                                 ===
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn roundtrip(out: &Output) -> serde_json::Value {
        let s = serde_json::to_string(out).unwrap();
        let back: Output = serde_json::from_str(&s).unwrap();
        // Re-serialize the round-tripped value so callers can assert on the
        // canonical wire shape without depending on float ordering quirks.
        serde_json::to_value(&back).unwrap()
    }

    #[test]
    fn error_wire_shape() {
        let out = Output::Error(Error {
            level: Level::Error,
            fatal: true,
            message: "favorite not found: foo".to_string(),
        });
        let v = roundtrip(&out);
        assert_eq!(v["type"], "error");
        assert_eq!(v["level"], "error");
        assert_eq!(v["fatal"], true);
        assert_eq!(v["message"], "favorite not found: foo");
    }

    #[test]
    fn non_fatal_warn_wire_shape() {
        // The auto-updater path: warn level, not fatal.
        let out = Output::Error(Error {
            level: Level::Warn,
            fatal: false,
            message: "auto-update failed".to_string(),
        });
        let v = roundtrip(&out);
        assert_eq!(v["type"], "error");
        assert_eq!(v["level"], "warn");
        assert_eq!(v["fatal"], false);
    }

    #[test]
    fn ack_config_set_wire_shape() {
        let out = Output::Notification(Notification::Ack(Ack::ConfigSet {
            key: "api.mode".to_string(),
        }));
        let v = roundtrip(&out);
        assert_eq!(v["type"], "notification");
        assert_eq!(v["kind"], "ack");
        assert_eq!(v["subkind"], "config_set");
        assert_eq!(v["key"], "api.mode");
    }

    #[test]
    fn config_jq_escape_hatch_wire_shape() {
        let out = Output::Notification(Notification::Config(Config::Jq {
            results: vec![json!({"hello": "world"}), json!(42), json!(null)],
        }));
        let v = roundtrip(&out);
        assert_eq!(v["type"], "notification");
        assert_eq!(v["kind"], "config");
        assert_eq!(v["subkind"], "jq");
        assert_eq!(v["results"][0], json!({"hello": "world"}));
        assert_eq!(v["results"][1], 42);
        assert_eq!(v["results"][2], serde_json::Value::Null);
    }

    #[test]
    fn log_content_json_nested_shape() {
        let out = Output::Notification(Notification::Log(Log::Content {
            content: LogContent::Json {
                value: json!({"completion": {"id": "abc"}}),
            },
        }));
        let v = roundtrip(&out);
        assert_eq!(v["type"], "notification");
        assert_eq!(v["kind"], "log");
        assert_eq!(v["subkind"], "content");
        // LogContent uses a distinct tag name (`encoding`) to avoid
        // colliding with the outer `type`/`kind`/`subkind` discriminators.
        assert_eq!(v["content"]["encoding"], "json");
        assert_eq!(v["content"]["value"], json!({"completion": {"id": "abc"}}));
    }

    #[test]
    fn process_detached_replaces_println() {
        let out = Output::Notification(Notification::Process(Process::Detached { pid: 12345 }));
        let v = roundtrip(&out);
        assert_eq!(v["type"], "notification");
        assert_eq!(v["kind"], "process");
        assert_eq!(v["subkind"], "detached");
        assert_eq!(v["pid"], 12345);
    }

    #[test]
    fn list_pair_listing_uses_pair_items() {
        // Build a Favorite-shaped PairListItem via deserialization so we
        // don't have to hand-construct the upstream type.
        let item_json = json!({
            "name": "fav",
            "function": {"remote": "github", "owner": "o", "repository": "r", "commit": "c"},
            "profile":  {"remote": "github", "owner": "o", "repository": "r", "commit": "c"},
            "note": ""
        });
        let pair_fav: PairListItem = serde_json::from_value(item_json).unwrap();
        let out = Output::Notification(Notification::List(List::Pairs {
            source: ListSource::Favorites,
            items: vec![pair_fav],
        }));
        let v = roundtrip(&out);
        assert_eq!(v["kind"], "list");
        assert_eq!(v["subkind"], "pairs");
        assert_eq!(v["source"], "favorites");
        assert_eq!(v["items"][0]["name"], "fav");
    }
}
