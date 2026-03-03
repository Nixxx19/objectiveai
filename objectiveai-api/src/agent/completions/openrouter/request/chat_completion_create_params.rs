//! Chat completion request parameters for OpenRouter.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Chat completion request parameters formatted for the OpenRouter API.
///
/// Combines parameters from both the Agent configuration and the
/// incoming request to create a complete request for OpenRouter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatCompletionCreateParams {
    /// Messages for the conversation, including any prefix/suffix from the Agent.
    pub messages: Vec<objectiveai::agent::completions::message::Message>,
    /// Provider preferences merged from request and Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<super::Provider>,

    /// The model identifier from the Agent.
    pub model: String,
    /// Frequency penalty from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,
    /// Logit bias from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<IndexMap<String, i64>>,
    /// Maximum completion tokens from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u64>,
    /// Presence penalty from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,
    /// Stop sequences from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<objectiveai::agent::openrouter::Stop>,
    /// Temperature from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Top-p (nucleus sampling) from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// Maximum tokens (legacy) from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,
    /// Min-p sampling from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_p: Option<f64>,
    /// Reasoning configuration from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<objectiveai::agent::openrouter::Reasoning>,
    /// Repetition penalty from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repetition_penalty: Option<f64>,
    /// Top-a sampling from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_a: Option<f64>,
    /// Top-k sampling from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u64>,
    /// Verbosity setting from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<objectiveai::agent::openrouter::Verbosity>,

    /// Whether to include log probabilities from request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,
    /// Number of top log probabilities to return from request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u64>,
    /// Response format specification (never ToolCall — that variant is extracted into tools).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<super::response_format::ResponseFormat>,
    /// Random seed from request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Tool choice configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<super::tool_choice::ToolChoice>,
    /// Available tools (MCP + invention + response format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<super::Tool>>,
    /// Whether to allow parallel tool calls from request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    /// Prediction hints from request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prediction: Option<super::Prediction>,

    /// Always true for streaming requests.
    pub stream: bool,
    /// Stream options for usage inclusion.
    pub stream_options: super::StreamOptions,
    /// Usage reporting options.
    pub usage: super::Usage,
}

/// Which source a tool originates from, used for conflict resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ToolSource {
    Mcp { url: String },
    Invention,
    ResponseFormat,
}

/// A tool paired with its origin for name-conflict resolution.
struct SourcedTool {
    name: String,
    source: ToolSource,
    tool: super::Tool,
}

impl ChatCompletionCreateParams {
    /// Creates request parameters by fetching MCP tools from live connections,
    /// then delegating to [`new_with_tools`](Self::new_with_tools).
    pub async fn new(
        agent: &objectiveai::agent::openrouter::Agent,
        params: &objectiveai::agent::completions::request::AgentCompletionCreateParams,
        messages: &[objectiveai::agent::completions::message::Message],
        mcp_connections: &[Arc<crate::mcp::Connection>],
        invention_tools: Option<
            &[objectiveai::functions::inventions::InventionTool],
        >,
    ) -> Result<Self, super::super::Error> {
        let mcp_tools = futures::future::try_join_all(
            mcp_connections.iter().map(|c| async {
                c.list_tools().await.map_err(|error| {
                    super::super::Error::Mcp {
                        url: c.url.clone(),
                        error,
                    }
                })
            }),
        )
        .await?;

        Ok(Self::new_with_tools(
            agent,
            params,
            messages,
            mcp_connections,
            &mcp_tools,
            invention_tools,
        ))
    }

