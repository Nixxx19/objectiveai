use rmcp::{
    handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content},
    schemars, tool, tool_router,
};

use super::state::FileStateCache;

// --- Input schemas (matching Claude Code exactly) ---

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ReadRequest {
    #[schemars(description = "The absolute path to the file to read")]
    file_path: String,
    #[schemars(description = "The line number to start reading from. Only provide if the file is too large to read at once.")]
    offset: Option<usize>,
    #[schemars(description = "The number of lines to read. Only provide if the file is too large to read at once")]
    limit: Option<usize>,
    #[schemars(description = "Page range for PDF files (e.g., \"1-5\", \"3\", \"10-20\"). Only applicable to PDF files. Maximum 20 pages per request.")]
    pages: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WriteRequest {
    #[schemars(description = "The absolute path to the file to write (must be absolute, not relative)")]
    file_path: String,
    #[schemars(description = "The content to write to the file")]
    content: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EditRequest {
    #[schemars(description = "The absolute path to the file to modify")]
    file_path: String,
    #[schemars(description = "The text to replace")]
    old_string: String,
    #[schemars(description = "The text to replace it with (must be different from old_string)")]
    new_string: String,
    #[schemars(description = "Replace all occurrences of old_string (default false)")]
    replace_all: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BashRequest {
    #[schemars(description = "The command to execute")]
    command: String,
    #[schemars(description = "Optional timeout in milliseconds (max 600000)")]
    timeout: Option<u64>,
    #[schemars(description = "Clear, concise description of what this command does in active voice")]
    description: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GlobRequest {
    #[schemars(description = "The glob pattern to match files against")]
    pattern: String,
    #[schemars(description = "The directory to search in. If not specified, the current working directory will be used.")]
    path: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GrepRequest {
    #[schemars(description = "The regular expression pattern to search for in file contents")]
    pattern: String,
    #[schemars(description = "File or directory to search in (rg PATH). Defaults to current working directory.")]
    path: Option<String>,
    #[schemars(description = "Glob pattern to filter files (e.g. \"*.js\", \"*.{ts,tsx}\")")]
    glob: Option<String>,
    #[schemars(description = "Output mode: \"content\" shows matching lines, \"files_with_matches\" shows only file paths (default), \"count\" shows match counts")]
    output_mode: Option<String>,
    #[serde(rename = "-B")]
    #[schemars(rename = "-B", description = "Number of lines to show before each match")]
    before: Option<usize>,
    #[serde(rename = "-A")]
    #[schemars(rename = "-A", description = "Number of lines to show after each match")]
    after: Option<usize>,
    #[serde(rename = "-C")]
    #[schemars(rename = "-C", description = "Alias for context.")]
    context_short: Option<usize>,
    #[schemars(description = "Number of lines to show before and after each match")]
    context: Option<usize>,
    #[serde(rename = "-n")]
    #[schemars(rename = "-n", description = "Show line numbers in output. Defaults to true.")]
    line_numbers: Option<bool>,
    #[serde(rename = "-i")]
    #[schemars(rename = "-i", description = "Case insensitive search")]
    case_insensitive: Option<bool>,
    #[serde(rename = "type")]
    #[schemars(rename = "type", description = "File type to search (e.g., \"js\", \"py\", \"rust\")")]
    file_type: Option<String>,
    #[schemars(description = "Limit output to first N lines/entries. Defaults to 250 when unspecified. Pass 0 for unlimited.")]
    head_limit: Option<usize>,
    #[schemars(description = "Skip first N lines/entries before applying head_limit. Defaults to 0.")]
    offset: Option<usize>,
    #[schemars(description = "Enable multiline mode where . matches newlines and patterns can span lines. Default: false.")]
    multiline: Option<bool>,
}

// --- Tool server ---

#[derive(Debug, Clone)]
pub struct FilesystemTools {
    pub tool_router: ToolRouter<Self>,
    file_state: FileStateCache,
    shell_state: super::bash::ShellState,
}

#[tool_router]
impl FilesystemTools {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            shell_state: super::bash::ShellState::new(),
            file_state: FileStateCache::new(),
        }
    }

    /// Initialize session state (shell snapshot, etc.).
    /// Should be called once after construction.
    pub async fn init(&self) {
        self.shell_state.init_snapshot().await;
    }

    #[tool(name = "Read", description = "Reads a file from the local filesystem.")]
    fn read(&self, Parameters(req): Parameters<ReadRequest>) -> Result<CallToolResult, rmcp::ErrorData> {
        match super::read_file::read_file(&self.file_state, &req.file_path, req.offset, req.limit) {
            Ok(super::read_file::ReadOutput::Text(json)) => {
                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Ok(super::read_file::ReadOutput::Image { base64, media_type }) => {
                Ok(CallToolResult::success(vec![Content::image(base64, media_type)]))
            }
            Ok(super::read_file::ReadOutput::Notebook(blocks)) => {
                let contents: Vec<Content> = blocks.into_iter().map(|b| match b {
                    super::notebook::NotebookBlock::Text(text) => Content::text(text),
                    super::notebook::NotebookBlock::Image { base64, media_type } => {
                        Content::image(base64, media_type)
                    }
                }).collect();
                Ok(CallToolResult::success(contents))
            }
            Ok(super::read_file::ReadOutput::FileUnchanged(stub)) => {
                Ok(CallToolResult::success(vec![Content::text(stub)]))
            }
            Ok(super::read_file::ReadOutput::Pdf(msg)) => {
                Ok(CallToolResult::success(vec![Content::text(msg)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    #[tool(name = "Write", description = "Writes a file to the local filesystem.")]
    fn write(&self, Parameters(req): Parameters<WriteRequest>) -> String {
        match super::write_file::write_file(&self.file_state, &req.file_path, &req.content) {
            Ok(output) => output,
            Err(e) => e,
        }
    }

    #[tool(name = "Edit", description = "Performs exact string replacements in files.")]
    fn edit(&self, Parameters(req): Parameters<EditRequest>) -> String {
        match super::edit_file::edit_file(
            &self.file_state,
            &req.file_path,
            &req.old_string,
            &req.new_string,
            req.replace_all.unwrap_or(false),
        ) {
            Ok(output) => output,
            Err(e) => e,
        }
    }

    #[tool(name = "Bash", description = "Executes a given bash command and returns its output.")]
    async fn bash(&self, Parameters(req): Parameters<BashRequest>) -> Content {
        match super::bash::execute_bash(&self.shell_state, &req.command, req.timeout).await {
            Ok(output) => {
                if output.is_image {
                    if let Some(parsed) = super::bash::parse_data_uri(&output.stdout) {
                        return Content::image(parsed.data, parsed.media_type);
                    }
                }
                let json = serde_json::to_string_pretty(&output).unwrap_or_default();
                Content::text(json)
            }
            Err(e) => Content::text(e),
        }
    }

    #[tool(name = "Glob", description = "Fast file pattern matching tool that works with any codebase size")]
    fn glob(&self, Parameters(req): Parameters<GlobRequest>) -> String {
        match super::glob_search::glob_search(&req.pattern, req.path.as_deref()) {
            Ok(output) => {
                if output.contains("\"truncated\": true") {
                    format!("{output}\n(Results are truncated. Consider using a more specific path or pattern.)")
                } else {
                    output
                }
            }
            Err(e) => e,
        }
    }

    #[tool(name = "Grep", description = "A powerful search tool built on ripgrep")]
    fn grep(&self, Parameters(req): Parameters<GrepRequest>) -> String {
        let input = super::grep_search::GrepSearchInput {
            pattern: req.pattern,
            path: req.path,
            glob: req.glob,
            output_mode: req.output_mode,
            before: req.before,
            after: req.after,
            context_short: req.context_short,
            context: req.context,
            line_numbers: req.line_numbers,
            case_insensitive: req.case_insensitive,
            file_type: req.file_type,
            head_limit: req.head_limit,
            offset: req.offset,
            multiline: req.multiline,
        };
        match super::grep_search::grep_search(&input) {
            Ok(output) => output,
            Err(e) => e,
        }
    }
}
