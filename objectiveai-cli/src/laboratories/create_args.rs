use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct CreateArgs {
    /// Docker image to use for the laboratory environment
    #[arg(long)]
    pub docker_image: String,

    /// Builder agent(s) — at least one required.
    /// Format: key=value,key=value (e.g. favorite=name or remote=github,owner=x,repository=y)
    #[arg(long, required = true)]
    pub builder_agent: Vec<crate::agent_ref::AgentRef>,

    /// Benchmark agent reference (e.g. favorite=name or remote=github,owner=x,repository=y)
    #[arg(long)]
    pub benchmark_agent: crate::agent_ref::AgentRef,

    #[command(flatten)]
    pub python: PythonSource,

    #[command(flatten)]
    pub builder_messages: BuilderMessageSource,

    #[command(flatten)]
    pub benchmark_messages: BenchmarkMessageSource,

    #[command(flatten)]
    pub builder_continuation: BuilderContinuationArgs,

    #[command(flatten)]
    pub agent_continuation: AgentContinuationArgs,

    #[command(flatten)]
    pub benchmark_output_schema: BenchmarkOutputSchemaSource,
}

/// Python script source — file or inline.
#[derive(Args)]
#[group(multiple = false)]
pub struct PythonSource {
    /// Inline Python code
    #[arg(long)]
    pub python_inline: Option<String>,

    /// Path to a Python file
    #[arg(long)]
    pub python_file: Option<PathBuf>,
}

/// Messages for builder agents.
#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct BuilderMessageSource {
    /// Builder agent messages as inline JSON array
    #[arg(long)]
    pub builder_messages_inline: Option<String>,

    /// Builder agent messages from inline Python code
    #[arg(long)]
    pub builder_messages_python_inline: Option<String>,

    /// Builder agent messages from a Python file
    #[arg(long)]
    pub builder_messages_python_file: Option<PathBuf>,
}

/// Messages for the benchmark agent.
#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct BenchmarkMessageSource {
    /// Benchmark agent messages as inline JSON array
    #[arg(long)]
    pub benchmark_messages_inline: Option<String>,

    /// Benchmark agent messages from inline Python code
    #[arg(long)]
    pub benchmark_messages_python_inline: Option<String>,

    /// Benchmark agent messages from a Python file
    #[arg(long)]
    pub benchmark_messages_python_file: Option<PathBuf>,
}

/// Continuation for builder agents.
#[derive(Args)]
pub struct BuilderContinuationArgs {
    /// OpenRouter continuation from a previous response (base64-encoded).
    #[arg(long, group = "builder_continuation")]
    pub builder_openrouter_continuation_from_response: Option<String>,

    /// Claude Agent SDK continuation from a previous response (base64-encoded).
    #[arg(long, group = "builder_continuation")]
    pub builder_claude_agent_sdk_continuation_from_response: Option<String>,

    /// Mock continuation from a previous response (base64-encoded).
    #[arg(long, group = "builder_continuation")]
    pub builder_mock_continuation_from_response: Option<String>,

    /// OpenRouter continuation messages as inline JSON.
    #[arg(long, group = "builder_continuation")]
    pub builder_openrouter_continuation_messages_inline: Option<String>,

    /// OpenRouter continuation messages from inline Python code.
    #[arg(long, group = "builder_continuation")]
    pub builder_openrouter_continuation_messages_python_inline: Option<String>,

    /// OpenRouter continuation messages from a Python file.
    #[arg(long, group = "builder_continuation")]
    pub builder_openrouter_continuation_messages_python_file: Option<PathBuf>,

    /// Mock continuation messages as inline JSON.
    #[arg(long, group = "builder_continuation")]
    pub builder_mock_continuation_messages_inline: Option<String>,

    /// Mock continuation messages from inline Python code.
    #[arg(long, group = "builder_continuation")]
    pub builder_mock_continuation_messages_python_inline: Option<String>,

    /// Mock continuation messages from a Python file.
    #[arg(long, group = "builder_continuation")]
    pub builder_mock_continuation_messages_python_file: Option<PathBuf>,

    /// Claude Agent SDK continuation with a session ID.
    #[arg(long, group = "builder_continuation")]
    pub builder_claude_agent_sdk_continuation_session_id: Option<String>,
}

/// Continuation for the benchmark agent.
#[derive(Args)]
pub struct AgentContinuationArgs {
    /// OpenRouter continuation from a previous response (base64-encoded).
    #[arg(long, group = "agent_continuation")]
    pub agent_openrouter_continuation_from_response: Option<String>,

    /// Claude Agent SDK continuation from a previous response (base64-encoded).
    #[arg(long, group = "agent_continuation")]
    pub agent_claude_agent_sdk_continuation_from_response: Option<String>,

    /// Mock continuation from a previous response (base64-encoded).
    #[arg(long, group = "agent_continuation")]
    pub agent_mock_continuation_from_response: Option<String>,

    /// OpenRouter continuation messages as inline JSON.
    #[arg(long, group = "agent_continuation")]
    pub agent_openrouter_continuation_messages_inline: Option<String>,

    /// OpenRouter continuation messages from inline Python code.
    #[arg(long, group = "agent_continuation")]
    pub agent_openrouter_continuation_messages_python_inline: Option<String>,

    /// OpenRouter continuation messages from a Python file.
    #[arg(long, group = "agent_continuation")]
    pub agent_openrouter_continuation_messages_python_file: Option<PathBuf>,

    /// Mock continuation messages as inline JSON.
    #[arg(long, group = "agent_continuation")]
    pub agent_mock_continuation_messages_inline: Option<String>,

    /// Mock continuation messages from inline Python code.
    #[arg(long, group = "agent_continuation")]
    pub agent_mock_continuation_messages_python_inline: Option<String>,

    /// Mock continuation messages from a Python file.
    #[arg(long, group = "agent_continuation")]
    pub agent_mock_continuation_messages_python_file: Option<PathBuf>,

    /// Claude Agent SDK continuation with a session ID.
    #[arg(long, group = "agent_continuation")]
    pub agent_claude_agent_sdk_continuation_session_id: Option<String>,
}

/// Benchmark output schema source (objectiveai-rs InputSchema as JSON).
#[derive(Args)]
#[group(multiple = false)]
pub struct BenchmarkOutputSchemaSource {
    /// Benchmark output schema as inline JSON
    #[arg(long)]
    pub benchmark_output_schema_inline: Option<String>,

    /// Benchmark output schema from inline Python code
    #[arg(long)]
    pub benchmark_output_schema_python_inline: Option<String>,

    /// Benchmark output schema from a Python file
    #[arg(long)]
    pub benchmark_output_schema_python_file: Option<PathBuf>,
}