    /// Creates request parameters from pre-fetched MCP tool results.
    ///
    /// Merges MCP tools, invention tools, and any response-format tool,
    /// resolves name conflicts with suffixes, and populates the `tools`
    /// and `tool_choice` fields.
    pub(super) fn new_with_tools(
        agent: &objectiveai::agent::openrouter::Agent,
        params: &objectiveai::agent::completions::request::AgentCompletionCreateParams,
        messages: &[objectiveai::agent::completions::message::Message],
        mcp_connections: &[Arc<crate::mcp::Connection>],
        mcp_tools: &[Arc<Vec<crate::mcp::tool::Tool>>],
        invention_tools: Option<
            &[objectiveai::functions::inventions::InventionTool],
        >,
    ) -> Self {
        // --- Step 1: Resolve response_format for this agent ---
        let resolved_response_format = resolve_response_format(params, agent);

        // --- Step 2: Extract ToolCall variant (if any) from response_format ---
        let (openrouter_response_format, response_format_tool) =
            match resolved_response_format {
                Some(objectiveai::agent::completions::request::ResponseFormat::ToolCall {
                    name,
                    description,
                    schema,
                    required,
                }) => (None, Some((name, description, schema, required))),
                Some(rf) => (Some(super::response_format::ResponseFormat::new(&rf)), None),
                None => (None, None),
            };

        // --- Step 3: Build sourced tool list ---
        let mut sourced_tools = Vec::new();

        // MCP tools
        for (connection, tools) in mcp_connections.iter().zip(mcp_tools.iter()) {
            for tool in tools.iter() {
                sourced_tools.push(SourcedTool {
                    name: tool.name.clone(),
                    source: ToolSource::Mcp {
                        url: connection.url.clone(),
                    },
                    tool: super::Tool::new_from_mcp(tool),
                });
            }
        }

        // Invention tools
        if let Some(inv_tools) = invention_tools {
            for tool in inv_tools {
                sourced_tools.push(SourcedTool {
                    name: tool.name.to_string(),
                    source: ToolSource::Invention,
                    tool: super::Tool::new_from_invention(tool),
                });
            }
        }

        // Response format tool
        if let Some((ref name, ref description, ref schema, _)) =
            response_format_tool
        {
            sourced_tools.push(SourcedTool {
                name: name.clone(),
                source: ToolSource::ResponseFormat,
                tool: super::Tool::Function {
                    function: super::FunctionTool {
                        name: name.clone(),
                        description: Some(description.clone()),
                        parameters: Some(schema.clone()),
                        strict: None,
                    },
                },
            });
        }

        // --- Step 4: Resolve name conflicts ---
        let final_tools = resolve_name_conflicts(sourced_tools);

        // --- Step 5: Determine tool_choice ---
        let (tools, tool_choice) = if final_tools.is_empty() {
            (None, None)
        } else if let Some((ref name, _, _, required)) = response_format_tool {
            let choice = if required == Some(true) {
                super::tool_choice::ToolChoice::Function(
                    super::tool_choice::ToolChoiceFunction::Function {
                        function:
                            super::tool_choice::ToolChoiceFunctionFunction {
                                name: name.clone(),
                            },
                    },
                )
            } else {
                super::tool_choice::ToolChoice::Auto
            };
            (Some(final_tools), Some(choice))
        } else {
            (
                Some(final_tools),
                Some(super::tool_choice::ToolChoice::Auto),
            )
        };

        Self {
            // `messages` already includes prefix/suffix from the agent —
            // the caller (UpstreamClient) handles that merging.
            messages: messages.to_vec(),
            provider: super::provider::Provider::new(
                params.provider,
                agent.base.provider.as_ref(),
            ),
            model: agent.base.model.clone(),
            frequency_penalty: agent.base.frequency_penalty,
            logit_bias: agent.base.logit_bias.clone(),
            max_completion_tokens: agent.base.max_completion_tokens,
            presence_penalty: agent.base.presence_penalty,
            stop: agent.base.stop.clone(),
            temperature: agent.base.temperature,
            top_p: agent.base.top_p,
            max_tokens: agent.base.max_tokens,
            min_p: agent.base.min_p,
            reasoning: agent.base.reasoning,
            repetition_penalty: agent.base.repetition_penalty,
            top_a: agent.base.top_a,
            top_k: agent.base.top_k,
            verbosity: agent.base.verbosity,
            logprobs: if let Some(top_logprobs) = agent.base.top_logprobs {
                Some(top_logprobs > 0)
            } else {
                None
            },
            top_logprobs: agent.base.top_logprobs,
            response_format: openrouter_response_format,
            seed: params.seed,
            tool_choice,
            tools,
            parallel_tool_calls: None,
            prediction: None,
            stream: true,
            stream_options: super::StreamOptions {
                include_usage: Some(true),
            },
            usage: super::Usage { include: true },
        }
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Resolves the response format for a specific agent from the request params.
fn resolve_response_format(
    params: &objectiveai::agent::completions::request::AgentCompletionCreateParams,
    agent: &objectiveai::agent::openrouter::Agent,
) -> Option<objectiveai::agent::completions::request::ResponseFormat> {
    match params.response_format.as_ref()? {
        objectiveai::agent::completions::request::ResponseFormatParam::Single(rf) => {
            Some(rf.clone())
        }
        objectiveai::agent::completions::request::ResponseFormatParam::PerAgent(map) => {
            map.get(&agent.id).cloned()
        }
    }
}

/// Resolves name conflicts across MCP, invention, and response-format tools.
///
/// Suffix rules (only applied when there IS a conflict on the same name):
/// - MCP tools always get ` (<url>)` suffix
/// - Invention tools get ` (invention)` suffix only when conflicting with the response format tool
/// - Response format tool never gets a suffix
fn resolve_name_conflicts(sourced_tools: Vec<SourcedTool>) -> Vec<super::Tool> {
    // Group tools by name to find conflicts.
    let mut by_name: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, st) in sourced_tools.iter().enumerate() {
        by_name.entry(st.name.clone()).or_default().push(i);
    }

    let mut result = Vec::with_capacity(sourced_tools.len());
    let mut processed = vec![false; sourced_tools.len()];

    for (name, indices) in &by_name {
        let has_conflict = indices.len() > 1;

        for &i in indices {
            processed[i] = true;
            let st = &sourced_tools[i];

            if !has_conflict {
                // No conflict — use the tool as-is.
                result.push(st.tool.clone());
                continue;
            }

            // Determine whether this tool's name needs a suffix.
            let suffix = match &st.source {
                ToolSource::Mcp { url } => Some(format!(" ({})", url)),
                ToolSource::Invention => {
                    // Invention gets suffix only when conflicting with a response format tool.
                    let conflicts_with_rf = indices.iter().any(|&j| {
                        sourced_tools[j].source == ToolSource::ResponseFormat
                    });
                    if conflicts_with_rf {
                        Some(" (invention)".to_string())
                    } else {
                        None
                    }
                }
                ToolSource::ResponseFormat => None,
            };

            let mut tool = st.tool.clone();
            if let Some(suffix) = suffix {
                match &mut tool {
                    super::Tool::Function { function } => {
                        function.name = format!("{}{}", name, suffix);
                    }
                }
            }
            result.push(tool);
        }
    }

    // Include any tools not in the by_name map (shouldn't happen, but defensive).
    for (i, st) in sourced_tools.iter().enumerate() {
        if !processed[i] {
            result.push(st.tool.clone());
        }
    }

    result
}
