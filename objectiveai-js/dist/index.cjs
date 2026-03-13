'use strict';

var z305 = require('zod');

function _interopDefault (e) { return e && e.__esModule ? e : { default: e }; }

var z305__default = /*#__PURE__*/_interopDefault(z305);

// src/agent/claude_agent_sdk/agent.ts
var AgentClaudeAgentSdkEffortSchema = z305.z.union([z305.z.literal("low").describe("Minimal output, concise responses."), z305.z.literal("medium").describe("Balanced output (default, normalized away during preparation)."), z305.z.literal("high").describe("Detailed output with thorough explanations."), z305.z.literal("max").describe("Maximum effort, most detailed output possible.")]).describe("The effort level for model output.\n\nThis setting hints to the model how detailed its responses should be.").meta({ title: "agent.claude_agent_sdk.Effort" });
var AgentClaudeAgentSdkOutputModeSchema = z305.z.union([z305.z.literal("instruction").describe("The model is instructed via the prompt to output a specific key.\n\nThis is the default and most widely supported mode.")]).describe("The method used to constrain LLM output to valid response keys.\n\nIn vector completions, the model must select from a predefined set of\nresponses. This enum controls *how* that constraint is enforced.\n\n**Note:** This setting is only relevant for vector completions and is\ncompletely ignored for agent completions.").meta({ title: "agent.claude_agent_sdk.OutputMode" });
var AgentClaudeAgentSdkUpstreamSchema = z305.z.literal("claude_agent_sdk").describe("Claude Agent SDK upstream marker.").meta({ title: "agent.claude_agent_sdk.Upstream" });
var AgentCompletionsMessageFileSchema = z305.z.object({
  file_data: z305.z.string().nullable().describe("Base64-encoded file data.").optional(),
  file_id: z305.z.string().nullable().describe("The ID of a previously uploaded file.").optional(),
  filename: z305.z.string().nullable().describe("The filename for display purposes.").optional(),
  file_url: z305.z.string().nullable().describe("A URL to fetch the file from.").optional()
}).describe("A file attachment for multimodal input.").meta({ title: "agent.completions.message.File" });
var AgentCompletionsMessageImageUrlDetailSchema = z305.z.union([z305.z.literal("auto").describe("Let the model decide the detail level."), z305.z.literal("low").describe("Low detail mode (faster, less tokens)."), z305.z.literal("high").describe("High detail mode (more accurate, more tokens).")]).describe("Detail level for image processing.").meta({ title: "agent.completions.message.ImageUrlDetail" });

// src/agent/completions/message/imageUrl.ts
var AgentCompletionsMessageImageUrlSchema = z305.z.object({
  url: z305.z.string().describe("The URL of the image (can be a data URL or HTTP URL)."),
  detail: AgentCompletionsMessageImageUrlDetailSchema.nullable().describe("The detail level for image processing.").optional()
}).describe("An image URL for multimodal input.").meta({ title: "agent.completions.message.ImageUrl" });
var AgentCompletionsMessageInputAudioSchema = z305.z.object({
  data: z305.z.string().describe("Base64-encoded audio data."),
  format: z305.z.string().describe('The audio format (e.g., "wav", "mp3").')
}).describe("Audio input for multimodal messages.").meta({ title: "agent.completions.message.InputAudio" });
var AgentCompletionsMessageVideoUrlSchema = z305.z.object({
  url: z305.z.string().describe("The URL of the video.")
}).describe("A video URL for multimodal input.").meta({ title: "agent.completions.message.VideoUrl" });

// src/agent/completions/message/richContentPart.ts
var AgentCompletionsMessageRichContentPartSchema = z305.z.union([z305.z.object({
  text: z305.z.string(),
  type: z305.z.literal("text")
}).describe("Text content."), z305.z.object({
  image_url: AgentCompletionsMessageImageUrlSchema,
  type: z305.z.literal("image_url")
}).describe("An image URL."), z305.z.object({
  input_audio: AgentCompletionsMessageInputAudioSchema,
  type: z305.z.literal("input_audio")
}).describe("Audio input."), z305.z.object({
  video_url: AgentCompletionsMessageVideoUrlSchema,
  type: z305.z.literal("input_video")
}).describe("Video input."), z305.z.object({
  video_url: AgentCompletionsMessageVideoUrlSchema,
  type: z305.z.literal("video_url")
}).describe("A video URL."), z305.z.object({
  file: AgentCompletionsMessageFileSchema,
  type: z305.z.literal("file")
}).describe("A file.")]).describe("A part of rich content.").meta({ title: "agent.completions.message.RichContentPart" });

// src/agent/completions/message/richContent.ts
var AgentCompletionsMessageRichContentSchema = z305.z.union([z305.z.string().describe("Plain text content."), z305.z.array(AgentCompletionsMessageRichContentPartSchema).describe("Multi-part content (text, images, audio, video, files).")]).describe("Rich content for user/assistant messages (supports multimodal input).").meta({ title: "agent.completions.message.RichContent" });
var AgentMcpServerSchema = z305.z.object({
  url: z305.z.string().describe("The URL of the MCP server."),
  authorization: z305.z.boolean().default(false).describe("Whether this MCP server uses authorization.").optional()
}).describe("An MCP server that the agent can connect to.").meta({ title: "agent.McpServer" });

// src/agent/claude_agent_sdk/agent.ts
var AgentClaudeAgentSdkAgentSchema = z305.z.object({
  id: z305.z.string().describe("The deterministic content-addressed ID (22-character base62 string)."),
  upstream: AgentClaudeAgentSdkUpstreamSchema.describe("The upstream provider marker."),
  model: z305.z.string().describe("The upstream language model identifier."),
  output_mode: AgentClaudeAgentSdkOutputModeSchema.describe("The output mode for vector completions. Ignored for agent completions."),
  synthetic_reasoning: z305.z.boolean().nullable().describe("Enable synthetic reasoning for non-reasoning LLMs.\n\n**Vector completions only.** Ignored for agent completions.\n\nWhen enabled, forces the LLM to output a `_think` field before voting,\nsimulating chain-of-thought reasoning. Requires `output_mode` to be\n`ToolCall` (not `Instruction`).").optional(),
  thinking: z305.z.boolean().nullable().describe("Whether thinking/extended thinking is enabled.\n\nDefaults to `true`. Set to `false` to disable.").optional(),
  effort: AgentClaudeAgentSdkEffortSchema.nullable().describe("The effort level for model output.").optional(),
  system_prompt: z305.z.string().nullable().describe("System prompt for the agent.").optional(),
  prefix_content: AgentCompletionsMessageRichContentSchema.nullable().describe("Rich content prepended to the user's prompt.").optional(),
  suffix_content: AgentCompletionsMessageRichContentSchema.nullable().describe("Rich content appended after the user's prompt.").optional(),
  mcp_servers: z305.z.array(AgentMcpServerSchema).nullable().describe("MCP servers the agent can connect to.").optional()
}).describe("A validated Claude Agent SDK Agent with its computed content-addressed ID.").meta({ title: "agent.claude_agent_sdk.Agent" });
var AgentClaudeAgentSdkAgentBaseSchema = z305.z.object({
  upstream: AgentClaudeAgentSdkUpstreamSchema.describe("The upstream provider marker."),
  model: z305.z.string().describe("The upstream language model identifier."),
  output_mode: AgentClaudeAgentSdkOutputModeSchema.describe("The output mode for vector completions. Ignored for agent completions."),
  synthetic_reasoning: z305.z.boolean().nullable().describe("Enable synthetic reasoning for non-reasoning LLMs.\n\n**Vector completions only.** Ignored for agent completions.\n\nWhen enabled, forces the LLM to output a `_think` field before voting,\nsimulating chain-of-thought reasoning. Requires `output_mode` to be\n`ToolCall` (not `Instruction`).").optional(),
  thinking: z305.z.boolean().nullable().describe("Whether thinking/extended thinking is enabled.\n\nDefaults to `true`. Set to `false` to disable.").optional(),
  effort: AgentClaudeAgentSdkEffortSchema.nullable().describe("The effort level for model output.").optional(),
  system_prompt: z305.z.string().nullable().describe("System prompt for the agent.").optional(),
  prefix_content: AgentCompletionsMessageRichContentSchema.nullable().describe("Rich content prepended to the user's prompt.").optional(),
  suffix_content: AgentCompletionsMessageRichContentSchema.nullable().describe("Rich content appended after the user's prompt.").optional(),
  mcp_servers: z305.z.array(AgentMcpServerSchema).nullable().describe("MCP servers the agent can connect to.").optional()
}).describe("The base configuration for a Claude Agent SDK Agent (without computed ID).").meta({ title: "agent.claude_agent_sdk.AgentBase" });
var AgentCompletionsMessageAssistantToolCallFunctionSchema = z305.z.object({
  name: z305.z.string().describe("The name of the function to call."),
  arguments: z305.z.string().describe("The arguments to pass to the function, as a JSON string.")
}).describe("Details of a function call made by the assistant.").meta({ title: "agent.completions.message.AssistantToolCallFunction" });

// src/agent/completions/message/assistantToolCall.ts
var AgentCompletionsMessageAssistantToolCallSchema = z305.z.union([z305.z.object({
  id: z305.z.string().describe("The unique ID of this tool call."),
  function: AgentCompletionsMessageAssistantToolCallFunctionSchema.describe("The function being called."),
  type: z305.z.literal("function")
}).describe("A function call with an ID and function details.")]).describe("A tool call made by the assistant.").meta({ title: "agent.completions.message.AssistantToolCall" });

// src/agent/completions/message/assistantMessage.ts
var AgentCompletionsMessageAssistantMessageSchema = z305.z.object({
  content: AgentCompletionsMessageRichContentSchema.nullable().describe("The message content, if any.").optional(),
  name: z305.z.string().nullable().describe("Optional name for the assistant.").optional(),
  refusal: z305.z.string().nullable().describe("Refusal message if the model declined to respond.").optional(),
  tool_calls: z305.z.array(AgentCompletionsMessageAssistantToolCallSchema).nullable().describe("Tool calls made by the assistant.").optional(),
  reasoning: z305.z.string().nullable().describe("Reasoning content from models that support chain-of-thought.").optional()
}).describe("An assistant message (model's previous response).").meta({ title: "agent.completions.message.AssistantMessage" });
var AgentCompletionsMessageAssistantToolCallExpressionSchema = z305.z.union([z305.z.object({
  id: z305.z.lazy(() => FunctionsExpressionWithExpressionStringSchema).describe("The tool call ID expression."),
  function: z305.z.lazy(() => FunctionsExpressionWithExpressionAgentCompletionsMessageAssistantToolCallFunctionExpressionSchema).describe("The function expression."),
  type: z305.z.literal("function")
}).describe("A function call expression.")]).describe("Expression variant of [`AssistantToolCall`] for dynamic content.").meta({ title: "agent.completions.message.AssistantToolCallExpression" });
var AgentCompletionsMessageAssistantToolCallFunctionExpressionSchema = z305.z.object({
  name: z305.z.lazy(() => FunctionsExpressionWithExpressionStringSchema).describe("The function name expression."),
  arguments: z305.z.lazy(() => FunctionsExpressionWithExpressionStringSchema).describe("The arguments expression.")
}).describe("Expression variant of [`AssistantToolCallFunction`] for dynamic content.").meta({ title: "agent.completions.message.AssistantToolCallFunctionExpression" });
var AgentCompletionsMessageDeveloperMessageExpressionSchema = z305.z.object({
  content: z305.z.lazy(() => FunctionsExpressionWithExpressionAgentCompletionsMessageSimpleContentExpressionSchema).describe("The message content expression."),
  name: z305.z.lazy(() => FunctionsExpressionWithExpressionNullableStringSchema).nullable().describe("Optional name expression.").optional()
}).describe("Expression variant of [`DeveloperMessage`] for dynamic content.").meta({ title: "agent.completions.message.DeveloperMessageExpression" });
var AgentCompletionsMessageSystemMessageExpressionSchema = z305.z.object({
  content: z305.z.lazy(() => FunctionsExpressionWithExpressionAgentCompletionsMessageSimpleContentExpressionSchema).describe("The message content expression."),
  name: z305.z.lazy(() => FunctionsExpressionWithExpressionNullableStringSchema).nullable().describe("Optional name expression.").optional()
}).describe("Expression variant of [`SystemMessage`] for dynamic content.").meta({ title: "agent.completions.message.SystemMessageExpression" });
var AgentCompletionsMessageToolMessageExpressionSchema = z305.z.object({
  content: z305.z.lazy(() => FunctionsExpressionWithExpressionAgentCompletionsMessageRichContentExpressionSchema).describe("The content expression."),
  tool_call_id: z305.z.lazy(() => FunctionsExpressionWithExpressionStringSchema).describe("The tool call ID expression.")
}).describe("Expression variant of [`ToolMessage`] for dynamic content.").meta({ title: "agent.completions.message.ToolMessageExpression" });
var AgentCompletionsMessageUserMessageExpressionSchema = z305.z.object({
  content: z305.z.lazy(() => FunctionsExpressionWithExpressionAgentCompletionsMessageRichContentExpressionSchema).describe("The message content expression."),
  name: z305.z.lazy(() => FunctionsExpressionWithExpressionNullableStringSchema).nullable().describe("Optional name expression.").optional()
}).describe("Expression variant of [`UserMessage`] for dynamic content.").meta({ title: "agent.completions.message.UserMessageExpression" });

// src/agent/completions/message/messageExpression.ts
var AgentCompletionsMessageMessageExpressionSchema = z305.z.union([z305.z.lazy(() => AgentCompletionsMessageDeveloperMessageExpressionSchema).and(z305.z.object({
  role: z305.z.literal("developer")
})), z305.z.lazy(() => AgentCompletionsMessageSystemMessageExpressionSchema).and(z305.z.object({
  role: z305.z.literal("system")
})), z305.z.lazy(() => AgentCompletionsMessageUserMessageExpressionSchema).and(z305.z.object({
  role: z305.z.literal("user")
})), z305.z.lazy(() => AgentCompletionsMessageAssistantMessageExpressionSchema).and(z305.z.object({
  role: z305.z.literal("assistant")
})), z305.z.lazy(() => AgentCompletionsMessageToolMessageExpressionSchema).and(z305.z.object({
  role: z305.z.literal("tool")
}))]).describe("A message with expressions for dynamic content.\n\nThis is the expression variant of [`Message`] used in function definitions\nwhere message content can be computed from the function input at runtime.\nSupports both JMESPath and Starlark expressions.").meta({ title: "agent.completions.message.MessageExpression" });
var AgentCompletionsMessageRichContentExpressionSchema = z305.z.union([z305.z.string().describe("Plain text content."), z305.z.array(z305.z.lazy(() => FunctionsExpressionWithExpressionAgentCompletionsMessageRichContentPartExpressionSchema)).describe("Multi-part content expressions.")]).describe("Expression variant of [`RichContent`] for dynamic content.").meta({ title: "agent.completions.message.RichContentExpression" });
var AgentCompletionsMessageRichContentPartExpressionSchema = z305.z.union([z305.z.object({
  text: z305.z.lazy(() => FunctionsExpressionWithExpressionStringSchema),
  type: z305.z.literal("text")
}), z305.z.object({
  image_url: z305.z.lazy(() => FunctionsExpressionWithExpressionAgentCompletionsMessageImageUrlSchema),
  type: z305.z.literal("image_url")
}), z305.z.object({
  input_audio: z305.z.lazy(() => FunctionsExpressionWithExpressionAgentCompletionsMessageInputAudioSchema),
  type: z305.z.literal("input_audio")
}), z305.z.object({
  video_url: z305.z.lazy(() => FunctionsExpressionWithExpressionAgentCompletionsMessageVideoUrlSchema),
  type: z305.z.literal("input_video")
}), z305.z.object({
  video_url: z305.z.lazy(() => FunctionsExpressionWithExpressionAgentCompletionsMessageVideoUrlSchema),
  type: z305.z.literal("video_url")
}), z305.z.object({
  file: z305.z.lazy(() => FunctionsExpressionWithExpressionAgentCompletionsMessageFileSchema),
  type: z305.z.literal("file")
})]).describe("Expression variant of [`RichContentPart`] for dynamic content.").meta({ title: "agent.completions.message.RichContentPartExpression" });
var AgentCompletionsMessageSimpleContentExpressionSchema = z305.z.union([z305.z.string().describe("Plain text content."), z305.z.array(z305.z.lazy(() => FunctionsExpressionWithExpressionAgentCompletionsMessageSimpleContentPartExpressionSchema)).describe("Multi-part text content expressions.")]).describe("Expression variant of [`SimpleContent`] for dynamic content.").meta({ title: "agent.completions.message.SimpleContentExpression" });
var AgentCompletionsMessageSimpleContentPartExpressionSchema = z305.z.union([z305.z.object({
  text: z305.z.lazy(() => FunctionsExpressionWithExpressionStringSchema).describe("The text expression."),
  type: z305.z.literal("text")
}).describe("A text part expression.")]).describe("Expression variant of [`SimpleContentPart`] for dynamic content.").meta({ title: "agent.completions.message.SimpleContentPartExpression" });
var FunctionsExpressionSpecialSchema = z305.z.union([z305.z.literal("input").describe("Returns the params input as-is."), z305.z.literal("output").describe("Returns the params output as-is."), z305.z.literal("task_output_l1_normalized").describe("L1-normalizes the output. Scalar/Err pass through.\nVector: L1 normalize. Vectors: L1 normalize each."), z305.z.literal("task_output_weighted_sum").describe("Weighted sum of the output. Vector \u2192 Scalar. Vectors \u2192 Vector."), z305.z.literal("input_items_output_length").describe("Returns the length of input['items'] as u64"), z305.z.literal("input_items_optional_context_split").describe("Splits an input containing items and optionally context into multiple inputs"), z305.z.literal("input_items_optional_context_merge").describe("Merges multiple inputs containing items and optionally context into a single input")]).describe("Predefined expression behaviors that require no user-authored code.").meta({ title: "functions.expression.Special" });

// src/functions/expression/expression.ts
var FunctionsExpressionExpressionSchema = z305.z.union([z305.z.object({
  $jmespath: z305.z.string()
}).strict().describe("A JMESPath expression."), z305.z.object({
  $starlark: z305.z.string()
}).strict().describe("A Starlark expression."), z305.z.object({
  $special: FunctionsExpressionSpecialSchema
}).strict().describe("A predefined special expression variant.")]).describe('An expression that can be either JMESPath or Starlark.\n\nSerializes as `{"$jmespath": "..."}` or `{"$starlark": "..."}` in JSON.\n\n# Examples\n\nJMESPath:\n```json\n{"$jmespath": "input.items[0].name"}\n```\n\nStarlark:\n```json\n{"$starlark": "input[\'items\'][0][\'name\']"}\n```').meta({ title: "functions.expression.Expression" });
var FunctionsExpressionInputValueExpressionSchema = z305.z.union([AgentCompletionsMessageRichContentPartSchema.describe("Rich content (image, audio, video, file)."), z305.z.record(z305.z.string(), z305.z.lazy(() => FunctionsExpressionWithExpressionFunctionsExpressionInputValueExpressionSchema)).describe("An object with values that may be expressions."), z305.z.array(z305.z.lazy(() => FunctionsExpressionWithExpressionFunctionsExpressionInputValueExpressionSchema)).describe("An array with elements that may be expressions."), z305.z.string().describe("A string value."), z305.z.number().int().meta({ format: "int64" }).describe("An integer value."), z305.z.number().meta({ format: "double" }).describe("A floating-point number."), z305.z.boolean().describe("A boolean value.")]).describe("An input value that may contain expressions (pre-compilation).\n\nSimilar to [`InputValue`] but object values and array elements can be\nexpressions (JMESPath or Starlark) that are evaluated during compilation.").meta({ title: "functions.expression.InputValueExpression" });

// src/functions/expression/withExpression.ts
var FunctionsExpressionWithExpressionAgentCompletionsMessageAssistantToolCallExpressionSchema = z305.z.union([FunctionsExpressionExpressionSchema.describe("An expression (JMESPath or Starlark) to evaluate."), z305.z.lazy(() => AgentCompletionsMessageAssistantToolCallExpressionSchema).describe("A literal value.")]).describe('A value that can be either a literal or an expression.\n\nThis allows Function definitions to mix static values with dynamic\nexpressions. During compilation, expressions are evaluated while\nliteral values pass through unchanged.\n\n# Example\n\nLiteral value:\n```json\n"hello world"\n```\n\nJMESPath expression:\n```json\n{"$jmespath": "input.greeting"}\n```\n\nStarlark expression:\n```json\n{"$starlark": "input[\'greeting\']"}\n```').meta({ title: "functions.expression.WithExpression.agent.completions.message.AssistantToolCallExpression" });
var FunctionsExpressionWithExpressionAgentCompletionsMessageAssistantToolCallFunctionExpressionSchema = z305.z.union([FunctionsExpressionExpressionSchema.describe("An expression (JMESPath or Starlark) to evaluate."), z305.z.lazy(() => AgentCompletionsMessageAssistantToolCallFunctionExpressionSchema).describe("A literal value.")]).describe('A value that can be either a literal or an expression.\n\nThis allows Function definitions to mix static values with dynamic\nexpressions. During compilation, expressions are evaluated while\nliteral values pass through unchanged.\n\n# Example\n\nLiteral value:\n```json\n"hello world"\n```\n\nJMESPath expression:\n```json\n{"$jmespath": "input.greeting"}\n```\n\nStarlark expression:\n```json\n{"$starlark": "input[\'greeting\']"}\n```').meta({ title: "functions.expression.WithExpression.agent.completions.message.AssistantToolCallFunctionExpression" });
var FunctionsExpressionWithExpressionAgentCompletionsMessageFileSchema = z305.z.union([FunctionsExpressionExpressionSchema.describe("An expression (JMESPath or Starlark) to evaluate."), AgentCompletionsMessageFileSchema.describe("A literal value.")]).describe('A value that can be either a literal or an expression.\n\nThis allows Function definitions to mix static values with dynamic\nexpressions. During compilation, expressions are evaluated while\nliteral values pass through unchanged.\n\n# Example\n\nLiteral value:\n```json\n"hello world"\n```\n\nJMESPath expression:\n```json\n{"$jmespath": "input.greeting"}\n```\n\nStarlark expression:\n```json\n{"$starlark": "input[\'greeting\']"}\n```').meta({ title: "functions.expression.WithExpression.agent.completions.message.File" });
var FunctionsExpressionWithExpressionAgentCompletionsMessageImageUrlSchema = z305.z.union([FunctionsExpressionExpressionSchema.describe("An expression (JMESPath or Starlark) to evaluate."), AgentCompletionsMessageImageUrlSchema.describe("A literal value.")]).describe('A value that can be either a literal or an expression.\n\nThis allows Function definitions to mix static values with dynamic\nexpressions. During compilation, expressions are evaluated while\nliteral values pass through unchanged.\n\n# Example\n\nLiteral value:\n```json\n"hello world"\n```\n\nJMESPath expression:\n```json\n{"$jmespath": "input.greeting"}\n```\n\nStarlark expression:\n```json\n{"$starlark": "input[\'greeting\']"}\n```').meta({ title: "functions.expression.WithExpression.agent.completions.message.ImageUrl" });
var FunctionsExpressionWithExpressionAgentCompletionsMessageInputAudioSchema = z305.z.union([FunctionsExpressionExpressionSchema.describe("An expression (JMESPath or Starlark) to evaluate."), AgentCompletionsMessageInputAudioSchema.describe("A literal value.")]).describe('A value that can be either a literal or an expression.\n\nThis allows Function definitions to mix static values with dynamic\nexpressions. During compilation, expressions are evaluated while\nliteral values pass through unchanged.\n\n# Example\n\nLiteral value:\n```json\n"hello world"\n```\n\nJMESPath expression:\n```json\n{"$jmespath": "input.greeting"}\n```\n\nStarlark expression:\n```json\n{"$starlark": "input[\'greeting\']"}\n```').meta({ title: "functions.expression.WithExpression.agent.completions.message.InputAudio" });
var FunctionsExpressionWithExpressionAgentCompletionsMessageMessageExpressionSchema = z305.z.union([FunctionsExpressionExpressionSchema.describe("An expression (JMESPath or Starlark) to evaluate."), z305.z.lazy(() => AgentCompletionsMessageMessageExpressionSchema).describe("A literal value.")]).describe('A value that can be either a literal or an expression.\n\nThis allows Function definitions to mix static values with dynamic\nexpressions. During compilation, expressions are evaluated while\nliteral values pass through unchanged.\n\n# Example\n\nLiteral value:\n```json\n"hello world"\n```\n\nJMESPath expression:\n```json\n{"$jmespath": "input.greeting"}\n```\n\nStarlark expression:\n```json\n{"$starlark": "input[\'greeting\']"}\n```').meta({ title: "functions.expression.WithExpression.agent.completions.message.MessageExpression" });
var FunctionsExpressionWithExpressionAgentCompletionsMessageRichContentExpressionSchema = z305.z.union([FunctionsExpressionExpressionSchema.describe("An expression (JMESPath or Starlark) to evaluate."), z305.z.lazy(() => AgentCompletionsMessageRichContentExpressionSchema).describe("A literal value.")]).describe('A value that can be either a literal or an expression.\n\nThis allows Function definitions to mix static values with dynamic\nexpressions. During compilation, expressions are evaluated while\nliteral values pass through unchanged.\n\n# Example\n\nLiteral value:\n```json\n"hello world"\n```\n\nJMESPath expression:\n```json\n{"$jmespath": "input.greeting"}\n```\n\nStarlark expression:\n```json\n{"$starlark": "input[\'greeting\']"}\n```').meta({ title: "functions.expression.WithExpression.agent.completions.message.RichContentExpression" });
var FunctionsExpressionWithExpressionAgentCompletionsMessageRichContentPartExpressionSchema = z305.z.union([FunctionsExpressionExpressionSchema.describe("An expression (JMESPath or Starlark) to evaluate."), z305.z.lazy(() => AgentCompletionsMessageRichContentPartExpressionSchema).describe("A literal value.")]).describe('A value that can be either a literal or an expression.\n\nThis allows Function definitions to mix static values with dynamic\nexpressions. During compilation, expressions are evaluated while\nliteral values pass through unchanged.\n\n# Example\n\nLiteral value:\n```json\n"hello world"\n```\n\nJMESPath expression:\n```json\n{"$jmespath": "input.greeting"}\n```\n\nStarlark expression:\n```json\n{"$starlark": "input[\'greeting\']"}\n```').meta({ title: "functions.expression.WithExpression.agent.completions.message.RichContentPartExpression" });
var FunctionsExpressionWithExpressionAgentCompletionsMessageSimpleContentExpressionSchema = z305.z.union([FunctionsExpressionExpressionSchema.describe("An expression (JMESPath or Starlark) to evaluate."), z305.z.lazy(() => AgentCompletionsMessageSimpleContentExpressionSchema).describe("A literal value.")]).describe('A value that can be either a literal or an expression.\n\nThis allows Function definitions to mix static values with dynamic\nexpressions. During compilation, expressions are evaluated while\nliteral values pass through unchanged.\n\n# Example\n\nLiteral value:\n```json\n"hello world"\n```\n\nJMESPath expression:\n```json\n{"$jmespath": "input.greeting"}\n```\n\nStarlark expression:\n```json\n{"$starlark": "input[\'greeting\']"}\n```').meta({ title: "functions.expression.WithExpression.agent.completions.message.SimpleContentExpression" });
var FunctionsExpressionWithExpressionAgentCompletionsMessageSimpleContentPartExpressionSchema = z305.z.union([FunctionsExpressionExpressionSchema.describe("An expression (JMESPath or Starlark) to evaluate."), z305.z.lazy(() => AgentCompletionsMessageSimpleContentPartExpressionSchema).describe("A literal value.")]).describe('A value that can be either a literal or an expression.\n\nThis allows Function definitions to mix static values with dynamic\nexpressions. During compilation, expressions are evaluated while\nliteral values pass through unchanged.\n\n# Example\n\nLiteral value:\n```json\n"hello world"\n```\n\nJMESPath expression:\n```json\n{"$jmespath": "input.greeting"}\n```\n\nStarlark expression:\n```json\n{"$starlark": "input[\'greeting\']"}\n```').meta({ title: "functions.expression.WithExpression.agent.completions.message.SimpleContentPartExpression" });
var FunctionsExpressionWithExpressionAgentCompletionsMessageVideoUrlSchema = z305.z.union([FunctionsExpressionExpressionSchema.describe("An expression (JMESPath or Starlark) to evaluate."), AgentCompletionsMessageVideoUrlSchema.describe("A literal value.")]).describe('A value that can be either a literal or an expression.\n\nThis allows Function definitions to mix static values with dynamic\nexpressions. During compilation, expressions are evaluated while\nliteral values pass through unchanged.\n\n# Example\n\nLiteral value:\n```json\n"hello world"\n```\n\nJMESPath expression:\n```json\n{"$jmespath": "input.greeting"}\n```\n\nStarlark expression:\n```json\n{"$starlark": "input[\'greeting\']"}\n```').meta({ title: "functions.expression.WithExpression.agent.completions.message.VideoUrl" });
var FunctionsExpressionWithExpressionArrayOfFunctionsExpressionWithExpressionAgentCompletionsMessageMessageExpressionSchema = z305.z.union([FunctionsExpressionExpressionSchema.describe("An expression (JMESPath or Starlark) to evaluate."), z305.z.array(FunctionsExpressionWithExpressionAgentCompletionsMessageMessageExpressionSchema).describe("A literal value.")]).describe('A value that can be either a literal or an expression.\n\nThis allows Function definitions to mix static values with dynamic\nexpressions. During compilation, expressions are evaluated while\nliteral values pass through unchanged.\n\n# Example\n\nLiteral value:\n```json\n"hello world"\n```\n\nJMESPath expression:\n```json\n{"$jmespath": "input.greeting"}\n```\n\nStarlark expression:\n```json\n{"$starlark": "input[\'greeting\']"}\n```').meta({ title: "functions.expression.WithExpression.Array_of_functions.expression.WithExpression.agent.completions.message.MessageExpression" });
var FunctionsExpressionWithExpressionArrayOfFunctionsExpressionWithExpressionAgentCompletionsMessageRichContentExpressionSchema = z305.z.union([FunctionsExpressionExpressionSchema.describe("An expression (JMESPath or Starlark) to evaluate."), z305.z.array(FunctionsExpressionWithExpressionAgentCompletionsMessageRichContentExpressionSchema).describe("A literal value.")]).describe('A value that can be either a literal or an expression.\n\nThis allows Function definitions to mix static values with dynamic\nexpressions. During compilation, expressions are evaluated while\nliteral values pass through unchanged.\n\n# Example\n\nLiteral value:\n```json\n"hello world"\n```\n\nJMESPath expression:\n```json\n{"$jmespath": "input.greeting"}\n```\n\nStarlark expression:\n```json\n{"$starlark": "input[\'greeting\']"}\n```').meta({ title: "functions.expression.WithExpression.Array_of_functions.expression.WithExpression.agent.completions.message.RichContentExpression" });
var FunctionsExpressionWithExpressionFunctionsExpressionInputValueExpressionSchema = z305.z.union([FunctionsExpressionExpressionSchema.describe("An expression (JMESPath or Starlark) to evaluate."), z305.z.lazy(() => FunctionsExpressionInputValueExpressionSchema).describe("A literal value.")]).describe('A value that can be either a literal or an expression.\n\nThis allows Function definitions to mix static values with dynamic\nexpressions. During compilation, expressions are evaluated while\nliteral values pass through unchanged.\n\n# Example\n\nLiteral value:\n```json\n"hello world"\n```\n\nJMESPath expression:\n```json\n{"$jmespath": "input.greeting"}\n```\n\nStarlark expression:\n```json\n{"$starlark": "input[\'greeting\']"}\n```').meta({ title: "functions.expression.WithExpression.functions.expression.InputValueExpression" });
var FunctionsExpressionWithExpressionNullableAgentCompletionsMessageRichContentExpressionSchema = z305.z.union([FunctionsExpressionExpressionSchema.describe("An expression (JMESPath or Starlark) to evaluate."), z305.z.lazy(() => AgentCompletionsMessageRichContentExpressionSchema).nullable().describe("A literal value.")]).describe('A value that can be either a literal or an expression.\n\nThis allows Function definitions to mix static values with dynamic\nexpressions. During compilation, expressions are evaluated while\nliteral values pass through unchanged.\n\n# Example\n\nLiteral value:\n```json\n"hello world"\n```\n\nJMESPath expression:\n```json\n{"$jmespath": "input.greeting"}\n```\n\nStarlark expression:\n```json\n{"$starlark": "input[\'greeting\']"}\n```').meta({ title: "functions.expression.WithExpression.Nullable_agent.completions.message.RichContentExpression" });
var FunctionsExpressionWithExpressionNullableArrayOfFunctionsExpressionWithExpressionAgentCompletionsMessageAssistantToolCallExpressionSchema = z305.z.union([FunctionsExpressionExpressionSchema.describe("An expression (JMESPath or Starlark) to evaluate."), z305.z.array(FunctionsExpressionWithExpressionAgentCompletionsMessageAssistantToolCallExpressionSchema).nullable().describe("A literal value.")]).describe('A value that can be either a literal or an expression.\n\nThis allows Function definitions to mix static values with dynamic\nexpressions. During compilation, expressions are evaluated while\nliteral values pass through unchanged.\n\n# Example\n\nLiteral value:\n```json\n"hello world"\n```\n\nJMESPath expression:\n```json\n{"$jmespath": "input.greeting"}\n```\n\nStarlark expression:\n```json\n{"$starlark": "input[\'greeting\']"}\n```').meta({ title: "functions.expression.WithExpression.Nullable_Array_of_functions.expression.WithExpression.agent.completions.message.AssistantToolCallExpression" });
var FunctionsExpressionWithExpressionNullableStringSchema = z305.z.union([FunctionsExpressionExpressionSchema.describe("An expression (JMESPath or Starlark) to evaluate."), z305.z.string().nullable().describe("A literal value.")]).describe('A value that can be either a literal or an expression.\n\nThis allows Function definitions to mix static values with dynamic\nexpressions. During compilation, expressions are evaluated while\nliteral values pass through unchanged.\n\n# Example\n\nLiteral value:\n```json\n"hello world"\n```\n\nJMESPath expression:\n```json\n{"$jmespath": "input.greeting"}\n```\n\nStarlark expression:\n```json\n{"$starlark": "input[\'greeting\']"}\n```').meta({ title: "functions.expression.WithExpression.Nullable_string" });
var FunctionsExpressionWithExpressionStringSchema = z305.z.union([FunctionsExpressionExpressionSchema.describe("An expression (JMESPath or Starlark) to evaluate."), z305.z.string().describe("A literal value.")]).describe('A value that can be either a literal or an expression.\n\nThis allows Function definitions to mix static values with dynamic\nexpressions. During compilation, expressions are evaluated while\nliteral values pass through unchanged.\n\n# Example\n\nLiteral value:\n```json\n"hello world"\n```\n\nJMESPath expression:\n```json\n{"$jmespath": "input.greeting"}\n```\n\nStarlark expression:\n```json\n{"$starlark": "input[\'greeting\']"}\n```').meta({ title: "functions.expression.WithExpression.string" });

// src/agent/completions/message/assistantMessageExpression.ts
var AgentCompletionsMessageAssistantMessageExpressionSchema = z305.z.object({
  content: z305.z.lazy(() => FunctionsExpressionWithExpressionNullableAgentCompletionsMessageRichContentExpressionSchema).nullable().describe("The content expression.").optional(),
  name: z305.z.lazy(() => FunctionsExpressionWithExpressionNullableStringSchema).nullable().optional(),
  refusal: z305.z.lazy(() => FunctionsExpressionWithExpressionNullableStringSchema).nullable().optional(),
  tool_calls: z305.z.lazy(() => FunctionsExpressionWithExpressionNullableArrayOfFunctionsExpressionWithExpressionAgentCompletionsMessageAssistantToolCallExpressionSchema).nullable().optional(),
  reasoning: z305.z.lazy(() => FunctionsExpressionWithExpressionNullableStringSchema).nullable().optional()
}).describe("Expression variant of [`AssistantMessage`] for dynamic content.").meta({ title: "agent.completions.message.AssistantMessageExpression" });
var AgentCompletionsMessageAssistantToolCallFunctionDeltaSchema = z305.z.object({
  name: z305.z.string().nullable().describe("The function name (only present in the first delta).").optional(),
  arguments: z305.z.string().nullable().describe("The arguments being streamed (accumulated across deltas).").optional()
}).describe("Function call details in a streaming tool call.").meta({ title: "agent.completions.message.AssistantToolCallFunctionDelta" });
var AgentCompletionsMessageAssistantToolCallTypeSchema = z305.z.union([z305.z.literal("function").describe("A function call.")]).describe("The type of tool call.").meta({ title: "agent.completions.message.AssistantToolCallType" });

// src/agent/completions/message/assistantToolCallDelta.ts
var AgentCompletionsMessageAssistantToolCallDeltaSchema = z305.z.object({
  index: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("The index of this tool call."),
  type: AgentCompletionsMessageAssistantToolCallTypeSchema.nullable().describe('The type of tool call (always "function").').optional(),
  id: z305.z.string().nullable().describe("The unique ID of this tool call.").optional(),
  function: AgentCompletionsMessageAssistantToolCallFunctionDeltaSchema.nullable().describe("The function call details.").optional()
}).describe("A tool call delta in a streaming response.").meta({ title: "agent.completions.message.AssistantToolCallDelta" });
var AgentCompletionsMessageSimpleContentPartSchema = z305.z.union([z305.z.object({
  text: z305.z.string().describe("The text content."),
  type: z305.z.literal("text")
}).describe("A text part.")]).describe("A part of simple text content.").meta({ title: "agent.completions.message.SimpleContentPart" });

// src/agent/completions/message/simpleContent.ts
var AgentCompletionsMessageSimpleContentSchema = z305.z.union([z305.z.string().describe("Plain text content."), z305.z.array(AgentCompletionsMessageSimpleContentPartSchema).describe("Multi-part text content.")]).describe("Simple text content for system/developer messages.").meta({ title: "agent.completions.message.SimpleContent" });

// src/agent/completions/message/developerMessage.ts
var AgentCompletionsMessageDeveloperMessageSchema = z305.z.object({
  content: AgentCompletionsMessageSimpleContentSchema.describe("The message content."),
  name: z305.z.string().nullable().describe("Optional name for the message author.").optional()
}).describe("A developer message.").meta({ title: "agent.completions.message.DeveloperMessage" });
var AgentCompletionsMessageSystemMessageSchema = z305.z.object({
  content: AgentCompletionsMessageSimpleContentSchema.describe("The message content."),
  name: z305.z.string().nullable().describe("Optional name for the message author.").optional()
}).describe("A system message setting context or instructions.").meta({ title: "agent.completions.message.SystemMessage" });
var AgentCompletionsMessageToolMessageSchema = z305.z.object({
  content: AgentCompletionsMessageRichContentSchema.describe("The content of the tool response."),
  tool_call_id: z305.z.string().describe("The ID of the tool call this message responds to.")
}).describe("A tool message containing the result of a tool call.").meta({ title: "agent.completions.message.ToolMessage" });
var AgentCompletionsMessageUserMessageSchema = z305.z.object({
  content: AgentCompletionsMessageRichContentSchema.describe("The message content (supports text, images, audio, video, files)."),
  name: z305.z.string().nullable().describe("Optional name for the user.").optional()
}).describe("A user message from the end user.").meta({ title: "agent.completions.message.UserMessage" });

// src/agent/completions/message/message.ts
var AgentCompletionsMessageMessageSchema = z305.z.union([AgentCompletionsMessageDeveloperMessageSchema.extend({
  role: z305.z.literal("developer")
}).describe("A developer message (similar to system, but from the developer)."), AgentCompletionsMessageSystemMessageSchema.extend({
  role: z305.z.literal("system")
}).describe("A system message setting context or instructions."), AgentCompletionsMessageUserMessageSchema.extend({
  role: z305.z.literal("user")
}).describe("A user message from the end user."), AgentCompletionsMessageAssistantMessageSchema.extend({
  role: z305.z.literal("assistant")
}).describe("An assistant message (model's previous response)."), AgentCompletionsMessageToolMessageSchema.extend({
  role: z305.z.literal("tool")
}).describe("A tool message containing the result of a tool call.")]).describe("A message in the conversation.").meta({ title: "agent.completions.message.Message" });

// src/agent/completions/message/richContentMerged.ts
function agentCompletionsMessageRichContentMerged(a, b) {
  const aIsString = typeof a === "string";
  const bIsString = typeof b === "string";
  if (aIsString && bIsString) {
    if (b === "") return [a, false];
    return [a + b, true];
  }
  if (aIsString && !bIsString) {
    const parts = [
      { text: a, type: "text" },
      ...b
    ];
    return [parts, true];
  }
  if (!aIsString && bIsString) {
    if (b === "") return [a, false];
    return [
      [...a, { text: b, type: "text" }],
      true
    ];
  }
  const bParts = b;
  if (bParts.length === 0) return [a, false];
  return [
    [...a, ...bParts],
    true
  ];
}

// src/merge.ts
function merge(a, b, combine) {
  if (a !== null && a !== void 0 && b !== null && b !== void 0) {
    return combine ? combine(a, b) : [a, false];
  } else if (a !== null && a !== void 0) {
    return [a, false];
  } else if (b !== null && b !== void 0) {
    return [b, true];
  } else if (a === null || b === null) {
    return [null, false];
  } else {
    return [void 0, false];
  }
}
function mergedString(a, b) {
  return b === "" ? [a, false] : [a + b, true];
}
function mergedNumberArray(a, b) {
  if (a.length === b.length) {
    for (let i = 0; i < a.length; i++) {
      if (a[i] !== b[i]) return [b, true];
    }
    return [a, false];
  }
  return [b, true];
}

// src/agent/completions/message/assistantToolCallFunctionDeltaMerged.ts
function agentCompletionsMessageAssistantToolCallFunctionDeltaMerged(a, b) {
  let changed = false;
  let name = a.name;
  if (a.name == null && b.name != null) {
    name = b.name;
    changed = true;
  }
  let args = a.arguments;
  if (a.arguments != null && b.arguments != null) {
    const [merged, c] = mergedString(a.arguments, b.arguments);
    args = merged;
    if (c) changed = true;
  } else if (b.arguments != null) {
    args = b.arguments;
    changed = true;
  }
  if (!changed) return [a, false];
  return [{
    ...name != null ? { name } : {},
    ...args != null ? { arguments: args } : {}
  }, true];
}

// src/agent/completions/message/assistantToolCallDeltaMerged.ts
function agentCompletionsMessageAssistantToolCallDeltaMerged(a, b) {
  let changed = false;
  let type = a.type;
  if (a.type == null && b.type != null) {
    type = b.type;
    changed = true;
  }
  let id = a.id;
  if (a.id == null && b.id != null) {
    id = b.id;
    changed = true;
  }
  const [fn, fnChanged] = merge(
    a.function ?? void 0,
    b.function ?? void 0,
    agentCompletionsMessageAssistantToolCallFunctionDeltaMerged
  );
  if (fnChanged) changed = true;
  if (!changed) return [a, false];
  return [{
    index: a.index,
    ...type != null ? { type } : {},
    ...id != null ? { id } : {},
    ...fn != null ? { function: fn } : {}
  }, true];
}
function agentCompletionsMessageAssistantToolCallDeltaMergedList(a, b) {
  let changed = false;
  const result = [...a];
  for (const bItem of b) {
    const existingIdx = result.findIndex((x) => x.index === bItem.index);
    if (existingIdx !== -1) {
      const [merged, c] = agentCompletionsMessageAssistantToolCallDeltaMerged(result[existingIdx], bItem);
      if (c) {
        result[existingIdx] = merged;
        changed = true;
      }
    } else {
      result.push(bItem);
      changed = true;
    }
  }
  if (!changed) return [a, false];
  return [result, true];
}
var AgentMockOutputModeSchema = z305.z.union([z305.z.literal("instruction").describe("The model is instructed via the prompt to output a specific key.\n\nThis is the default and most widely supported mode."), z305.z.literal("json_schema").describe("A JSON schema response format is used with an enum of possible keys.\n\nRequires model support for structured JSON output."), z305.z.literal("tool_call").describe("A forced tool call with an argument schema containing possible keys.\n\nRequires model support for tool/function calling.")]).describe("The method used to constrain LLM output to valid response keys.\n\nIn vector completions, the model must select from a predefined set of\nresponses. This enum controls *how* that constraint is enforced.\n\n**Note:** This setting is only relevant for vector completions and is\ncompletely ignored for agent completions.").meta({ title: "agent.mock.OutputMode" });
var AgentMockUpstreamSchema = z305.z.literal("mock").describe("Mock upstream marker.").meta({ title: "agent.mock.Upstream" });

// src/agent/mock/agentBase.ts
var AgentMockAgentBaseSchema = z305.z.object({
  upstream: AgentMockUpstreamSchema.describe("The upstream provider marker."),
  output_mode: AgentMockOutputModeSchema.describe("The output mode for vector completions. Ignored for agent completions."),
  top_logprobs: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Number of top log probabilities to return (2-20).\n\n**Vector completions only.** Ignored for agent completions.").optional(),
  error: z305.z.boolean().nullable().describe("If true, the mock client will return an error instead of a response.").optional(),
  invention: z305.z.boolean().nullable().describe("If true, this mock agent supports invention tool calling.\nIncompatible with output modes other than `instruction`.").optional()
}).describe("The base configuration for a Mock Agent (without computed ID).").meta({ title: "agent.mock.AgentBase" });
var AgentOpenrouterOutputModeSchema = z305.z.union([z305.z.literal("instruction").describe("The model is instructed via the prompt to output a specific key.\n\nThis is the default and most widely supported mode."), z305.z.literal("json_schema").describe("A JSON schema response format is used with an enum of possible keys.\n\nRequires model support for structured JSON output."), z305.z.literal("tool_call").describe("A forced tool call with an argument schema containing possible keys.\n\nRequires model support for tool/function calling.")]).describe("The method used to constrain LLM output to valid response keys.\n\nIn vector completions, the model must select from a predefined set of\nresponses. This enum controls *how* that constraint is enforced.\n\n**Note:** This setting is only relevant for vector completions and is\ncompletely ignored for agent completions.").meta({ title: "agent.openrouter.OutputMode" });
var AgentOpenrouterProviderQuantizationSchema = z305.z.union([z305.z.literal("int4").describe("4-bit integer quantization."), z305.z.literal("int8").describe("8-bit integer quantization."), z305.z.literal("fp4").describe("4-bit floating point quantization."), z305.z.literal("fp6").describe("6-bit floating point quantization."), z305.z.literal("fp8").describe("8-bit floating point quantization."), z305.z.literal("fp16").describe("16-bit floating point (half precision)."), z305.z.literal("bf16").describe("16-bit brain floating point."), z305.z.literal("fp32").describe("32-bit floating point (full precision)."), z305.z.literal("unknown").describe("Unknown quantization level.")]).describe("Model quantization levels for provider filtering.\n\nQuantization reduces model precision to decrease memory usage and\nincrease inference speed, potentially at the cost of output quality.").meta({ title: "agent.openrouter.ProviderQuantization" });

// src/agent/openrouter/provider.ts
var AgentOpenrouterProviderSchema = z305.z.object({
  allow_fallbacks: z305.z.boolean().nullable().describe("Whether to allow fallback to other providers if preferred ones fail.\nDefaults to `true`.").optional(),
  require_parameters: z305.z.boolean().nullable().describe("Whether to require that the provider supports all request parameters.\nDefaults to `false`.").optional(),
  order: z305.z.array(z305.z.string()).nullable().describe("Preferred provider order. Earlier providers are tried first.").optional(),
  only: z305.z.array(z305.z.string()).nullable().describe("Exclusive list of allowed providers. If set, only these providers are used.").optional(),
  ignore: z305.z.array(z305.z.string()).nullable().describe("Providers to exclude from routing.").optional(),
  quantizations: z305.z.array(AgentOpenrouterProviderQuantizationSchema).nullable().describe("Allowed model quantization levels.").optional()
}).describe("Provider routing preferences.\n\nControls which providers are used and in what order when routing\nrequests to upstream model hosts.").meta({ title: "agent.openrouter.Provider" });
var AgentOpenrouterReasoningEffortSchema = z305.z.union([z305.z.literal("none").describe("No reasoning."), z305.z.literal("minimal").describe("Minimal reasoning effort."), z305.z.literal("low").describe("Low reasoning effort."), z305.z.literal("medium").describe("Medium reasoning effort."), z305.z.literal("high").describe("High reasoning effort."), z305.z.literal("xhigh").describe("Maximum reasoning effort.")]).describe("The level of effort the model should put into reasoning.\n\nOnly supported by some models.").meta({ title: "agent.openrouter.ReasoningEffort" });
var AgentOpenrouterReasoningSummaryVerbositySchema = z305.z.union([z305.z.literal("auto").describe("Let the model decide (default, normalized away)."), z305.z.literal("concise").describe("Brief summary of reasoning."), z305.z.literal("detailed").describe("Thorough summary of reasoning.")]).describe("Verbosity of the reasoning summary included in responses.\n\nOnly supported by some models.").meta({ title: "agent.openrouter.ReasoningSummaryVerbosity" });

// src/agent/openrouter/reasoning.ts
var AgentOpenrouterReasoningSchema = z305.z.object({
  enabled: z305.z.boolean().nullable().describe("Whether reasoning is enabled. Defaults to `true` if other fields are set.").optional(),
  max_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Maximum tokens for the reasoning/thinking output.\n\nOnly supported by some models.").optional(),
  effort: AgentOpenrouterReasoningEffortSchema.nullable().describe("The reasoning effort level.\n\nOnly supported by some models.").optional(),
  summary_verbosity: AgentOpenrouterReasoningSummaryVerbositySchema.nullable().describe("Verbosity of reasoning summaries in the response.\n\nOnly supported by some models.").optional()
}).describe('Configuration for model reasoning/thinking capabilities.\n\nSome models (like o1, o3, Claude with extended thinking) support\nexplicit reasoning modes where they can "think" before responding.\nThis struct configures those capabilities.\n\n**Note:** The `max_tokens`, `effort`, and `summary_verbosity` fields are\nonly supported by some models. Unsupported fields are silently ignored.').meta({ title: "agent.openrouter.Reasoning" });
var AgentOpenrouterStopSchema = z305.z.union([z305.z.string().describe("A single stop sequence."), z305.z.array(z305.z.string()).describe("Multiple stop sequences (up to 4 typically supported).")]).describe("Stop sequences that terminate model generation.\n\nWhen the model generates any of these sequences, it immediately\nstops producing further tokens.").meta({ title: "agent.openrouter.Stop" });
var AgentOpenrouterUpstreamSchema = z305.z.literal("openrouter").describe("OpenRouter upstream marker.").meta({ title: "agent.openrouter.Upstream" });
var AgentOpenrouterVerbositySchema = z305.z.union([z305.z.literal("low").describe("Minimal output, concise responses."), z305.z.literal("medium").describe("Balanced output (default, normalized away during preparation)."), z305.z.literal("high").describe("Detailed output with thorough explanations."), z305.z.literal("max").describe("Maximum verbosity, most detailed output possible.")]).describe("The verbosity level for model output.\n\nThis setting hints to the model how detailed its responses should be.\nNot all models support this parameter.").meta({ title: "agent.openrouter.Verbosity" });

// src/agent/openrouter/agentBase.ts
var AgentOpenrouterAgentBaseSchema = z305.z.object({
  upstream: AgentOpenrouterUpstreamSchema.describe("The upstream provider marker."),
  model: z305.z.string().describe('The upstream language model identifier (e.g., `"gpt-4"`, `"claude-3-opus"`).'),
  output_mode: AgentOpenrouterOutputModeSchema.describe("The output mode for vector completions. Ignored for agent completions.").optional(),
  synthetic_reasoning: z305.z.boolean().nullable().describe("Enable synthetic reasoning for non-reasoning LLMs.\n\n**Vector completions only.** Ignored for agent completions.").optional(),
  top_logprobs: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Number of top log probabilities to return (2-20).\n\n**Vector completions only.** Ignored for agent completions.").optional(),
  prefix_messages: z305.z.array(AgentCompletionsMessageMessageSchema).nullable().describe("Messages prepended to the user's prompt.").optional(),
  post_system_prefix_messages: z305.z.array(AgentCompletionsMessageMessageSchema).nullable().describe("Messages inserted after the leading chain of system/developer messages.").optional(),
  suffix_messages: z305.z.array(AgentCompletionsMessageMessageSchema).nullable().describe("Messages appended after the user's prompt.").optional(),
  mcp_servers: z305.z.array(AgentMcpServerSchema).nullable().describe("MCP servers the agent can connect to.").optional(),
  frequency_penalty: z305.z.number().meta({ format: "double" }).nullable().describe("Penalizes tokens based on their frequency in the output so far (-2.0 to 2.0).").optional(),
  logit_bias: z305.z.record(z305.z.string(), z305.z.number().int().meta({ format: "int64" })).nullable().describe("Token ID to bias mapping (-100 to 100). Positive values increase likelihood.").optional(),
  max_completion_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Maximum tokens in the completion.").optional(),
  presence_penalty: z305.z.number().meta({ format: "double" }).nullable().describe("Penalizes tokens based on their presence in the output so far (-2.0 to 2.0).").optional(),
  stop: AgentOpenrouterStopSchema.nullable().describe("Stop sequences that halt generation.").optional(),
  temperature: z305.z.number().meta({ format: "double" }).nullable().describe("Sampling temperature (0.0 to 2.0). Higher = more random.").optional(),
  top_p: z305.z.number().meta({ format: "double" }).nullable().describe("Nucleus sampling probability (0.0 to 1.0).").optional(),
  max_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Maximum tokens (OpenRouter variant of max_completion_tokens).").optional(),
  min_p: z305.z.number().meta({ format: "double" }).nullable().describe("Minimum probability threshold for sampling (0.0 to 1.0).").optional(),
  provider: AgentOpenrouterProviderSchema.nullable().describe("Provider routing preferences.").optional(),
  reasoning: AgentOpenrouterReasoningSchema.nullable().describe("Reasoning/thinking configuration for supported models.").optional(),
  repetition_penalty: z305.z.number().meta({ format: "double" }).nullable().describe("Repetition penalty (0.0 to 2.0). Values > 1.0 penalize repetition.").optional(),
  top_a: z305.z.number().meta({ format: "double" }).nullable().describe("Top-a sampling parameter (0.0 to 1.0).").optional(),
  top_k: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Top-k sampling: only consider the k most likely tokens.").optional(),
  verbosity: AgentOpenrouterVerbositySchema.nullable().describe("Output verbosity hint for supported models.").optional()
}).describe("The base configuration for an OpenRouter Agent (without computed ID).").meta({ title: "agent.openrouter.AgentBase" });

// src/agent/agentBase.ts
var AgentAgentBaseSchema = z305.z.union([AgentOpenrouterAgentBaseSchema, AgentClaudeAgentSdkAgentBaseSchema, AgentMockAgentBaseSchema]).describe("The base configuration for an Agent (without computed ID).\n\nThis is an untagged enum that dispatches to the per-upstream AgentBase.\nDeserialization tries each variant in order until one matches.").meta({ title: "agent.AgentBase" });

// src/agent/completions/request/agent.ts
var AgentCompletionsRequestAgentSchema = z305.z.union([z305.z.string().describe("The content-addressed ID of an Agent stored in ObjectiveAI's database."), AgentAgentBaseSchema.describe("An inline Agent configuration.")]).describe('The agent to use for agent completion.\n\nCan be either:\n- An inline [`AgentBase`](super::super::super::AgentBase) configuration\n- The ID of a previously used Agent (22-character base62 string)\n\nSince IDs are content-addressed, ObjectiveAI stores Agent definitions\nwhen they are successfully used. "Previously used" means the ID exists in\nObjectiveAI\'s database from any successful use by anyone.').meta({ title: "agent.completions.request.Agent" });
var AgentCompletionsRequestProviderDataCollectionSchema = z305.z.union([z305.z.literal("deny").describe("Do not allow data collection."), z305.z.literal("allow").describe("Allow data collection.")]).describe("Data collection policy for providers.").meta({ title: "agent.completions.request.ProviderDataCollection" });
var AgentCompletionsRequestProviderMaxPriceSchema = z305.z.object({
  prompt: z305.z.number().meta({ format: "double" }).nullable().describe("Maximum price per prompt token.").optional(),
  completion: z305.z.number().meta({ format: "double" }).nullable().describe("Maximum price per completion token.").optional(),
  image: z305.z.number().meta({ format: "double" }).nullable().describe("Maximum price per image.").optional(),
  audio: z305.z.number().meta({ format: "double" }).nullable().describe("Maximum price per audio second.").optional(),
  request: z305.z.number().meta({ format: "double" }).nullable().describe("Maximum price per request.").optional()
}).describe("Maximum price constraints per token type.").meta({ title: "agent.completions.request.ProviderMaxPrice" });
var AgentCompletionsRequestProviderSortSchema = z305.z.union([z305.z.literal("price").describe("Prioritize by price (cheapest first)."), z305.z.literal("throughput").describe("Prioritize by throughput (fastest first)."), z305.z.literal("latency").describe("Prioritize by latency (lowest first).")]).describe("How to sort/prioritize providers.").meta({ title: "agent.completions.request.ProviderSort" });

// src/agent/completions/request/provider.ts
var AgentCompletionsRequestProviderSchema = z305.z.object({
  data_collection: AgentCompletionsRequestProviderDataCollectionSchema.nullable().describe("Whether to allow providers to collect data.").optional(),
  zdr: z305.z.boolean().nullable().describe("Whether to use zero data retention providers only.").optional(),
  sort: AgentCompletionsRequestProviderSortSchema.nullable().describe("How to sort/prioritize providers.").optional(),
  max_price: AgentCompletionsRequestProviderMaxPriceSchema.nullable().describe("Maximum price constraints.").optional(),
  preferred_min_throughput: z305.z.number().meta({ format: "double" }).nullable().describe("Preferred minimum throughput (tokens/second).").optional(),
  preferred_max_latency: z305.z.number().meta({ format: "double" }).nullable().describe("Preferred maximum latency (seconds).").optional(),
  min_throughput: z305.z.number().meta({ format: "double" }).nullable().describe("Hard minimum throughput requirement (tokens/second).").optional(),
  max_latency: z305.z.number().meta({ format: "double" }).nullable().describe("Hard maximum latency requirement (seconds).").optional()
}).describe("Provider routing and selection preferences.").meta({ title: "agent.completions.request.Provider" });
var JsonValueSchema = z305.z.union([
  z305.z.string(),
  z305.z.number(),
  z305.z.boolean(),
  z305.z.null(),
  z305.z.array(z305.z.lazy(() => JsonValueSchema)),
  z305.z.record(z305.z.string(), z305.z.lazy(() => JsonValueSchema))
]);

// src/agent/completions/request/responseFormat.ts
var AgentCompletionsRequestResponseFormatSchema = z305.z.union([z305.z.object({
  type: z305.z.literal("text")
}).describe("Plain text response (default)."), z305.z.object({
  type: z305.z.literal("json_object")
}).describe("Response must be valid JSON."), z305.z.object({
  schema: z305.z.record(z305.z.string(), JsonValueSchema).describe("The JSON Schema definition."),
  type: z305.z.literal("json_schema")
}).describe("Response must conform to a JSON schema."), z305.z.object({
  grammar: z305.z.string(),
  type: z305.z.literal("grammar")
}).describe("Response must conform to a grammar."), z305.z.object({
  type: z305.z.literal("python")
}).describe("Response must be valid Python code."), z305.z.object({
  name: z305.z.string().describe("The name of the tool."),
  description: z305.z.string().describe("A description of the tool."),
  schema: z305.z.record(z305.z.string(), JsonValueSchema).describe("The JSON Schema definition."),
  required: z305.z.boolean().nullable().describe("Whether the tool MUST be called.").optional(),
  type: z305.z.literal("tool_call")
}).describe("The final assistant message will contain this tool call")]).describe("The format of the model's response.").meta({ title: "agent.completions.request.ResponseFormat" });

// src/agent/completions/request/responseFormatParam.ts
var AgentCompletionsRequestResponseFormatParamSchema = z305.z.union([AgentCompletionsRequestResponseFormatSchema.describe("A single response format applied to all agents."), z305.z.record(z305.z.string(), AgentCompletionsRequestResponseFormatSchema).describe("Per-agent response formats, keyed by agent ID.")]).describe("Either a single response format or a per-agent map.").meta({ title: "agent.completions.request.ResponseFormatParam" });

// src/agent/completions/request/agentCompletionCreateParams.ts
var AgentCompletionsRequestAgentCompletionCreateParamsSchema = z305.z.object({
  messages: z305.z.array(AgentCompletionsMessageMessageSchema).describe("The conversation messages."),
  provider: AgentCompletionsRequestProviderSchema.nullable().describe("Provider routing preferences.").optional(),
  agent: AgentCompletionsRequestAgentSchema.describe("The agent to use (inline Agent or stored ID)."),
  agents: z305.z.array(AgentCompletionsRequestAgentSchema).nullable().describe("Alternative agents to try if the primary agent fails.").optional(),
  response_format: AgentCompletionsRequestResponseFormatParamSchema.nullable().describe("Output format constraints (text, JSON, or JSON schema).").optional(),
  seed: z305.z.number().int().meta({ format: "int64" }).nullable().describe("Random seed for deterministic generation.").optional(),
  stream: z305.z.boolean().nullable().describe("Whether to stream the response.").optional(),
  mcp_server_authorization: z305.z.record(z305.z.string(), z305.z.string()).nullable().describe("Map from MCP server URL to authorization header value.").optional()
}).describe("Parameters for creating a agent completion.").meta({ title: "agent.completions.request.AgentCompletionCreateParams" });
var AgentCompletionsResponseAssistantRoleSchema = z305.z.union([z305.z.literal("assistant").describe("The assistant role.")]).describe('The role of a message in a response (always "assistant").').meta({ title: "agent.completions.response.AssistantRole" });
var AgentCompletionsResponseFinishReasonSchema = z305.z.union([z305.z.literal("stop").describe("The model reached a natural stop point or stop sequence."), z305.z.literal("length").describe("The model reached the maximum token limit."), z305.z.literal("tool_calls").describe("The model decided to call one or more tools."), z305.z.literal("content_filter").describe("The response was filtered due to content policy."), z305.z.literal("error").describe("An error occurred during generation.")]).describe("The reason the model stopped generating.").meta({ title: "agent.completions.response.FinishReason" });
var AgentCompletionsResponseTopLogprobSchema = z305.z.object({
  token: z305.z.string().describe("The token string."),
  bytes: z305.z.array(z305.z.number().int().min(0).max(255).meta({ format: "uint8" })).nullable().describe("The raw bytes of the token.").optional(),
  logprob: z305.z.number().meta({ format: "double" }).nullable().describe("The log probability of this token.").optional()
}).describe("A top alternative token with its log probability.").meta({ title: "agent.completions.response.TopLogprob" });

// src/agent/completions/response/logprob.ts
var AgentCompletionsResponseLogprobSchema = z305.z.object({
  token: z305.z.string().describe("The token string."),
  bytes: z305.z.array(z305.z.number().int().min(0).max(255).meta({ format: "uint8" })).nullable().describe("The raw bytes of the token.").optional(),
  logprob: z305.z.number().meta({ format: "double" }).describe("The log probability of this token."),
  top_logprobs: z305.z.array(AgentCompletionsResponseTopLogprobSchema).describe("The top alternative tokens and their log probabilities.")
}).describe("Log probability information for a single token.").meta({ title: "agent.completions.response.Logprob" });

// src/agent/completions/response/logprobs.ts
var AgentCompletionsResponseLogprobsSchema = z305.z.object({
  content: z305.z.array(AgentCompletionsResponseLogprobSchema).nullable().describe("Log probabilities for content tokens.").optional(),
  refusal: z305.z.array(AgentCompletionsResponseLogprobSchema).nullable().describe("Log probabilities for refusal tokens.").optional()
}).describe("Log probabilities for generated tokens.").meta({ title: "agent.completions.response.Logprobs" });
var AgentCompletionsResponseCompletionTokensDetailsSchema = z305.z.object({
  accepted_prediction_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Tokens from accepted predictions (speculative decoding).").optional(),
  audio_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Audio output tokens.").optional(),
  reasoning_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Tokens used for reasoning/thinking.").optional(),
  rejected_prediction_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Tokens from rejected predictions (speculative decoding).").optional()
}).describe("Detailed breakdown of completion token usage.").meta({ title: "agent.completions.response.CompletionTokensDetails" });
var AgentCompletionsResponseCostDetailsSchema = z305.z.object({
  upstream_inference_cost: z305.z.number().meta({ format: "double" }).describe("Cost charged by the immediate upstream (e.g., OpenRouter)."),
  upstream_upstream_inference_cost: z305.z.number().meta({ format: "double" }).describe("Cost charged by the upstream's upstream (e.g., the actual model provider).")
}).describe("Detailed cost breakdown.").meta({ title: "agent.completions.response.CostDetails" });
var AgentCompletionsResponsePromptTokensDetailsSchema = z305.z.object({
  audio_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Audio input tokens.").optional(),
  cached_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Tokens served from cache.").optional(),
  cache_write_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Tokens written to cache.").optional(),
  video_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Video input tokens.").optional()
}).describe("Detailed breakdown of prompt token usage.").meta({ title: "agent.completions.response.PromptTokensDetails" });

// src/agent/completions/response/upstreamUsage.ts
var AgentCompletionsResponseUpstreamUsageSchema = z305.z.object({
  completion_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Number of tokens in the completion."),
  prompt_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Number of tokens in the prompt."),
  total_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Total tokens (prompt + completion)."),
  completion_tokens_details: AgentCompletionsResponseCompletionTokensDetailsSchema.nullable().describe("Detailed breakdown of completion tokens.").optional(),
  prompt_tokens_details: AgentCompletionsResponsePromptTokensDetailsSchema.nullable().describe("Detailed breakdown of prompt tokens.").optional(),
  cost: z305.z.number().meta({ format: "double" }).describe("The cost charged by ObjectiveAI for this request."),
  cost_details: AgentCompletionsResponseCostDetailsSchema.nullable().describe("Detailed cost breakdown.").optional(),
  total_cost: z305.z.number().meta({ format: "double" }).describe("Total cost including ObjectiveAI's charge plus all upstream charges.\nFor BYOK requests, ObjectiveAI only charges the cost_multiplier difference,\nbut total_cost still includes what the upstream provider charged."),
  cost_multiplier: z305.z.number().meta({ format: "double" }).describe("The multiplier applied to compute ObjectiveAI's charge."),
  is_byok: z305.z.boolean().describe("Whether this request used Bring Your Own Key (BYOK).")
}).describe("Token usage and cost information from an upstream provider.\n\nThis is the per-assistant-response usage yielded by upstream clients.\nIt includes upstream-specific fields like `cost_multiplier` and `is_byok`.").meta({ title: "agent.completions.response.UpstreamUsage" });

// src/agent/completions/response/streaming/assistantResponseChunk.ts
var AgentCompletionsResponseStreamingAssistantResponseChunkSchema = z305.z.object({
  role: AgentCompletionsResponseAssistantRoleSchema,
  index: z305.z.number().int().min(0).meta({ format: "uint64" }),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }),
  agent: z305.z.string(),
  model: z305.z.string(),
  upstream_id: z305.z.string(),
  reasoning: z305.z.string().nullable().optional(),
  tool_calls: z305.z.array(AgentCompletionsMessageAssistantToolCallDeltaSchema).nullable().optional(),
  content: AgentCompletionsMessageRichContentSchema.nullable().optional(),
  refusal: z305.z.string().nullable().optional(),
  finish_reason: AgentCompletionsResponseFinishReasonSchema.nullable().optional(),
  logprobs: AgentCompletionsResponseLogprobsSchema.nullable().optional(),
  service_tier: z305.z.string().nullable().optional(),
  system_fingerprint: z305.z.string().nullable().optional(),
  provider: z305.z.string().nullable().optional(),
  usage: AgentCompletionsResponseUpstreamUsageSchema.nullable().describe("Upstream usage for this assistant response (set by upstream clients).").optional()
}).describe("A chunk of a streaming agent completion response.\n\nMultiple chunks are received via Server-Sent Events and can be\naccumulated into a complete [`AgentCompletion`](response::unary::AgentCompletion)\nusing the [`push`](Self::push) method.").meta({ title: "agent.completions.response.streaming.AssistantResponseChunk" });
var AgentCompletionsResponseToolRoleSchema = z305.z.literal("tool").meta({ title: "agent.completions.response.ToolRole" });

// src/agent/completions/response/toolResponse.ts
var AgentCompletionsResponseToolResponseSchema = z305.z.object({
  role: AgentCompletionsResponseToolRoleSchema,
  index: z305.z.number().int().min(0).meta({ format: "uint64" }),
  content: AgentCompletionsMessageRichContentSchema.describe("The content of the tool response."),
  tool_call_id: z305.z.string().describe("The ID of the tool call this message responds to.")
}).describe("A tool message containing the result of a tool call.").meta({ title: "agent.completions.response.ToolResponse" });

// src/agent/completions/response/streaming/messageChunk.ts
var AgentCompletionsResponseStreamingMessageChunkSchema = z305.z.union([AgentCompletionsResponseStreamingAssistantResponseChunkSchema, AgentCompletionsResponseToolResponseSchema]).meta({ title: "agent.completions.response.streaming.MessageChunk" });
var AgentCompletionsResponseStreamingObjectSchema = z305.z.union([z305.z.literal("agent.completion.chunk").describe("A agent completion chunk object.")]).describe("The object type for streaming agent completion chunks.").meta({ title: "agent.completions.response.streaming.Object" });
var AgentCompletionsResponseUsageSchema = z305.z.object({
  completion_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Total tokens generated across all assistant responses."),
  prompt_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Total prompt tokens across all assistant responses."),
  total_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Sum of completion and prompt tokens."),
  completion_tokens_details: AgentCompletionsResponseCompletionTokensDetailsSchema.nullable().describe("Breakdown of completion tokens (reasoning, audio, etc.) if available.").optional(),
  prompt_tokens_details: AgentCompletionsResponsePromptTokensDetailsSchema.nullable().describe("Breakdown of prompt tokens (cached, audio, etc.) if available.").optional(),
  cost: z305.z.number().meta({ format: "double" }).describe("Cost charged by ObjectiveAI for this request."),
  cost_details: AgentCompletionsResponseCostDetailsSchema.nullable().describe("Breakdown of upstream and upstream_upstream costs if available.").optional(),
  total_cost: z305.z.number().meta({ format: "double" }).describe("Total cost including upstream provider charges. Only differs from `cost`\nwhen using BYOK (Bring Your Own Key).")
}).describe('Aggregated token and cost usage for an agent completion.\n\nThis is the "primary" usage type that aggregates across all upstream\nassistant responses within a single agent completion.').meta({ title: "agent.completions.response.Usage" });
var AgentUpstreamSchema = z305.z.union([z305.z.literal("unknown").describe("Unknown Upstream."), z305.z.literal("openrouter").describe("OpenRouter Upstream."), z305.z.literal("claude_agent_sdk").describe("Claude Agent SDK Upstream."), z305.z.literal("mock").describe("Mock Upstream.")]).describe("Supported agent upstreams.").meta({ title: "agent.Upstream" });
var ResponseErrorSchema = z305.z.object({
  code: z305.z.number().int().min(0).max(65535).meta({ format: "uint16" }).describe("The HTTP status code of the error response."),
  message: JsonValueSchema.describe("The error message or details as a JSON value.")
}).describe('An error returned by the ObjectiveAI API.\n\nThis struct represents an API error response containing an HTTP status\ncode and a message. The message can be any JSON value, allowing for\nboth simple string errors and structured error objects.\n\n# Examples\n\n```\nuse objectiveai::error::ResponseError;\nuse serde_json::json;\n\nlet error = ResponseError {\n    code: 400,\n    message: json!({"error": "Invalid request"}),\n};\n```').meta({ title: "ResponseError" });

// src/agent/completions/response/streaming/agentCompletionChunk.ts
var AgentCompletionsResponseStreamingAgentCompletionChunkSchema = z305.z.object({
  id: z305.z.string(),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }),
  messages: z305.z.array(AgentCompletionsResponseStreamingMessageChunkSchema),
  object: AgentCompletionsResponseStreamingObjectSchema.describe('The object type (always "agent.completion.chunk").'),
  usage: AgentCompletionsResponseUsageSchema.nullable().describe("Token usage (only present in the final chunk).").optional(),
  upstream: AgentUpstreamSchema.describe("Upstream provider"),
  error: ResponseErrorSchema.nullable().describe("Error details if this completion failed.").optional()
}).describe("A chunk of a streaming agent completion response.\n\nMultiple chunks are received via Server-Sent Events and can be\naccumulated into a complete [`AgentCompletion`](response::unary::AgentCompletion)\nusing the [`push`](Self::push) method.").meta({ title: "agent.completions.response.streaming.AgentCompletionChunk" });

// src/agent/completions/response/logprobsMerged.ts
function agentCompletionsResponseLogprobsMerged(a, b) {
  let changed = false;
  let content = a.content;
  if (a.content != null && b.content != null) {
    if (b.content.length > 0) {
      content = [...a.content, ...b.content];
      changed = true;
    }
  } else if (b.content != null) {
    content = b.content;
    changed = true;
  }
  let refusal = a.refusal;
  if (a.refusal != null && b.refusal != null) {
    if (b.refusal.length > 0) {
      refusal = [...a.refusal, ...b.refusal];
      changed = true;
    }
  } else if (b.refusal != null) {
    refusal = b.refusal;
    changed = true;
  }
  if (!changed) return [a, false];
  return [{
    ...content !== void 0 ? { content } : {},
    ...refusal !== void 0 ? { refusal } : {}
  }, true];
}

// src/agent/completions/response/completionTokensDetailsMerged.ts
function mergedOptionU64(a, b) {
  if (a != null && b != null) return [a + b, true];
  if (a != null) return [a, false];
  if (b != null) return [b, true];
  return [void 0, false];
}
function agentCompletionsResponseCompletionTokensDetailsMerged(a, b) {
  const [accepted_prediction_tokens, c1] = mergedOptionU64(a.accepted_prediction_tokens, b.accepted_prediction_tokens);
  const [audio_tokens, c2] = mergedOptionU64(a.audio_tokens, b.audio_tokens);
  const [reasoning_tokens, c3] = mergedOptionU64(a.reasoning_tokens, b.reasoning_tokens);
  const [rejected_prediction_tokens, c4] = mergedOptionU64(a.rejected_prediction_tokens, b.rejected_prediction_tokens);
  const changed = c1 || c2 || c3 || c4;
  if (!changed) return [a, false];
  return [{
    ...accepted_prediction_tokens != null ? { accepted_prediction_tokens } : {},
    ...audio_tokens != null ? { audio_tokens } : {},
    ...reasoning_tokens != null ? { reasoning_tokens } : {},
    ...rejected_prediction_tokens != null ? { rejected_prediction_tokens } : {}
  }, true];
}

// src/agent/completions/response/promptTokensDetailsMerged.ts
function mergedOptionU642(a, b) {
  if (a != null && b != null) return [a + b, true];
  if (a != null) return [a, false];
  if (b != null) return [b, true];
  return [void 0, false];
}
function agentCompletionsResponsePromptTokensDetailsMerged(a, b) {
  const [audio_tokens, c1] = mergedOptionU642(a.audio_tokens, b.audio_tokens);
  const [cached_tokens, c2] = mergedOptionU642(a.cached_tokens, b.cached_tokens);
  const [cache_write_tokens, c3] = mergedOptionU642(a.cache_write_tokens, b.cache_write_tokens);
  const [video_tokens, c4] = mergedOptionU642(a.video_tokens, b.video_tokens);
  const changed = c1 || c2 || c3 || c4;
  if (!changed) return [a, false];
  return [{
    ...audio_tokens != null ? { audio_tokens } : {},
    ...cached_tokens != null ? { cached_tokens } : {},
    ...cache_write_tokens != null ? { cache_write_tokens } : {},
    ...video_tokens != null ? { video_tokens } : {}
  }, true];
}

// src/agent/completions/response/costDetailsMerged.ts
function agentCompletionsResponseCostDetailsMerged(a, b) {
  const upstream_inference_cost = a.upstream_inference_cost + b.upstream_inference_cost;
  const upstream_upstream_inference_cost = a.upstream_upstream_inference_cost + b.upstream_upstream_inference_cost;
  return [{
    upstream_inference_cost,
    upstream_upstream_inference_cost
  }, true];
}

// src/agent/completions/response/upstreamUsageMerged.ts
function agentCompletionsResponseUpstreamUsageMerged(a, b) {
  const completion_tokens = a.completion_tokens + b.completion_tokens;
  const prompt_tokens = a.prompt_tokens + b.prompt_tokens;
  const total_tokens = a.total_tokens + b.total_tokens;
  const [completion_tokens_details, c1] = merge(
    a.completion_tokens_details ?? void 0,
    b.completion_tokens_details ?? void 0,
    agentCompletionsResponseCompletionTokensDetailsMerged
  );
  const [prompt_tokens_details, c2] = merge(
    a.prompt_tokens_details ?? void 0,
    b.prompt_tokens_details ?? void 0,
    agentCompletionsResponsePromptTokensDetailsMerged
  );
  const cost = a.cost + b.cost;
  const [cost_details, c3] = merge(
    a.cost_details ?? void 0,
    b.cost_details ?? void 0,
    agentCompletionsResponseCostDetailsMerged
  );
  const total_cost = a.total_cost + b.total_cost;
  return [{
    completion_tokens,
    prompt_tokens,
    total_tokens,
    ...completion_tokens_details != null ? { completion_tokens_details } : {},
    ...prompt_tokens_details != null ? { prompt_tokens_details } : {},
    cost,
    ...cost_details != null ? { cost_details } : {},
    total_cost,
    cost_multiplier: a.cost_multiplier,
    is_byok: a.is_byok
  }, true];
}

// src/agent/completions/response/streaming/assistantResponseChunkMerged.ts
function mergedOptionString(a, b) {
  if (a != null && b != null) {
    return mergedString(a, b);
  } else if (b != null) {
    return [b, true];
  }
  return [a, false];
}
function agentCompletionsResponseStreamingAssistantResponseChunkMerged(a, b) {
  let changed = false;
  const [reasoning, c1] = mergedOptionString(a.reasoning, b.reasoning);
  if (c1) changed = true;
  let tool_calls = a.tool_calls;
  if (a.tool_calls != null && b.tool_calls != null) {
    const [merged, c] = agentCompletionsMessageAssistantToolCallDeltaMergedList(a.tool_calls, b.tool_calls);
    tool_calls = merged;
    if (c) changed = true;
  } else if (b.tool_calls != null) {
    tool_calls = b.tool_calls;
    changed = true;
  }
  let content = a.content;
  if (a.content != null && b.content != null) {
    const [merged, c] = agentCompletionsMessageRichContentMerged(a.content, b.content);
    content = merged;
    if (c) changed = true;
  } else if (b.content != null) {
    content = b.content;
    changed = true;
  }
  const [refusal, c2] = mergedOptionString(a.refusal, b.refusal);
  if (c2) changed = true;
  let finish_reason = a.finish_reason;
  if (a.finish_reason == null && b.finish_reason != null) {
    finish_reason = b.finish_reason;
    changed = true;
  }
  let logprobs = a.logprobs;
  if (a.logprobs != null && b.logprobs != null) {
    const [merged, c] = agentCompletionsResponseLogprobsMerged(a.logprobs, b.logprobs);
    logprobs = merged;
    if (c) changed = true;
  } else if (b.logprobs != null) {
    logprobs = b.logprobs;
    changed = true;
  }
  let upstream_id = a.upstream_id;
  if (a.upstream_id === "" && b.upstream_id !== "") {
    upstream_id = b.upstream_id;
    changed = true;
  }
  let service_tier = a.service_tier;
  if (a.service_tier == null && b.service_tier != null) {
    service_tier = b.service_tier;
    changed = true;
  }
  let system_fingerprint = a.system_fingerprint;
  if (a.system_fingerprint == null && b.system_fingerprint != null) {
    system_fingerprint = b.system_fingerprint;
    changed = true;
  }
  let provider = a.provider;
  if (a.provider == null && b.provider != null) {
    provider = b.provider;
    changed = true;
  }
  let usage = a.usage;
  if (a.usage != null && b.usage != null) {
    const [merged, c] = agentCompletionsResponseUpstreamUsageMerged(a.usage, b.usage);
    usage = merged;
    if (c) changed = true;
  } else if (b.usage != null) {
    usage = b.usage;
    changed = true;
  }
  if (!changed) return [a, false];
  return [{
    role: a.role,
    index: a.index,
    created: a.created,
    agent: a.agent,
    model: a.model,
    upstream_id,
    ...reasoning != null ? { reasoning } : {},
    ...tool_calls != null ? { tool_calls } : {},
    ...content != null ? { content } : {},
    ...refusal != null ? { refusal } : {},
    // finish_reason: no skip_serializing_if — must be present (null or value)
    ...finish_reason !== void 0 ? { finish_reason } : {},
    ...logprobs != null ? { logprobs } : {},
    ...service_tier != null ? { service_tier } : {},
    ...system_fingerprint != null ? { system_fingerprint } : {},
    ...provider != null ? { provider } : {},
    ...usage != null ? { usage } : {}
  }, true];
}

// src/agent/completions/response/streaming/messageChunkMerged.ts
function messageChunkIndex(chunk) {
  return chunk.index;
}
function agentCompletionsResponseStreamingMessageChunkMerged(a, b) {
  if (a.role === "assistant" && b.role === "assistant") {
    return agentCompletionsResponseStreamingAssistantResponseChunkMerged(a, b);
  }
  return [a, false];
}
function agentCompletionsResponseStreamingMessageChunkMergedList(a, b) {
  let changed = false;
  const result = [...a];
  for (const bItem of b) {
    const bIndex = messageChunkIndex(bItem);
    const existingIdx = result.findIndex((x) => messageChunkIndex(x) === bIndex);
    if (existingIdx !== -1) {
      const [merged, c] = agentCompletionsResponseStreamingMessageChunkMerged(result[existingIdx], bItem);
      if (c) {
        result[existingIdx] = merged;
        changed = true;
      }
    } else {
      result.push(bItem);
      changed = true;
    }
  }
  if (!changed) return [a, false];
  return [result, true];
}

// src/agent/completions/response/usageMerged.ts
function agentCompletionsResponseUsageMerged(a, b) {
  const completion_tokens = a.completion_tokens + b.completion_tokens;
  const prompt_tokens = a.prompt_tokens + b.prompt_tokens;
  const total_tokens = a.total_tokens + b.total_tokens;
  const [completion_tokens_details, c1] = merge(
    a.completion_tokens_details ?? void 0,
    b.completion_tokens_details ?? void 0,
    agentCompletionsResponseCompletionTokensDetailsMerged
  );
  const [prompt_tokens_details, c2] = merge(
    a.prompt_tokens_details ?? void 0,
    b.prompt_tokens_details ?? void 0,
    agentCompletionsResponsePromptTokensDetailsMerged
  );
  const cost = a.cost + b.cost;
  const [cost_details, c3] = merge(
    a.cost_details ?? void 0,
    b.cost_details ?? void 0,
    agentCompletionsResponseCostDetailsMerged
  );
  const total_cost = a.total_cost + b.total_cost;
  return [{
    completion_tokens,
    prompt_tokens,
    total_tokens,
    ...completion_tokens_details != null ? { completion_tokens_details } : {},
    ...prompt_tokens_details != null ? { prompt_tokens_details } : {},
    cost,
    ...cost_details != null ? { cost_details } : {},
    total_cost
  }, true];
}

// src/agent/completions/response/streaming/agentCompletionChunkMerged.ts
function agentCompletionsResponseStreamingAgentCompletionChunkMerged(a, b) {
  let changed = false;
  const [messages, c1] = agentCompletionsResponseStreamingMessageChunkMergedList(a.messages, b.messages);
  if (c1) changed = true;
  let usage = a.usage;
  if (a.usage != null && b.usage != null) {
    const [merged, c] = agentCompletionsResponseUsageMerged(a.usage, b.usage);
    usage = merged;
    if (c) changed = true;
  } else if (b.usage != null) {
    usage = b.usage;
    changed = true;
  }
  let error = a.error;
  if (a.error == null && b.error != null) {
    error = b.error;
    changed = true;
  }
  if (!changed) return [a, false];
  return [{
    id: a.id,
    created: a.created,
    messages,
    object: a.object,
    ...usage != null ? { usage } : {},
    upstream: a.upstream,
    ...error != null ? { error } : {}
  }, true];
}
var AgentCompletionsResponseUnaryAssistantResponseSchema = z305.z.object({
  role: AgentCompletionsResponseAssistantRoleSchema,
  index: z305.z.number().int().min(0).meta({ format: "uint64" }),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }),
  agent: z305.z.string(),
  model: z305.z.string(),
  upstream_id: z305.z.string(),
  reasoning: z305.z.string().nullable().optional(),
  tool_calls: z305.z.array(AgentCompletionsMessageAssistantToolCallSchema).nullable().optional(),
  content: AgentCompletionsMessageRichContentSchema.nullable().optional(),
  refusal: z305.z.string().nullable().optional(),
  finish_reason: AgentCompletionsResponseFinishReasonSchema,
  logprobs: AgentCompletionsResponseLogprobsSchema.nullable().optional(),
  service_tier: z305.z.string().nullable().optional(),
  system_fingerprint: z305.z.string().nullable().optional(),
  provider: z305.z.string().nullable().optional(),
  usage: AgentCompletionsResponseUpstreamUsageSchema.describe("Upstream usage for this assistant response (set by upstream clients).")
}).describe("An assistant response in a unary agent completion.").meta({ title: "agent.completions.response.unary.AssistantResponse" });

// src/agent/completions/response/unary/message.ts
var AgentCompletionsResponseUnaryMessageSchema = z305.z.union([AgentCompletionsResponseUnaryAssistantResponseSchema, AgentCompletionsResponseToolResponseSchema]).meta({ title: "agent.completions.response.unary.Message" });
var AgentCompletionsResponseUnaryObjectSchema = z305.z.union([z305.z.literal("agent.completion").describe("A agent completion object.")]).describe("The object type for agent completion responses.").meta({ title: "agent.completions.response.unary.Object" });

// src/agent/completions/response/unary/agentCompletion.ts
var AgentCompletionsResponseUnaryAgentCompletionSchema = z305.z.object({
  id: z305.z.string(),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }),
  messages: z305.z.array(AgentCompletionsResponseUnaryMessageSchema),
  object: AgentCompletionsResponseUnaryObjectSchema.describe('The object type (always "agent.completion").'),
  usage: AgentCompletionsResponseUsageSchema,
  upstream: AgentUpstreamSchema.describe("Upstream provider"),
  error: ResponseErrorSchema.nullable().describe("Error details if this completion failed.").optional()
}).describe("A complete agent completion response.").meta({ title: "agent.completions.response.unary.AgentCompletion" });
var AgentCompletionsRequestAgentCompletionCreateParamsStreamingSchema = AgentCompletionsRequestAgentCompletionCreateParamsSchema.extend({
  stream: z305__default.default.literal(true)
});
var AgentCompletionsRequestAgentCompletionCreateParamsUnarySchema = AgentCompletionsRequestAgentCompletionCreateParamsSchema.extend({
  stream: z305__default.default.literal(false).optional().nullable()
});
function agentCompletionsCreateAgentCompletion(client, body, options) {
  if (body.stream) {
    return client.post_streaming(
      "/agent/completions",
      body,
      options
    );
  }
  return client.post_unary(
    "/agent/completions",
    body,
    options
  );
}
var AgentMockAgentSchema = z305.z.object({
  id: z305.z.string().describe("The deterministic content-addressed ID (22-character base62 string)."),
  upstream: AgentMockUpstreamSchema.describe("The upstream provider marker."),
  output_mode: AgentMockOutputModeSchema.describe("The output mode for vector completions. Ignored for agent completions."),
  top_logprobs: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Number of top log probabilities to return (2-20).\n\n**Vector completions only.** Ignored for agent completions.").optional(),
  error: z305.z.boolean().nullable().describe("If true, the mock client will return an error instead of a response.").optional(),
  invention: z305.z.boolean().nullable().describe("If true, this mock agent supports invention tool calling.\nIncompatible with output modes other than `instruction`.").optional()
}).describe("A validated Mock Agent with its computed content-addressed ID.").meta({ title: "agent.mock.Agent" });
var AgentOpenrouterAgentSchema = z305.z.object({
  id: z305.z.string().describe("The deterministic content-addressed ID (22-character base62 string)."),
  upstream: AgentOpenrouterUpstreamSchema.describe("The upstream provider marker."),
  model: z305.z.string().describe('The upstream language model identifier (e.g., `"gpt-4"`, `"claude-3-opus"`).'),
  output_mode: AgentOpenrouterOutputModeSchema.describe("The output mode for vector completions. Ignored for agent completions.").optional(),
  synthetic_reasoning: z305.z.boolean().nullable().describe("Enable synthetic reasoning for non-reasoning LLMs.\n\n**Vector completions only.** Ignored for agent completions.").optional(),
  top_logprobs: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Number of top log probabilities to return (2-20).\n\n**Vector completions only.** Ignored for agent completions.").optional(),
  prefix_messages: z305.z.array(AgentCompletionsMessageMessageSchema).nullable().describe("Messages prepended to the user's prompt.").optional(),
  post_system_prefix_messages: z305.z.array(AgentCompletionsMessageMessageSchema).nullable().describe("Messages inserted after the leading chain of system/developer messages.").optional(),
  suffix_messages: z305.z.array(AgentCompletionsMessageMessageSchema).nullable().describe("Messages appended after the user's prompt.").optional(),
  mcp_servers: z305.z.array(AgentMcpServerSchema).nullable().describe("MCP servers the agent can connect to.").optional(),
  frequency_penalty: z305.z.number().meta({ format: "double" }).nullable().describe("Penalizes tokens based on their frequency in the output so far (-2.0 to 2.0).").optional(),
  logit_bias: z305.z.record(z305.z.string(), z305.z.number().int().meta({ format: "int64" })).nullable().describe("Token ID to bias mapping (-100 to 100). Positive values increase likelihood.").optional(),
  max_completion_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Maximum tokens in the completion.").optional(),
  presence_penalty: z305.z.number().meta({ format: "double" }).nullable().describe("Penalizes tokens based on their presence in the output so far (-2.0 to 2.0).").optional(),
  stop: AgentOpenrouterStopSchema.nullable().describe("Stop sequences that halt generation.").optional(),
  temperature: z305.z.number().meta({ format: "double" }).nullable().describe("Sampling temperature (0.0 to 2.0). Higher = more random.").optional(),
  top_p: z305.z.number().meta({ format: "double" }).nullable().describe("Nucleus sampling probability (0.0 to 1.0).").optional(),
  max_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Maximum tokens (OpenRouter variant of max_completion_tokens).").optional(),
  min_p: z305.z.number().meta({ format: "double" }).nullable().describe("Minimum probability threshold for sampling (0.0 to 1.0).").optional(),
  provider: AgentOpenrouterProviderSchema.nullable().describe("Provider routing preferences.").optional(),
  reasoning: AgentOpenrouterReasoningSchema.nullable().describe("Reasoning/thinking configuration for supported models.").optional(),
  repetition_penalty: z305.z.number().meta({ format: "double" }).nullable().describe("Repetition penalty (0.0 to 2.0). Values > 1.0 penalize repetition.").optional(),
  top_a: z305.z.number().meta({ format: "double" }).nullable().describe("Top-a sampling parameter (0.0 to 1.0).").optional(),
  top_k: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Top-k sampling: only consider the k most likely tokens.").optional(),
  verbosity: AgentOpenrouterVerbositySchema.nullable().describe("Output verbosity hint for supported models.").optional()
}).describe("A validated OpenRouter Agent with its computed content-addressed ID.").meta({ title: "agent.openrouter.Agent" });
var AgentAgentSchema = z305.z.union([AgentOpenrouterAgentSchema, AgentClaudeAgentSdkAgentSchema, AgentMockAgentSchema]).describe("A validated Agent with its computed content-addressed ID.\n\nThis is an untagged enum that dispatches to the per-upstream Agent.").meta({ title: "agent.Agent" });
var AgentGetAgentSchema = z305.z.union([AgentOpenrouterAgentSchema, AgentClaudeAgentSdkAgentSchema, AgentMockAgentSchema]).and(z305.z.object({
  created: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Unix timestamp when this Agent was first used.")
})).describe("Response containing a single Agent with creation timestamp.").meta({ title: "agent.GetAgent" });
var AgentListAgentItemSchema = z305.z.object({
  id: z305.z.string().describe("The unique content-addressed ID of the Agent.")
}).describe("Summary information for a listed Agent.").meta({ title: "agent.ListAgentItem" });

// src/agent/listAgent.ts
var AgentListAgentSchema = z305.z.object({
  data: z305.z.array(AgentListAgentItemSchema).describe("The list of Agent summaries.")
}).describe("Response containing a list of Agents.").meta({ title: "agent.ListAgent" });
var AgentOutputModeSchema = z305.z.union([z305.z.literal("instruction").describe("The model is instructed via the prompt to output a specific key.\n\nThis is the default and most widely supported mode."), z305.z.literal("json_schema").describe("A JSON schema response format is used with an enum of possible keys.\n\nRequires model support for structured JSON output."), z305.z.literal("tool_call").describe("A forced tool call with an argument schema containing possible keys.\n\nRequires model support for tool/function calling.")]).describe("The method used to constrain LLM output to valid response keys.\n\nIn vector completions, the model must select from a predefined set of\nresponses. This enum controls *how* that constraint is enforced.\n\n**Note:** This setting is only relevant for vector completions and is\ncompletely ignored for agent completions.").meta({ title: "agent.OutputMode" });
var AgentUsageAgentSchema = z305.z.object({
  requests: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Total number of requests made with this Agent."),
  completion_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Total completion tokens generated."),
  prompt_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Total prompt tokens processed."),
  total_cost: z305.z.number().meta({ format: "double" }).describe("Total cost incurred.")
}).describe("Usage statistics for an Agent.").meta({ title: "agent.UsageAgent" });
var AgentWithFallbacksAndCountAgentAgentSchema = z305.z.union([AgentOpenrouterAgentSchema, AgentClaudeAgentSdkAgentSchema, AgentMockAgentSchema]).and(z305.z.object({
  count: z305.z.number().int().min(0).meta({ format: "uint64" }).default(1).describe("Number of instances of this agent in the ensemble. Defaults to 1.").optional(),
  fallbacks: z305.z.array(AgentAgentSchema).nullable().describe("Fallback agents to try if the primary fails.").optional()
})).describe("Wrapper that adds fallback agents and a count to any agent type.\n\nUsed to specify how many instances of an agent to include in an ensemble,\nalong with fallback agents to try if the primary fails.").meta({ title: "agent.WithFallbacksAndCount.agent.Agent" });
var AgentWithFallbacksAndCountAgentAgentBaseSchema = z305.z.union([AgentOpenrouterAgentBaseSchema, AgentClaudeAgentSdkAgentBaseSchema, AgentMockAgentBaseSchema]).and(z305.z.object({
  count: z305.z.number().int().min(0).meta({ format: "uint64" }).default(1).describe("Number of instances of this agent in the ensemble. Defaults to 1.").optional(),
  fallbacks: z305.z.array(AgentAgentBaseSchema).nullable().describe("Fallback agents to try if the primary fails.").optional()
})).describe("Wrapper that adds fallback agents and a count to any agent type.\n\nUsed to specify how many instances of an agent to include in an ensemble,\nalong with fallback agents to try if the primary fails.").meta({ title: "agent.WithFallbacksAndCount.agent.AgentBase" });

// src/agent/http.ts
function agentListAgents(client, options) {
  return client.get_unary("/agents", void 0, options);
}
function agentGetAgent(client, agentId, options) {
  return client.get_unary(`/agents/${agentId}`, void 0, options);
}
function agentGetAgentUsage(client, agentId, options) {
  return client.get_unary(
    `/agents/${agentId}/usage`,
    void 0,
    options
  );
}
var PrefixedUuidSchema = z305.z.object({
  uuid: z305.z.string().meta({ format: "uuid" })
}).describe("A UUID with a 3-character prefix for type-safe identifiers.\n\nThis struct wraps a standard UUID and adds a compile-time prefix,\nensuring that different types of identifiers (API keys, ensemble IDs, etc.)\ncannot be confused at the type level.\n\nThe prefix is specified as three `const char` generic parameters.\n\n# Type Parameters\n\n* `PFX_1` - First character of the prefix\n* `PFX_2` - Second character of the prefix\n* `PFX_3` - Third character of the prefix\n\n# Examples\n\n```\nuse objectiveai::prefixed_uuid::PrefixedUuid;\n\n// Define an API key type with prefix \"apk\"\ntype ApiKey = PrefixedUuid<'a', 'p', 'k'>;\n\n// Create a new API key\nlet key = ApiKey::new();\nprintln!(\"{}\", key); // Outputs: apk<uuid>\n```").meta({ title: "PrefixedUuid" });

// src/auth/apiKeyWithMetadata.ts
var AuthApiKeyWithMetadataSchema = z305.z.object({
  api_key: PrefixedUuidSchema.describe("The API key itself."),
  created: z305.z.string().meta({ format: "date-time" }).describe("The timestamp when the API key was created (RFC 3339 format)."),
  expires: z305.z.string().meta({ format: "date-time" }).nullable().describe("The timestamp when the API key expires, or `None` if it does not expire.").optional(),
  disabled: z305.z.string().meta({ format: "date-time" }).nullable().describe("The timestamp when the API key was disabled, or `None` if it is active.").optional(),
  name: z305.z.string().describe("The user-provided name of the API key."),
  description: z305.z.string().nullable().describe("The user-provided description of the API key, or `None` if not provided.").optional()
}).describe("An ObjectiveAI API Key with associated metadata.\n\nThis struct contains the API key itself along with information about\nwhen it was created, when it expires (if ever), whether it has been\ndisabled, and user-provided name and description.").meta({ title: "auth.ApiKeyWithMetadata" });
var AuthCreateApiKeyRequestSchema = z305.z.object({
  expires: z305.z.string().meta({ format: "date-time" }).nullable().describe("The expiration timestamp for the API key, or `None` for a non-expiring key.").optional(),
  name: z305.z.string().describe("A user-provided name to identify this API key."),
  description: z305.z.string().nullable().describe("An optional description providing additional context about the key's purpose.").optional()
}).describe("Request to create a new API key.\n\n# Fields\n\n* `expires` - Optional expiration timestamp. If `None`, the key never expires.\n* `name` - A user-provided name for identifying the key.\n* `description` - Optional description providing additional context.").meta({ title: "auth.CreateApiKeyRequest" });
var AuthCreateOpenRouterByokApiKeyRequestSchema = z305.z.object({
  api_key: z305.z.string().describe("The OpenRouter API key to associate with the user's account.")
}).describe("Request to create or update an OpenRouter BYOK (Bring Your Own Key) API key.\n\nThis allows users to provide their own OpenRouter API key for routing\nrequests through OpenRouter's model marketplace.").meta({ title: "auth.CreateOpenRouterByokApiKeyRequest" });
var AuthDisableApiKeyRequestSchema = z305.z.object({
  api_key: PrefixedUuidSchema.describe("The API key to disable.")
}).describe("Request to disable an existing API key.\n\nOnce disabled, the API key can no longer be used for authentication.\nThis action is reversible only by creating a new key.").meta({ title: "auth.DisableApiKeyRequest" });
var AuthGetCreditsResponseSchema = z305.z.object({
  credits: z305.z.number().meta({ format: "double" }).describe("The current available credit balance."),
  total_credits_purchased: z305.z.number().meta({ format: "double" }).describe("The total amount of credits ever purchased."),
  total_credits_used: z305.z.number().meta({ format: "double" }).describe("The total amount of credits consumed by API usage.")
}).describe("Response containing the user's credit balance information.\n\nCredits are the billing unit for ObjectiveAI. This response provides\na complete view of the user's credit status.").meta({ title: "auth.GetCreditsResponse" });
var AuthGetOpenRouterByokApiKeyResponseSchema = z305.z.object({
  api_key: z305.z.string().nullable().describe("The OpenRouter API key, or `None` if not configured.").optional()
}).describe("Response containing the user's OpenRouter BYOK API key.").meta({ title: "auth.GetOpenRouterByokApiKeyResponse" });
var AuthListApiKeyItemSchema = z305.z.object({
  api_key: PrefixedUuidSchema.describe("The API key itself."),
  created: z305.z.string().meta({ format: "date-time" }).describe("The timestamp when the API key was created (RFC 3339 format)."),
  expires: z305.z.string().meta({ format: "date-time" }).nullable().describe("The timestamp when the API key expires, or `None` if it does not expire.").optional(),
  disabled: z305.z.string().meta({ format: "date-time" }).nullable().describe("The timestamp when the API key was disabled, or `None` if it is active.").optional(),
  name: z305.z.string().describe("The user-provided name of the API key."),
  description: z305.z.string().nullable().describe("The user-provided description of the API key, or `None` if not provided.").optional(),
  cost: z305.z.number().meta({ format: "double" }).describe("The total cost incurred by this API key.")
}).describe("An API key with metadata and accumulated cost information.\n\nThis extends [`ApiKeyWithMetadata`](super::ApiKeyWithMetadata) with\nthe total cost incurred by requests using this key.").meta({ title: "auth.ListApiKeyItem" });
var AuthListApiKeyResponseSchema = z305.z.object({
  data: z305.z.array(AuthListApiKeyItemSchema).describe("The list of API keys with their metadata and usage costs.")
}).describe("Response containing a list of API keys.").meta({ title: "auth.ListApiKeyResponse" });

// src/auth/http.ts
function authCreateApiKey(client, body, options) {
  return client.post_unary("/auth/keys", body, options);
}
function authCreateOpenrouterByokApiKey(client, body, options) {
  return client.post_unary(
    "/auth/keys/openrouter",
    body,
    options
  );
}
function authDisableApiKey(client, body, options) {
  return client.delete_unary("/auth/keys", body, options);
}
function authDeleteOpenrouterByokApiKey(client, options) {
  return client.delete_unary("/auth/keys/openrouter", void 0, options);
}
function authListApiKeys(client, options) {
  return client.get_unary("/auth/keys", void 0, options);
}
function authGetOpenrouterByokApiKey(client, options) {
  return client.get_unary(
    "/auth/keys/openrouter",
    void 0,
    options
  );
}
function authGetCredits(client, options) {
  return client.get_unary("/auth/credits", void 0, options);
}
var EnsembleEnsembleSchema = z305.z.object({
  id: z305.z.string().describe("The deterministic content-addressed ID (22-character base62 string)."),
  agents: z305.z.array(AgentWithFallbacksAndCountAgentAgentSchema).describe("The validated and deduplicated LLMs, sorted by full_id.")
}).describe("A validated Ensemble with its computed content-addressed ID.\n\nCreated by converting from [`EnsembleBase`] via [`TryFrom`]. The conversion:\n1. Validates and normalizes each agent\n2. Merges duplicate LLMs (by full_id) and sums their counts\n3. Sorts LLMs by full_id for deterministic ordering\n4. Computes the ensemble ID from the sorted (full_id, count) pairs\n\n# Constraints\n\n- Individual LLMs with `count: 0` are skipped\n- Total agent count (sum of all counts) must be between 1 and 128").meta({ title: "ensemble.Ensemble" });
var EnsembleEnsembleBaseSchema = z305.z.object({
  agents: z305.z.array(AgentWithFallbacksAndCountAgentAgentBaseSchema).describe("The LLMs in this ensemble, with optional counts and fallbacks.")
}).describe("The base configuration for an Ensemble (without computed ID).\n\nContains a list of agent configurations that will be validated, deduplicated,\nand sorted when converting to [`Ensemble`].").meta({ title: "ensemble.EnsembleBase" });
var EnsembleGetEnsembleSchema = z305.z.object({
  created: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Unix timestamp when this Ensemble was first used."),
  id: z305.z.string().describe("The deterministic content-addressed ID (22-character base62 string)."),
  agents: z305.z.array(AgentWithFallbacksAndCountAgentAgentSchema).describe("The validated and deduplicated LLMs, sorted by full_id.")
}).describe("Response containing a single Ensemble with creation timestamp.").meta({ title: "ensemble.GetEnsemble" });
var EnsembleListEnsembleItemSchema = z305.z.object({
  id: z305.z.string().describe("The unique content-addressed ID of the Ensemble.")
}).describe("Summary information for a listed Ensemble.").meta({ title: "ensemble.ListEnsembleItem" });

// src/ensemble/listEnsemble.ts
var EnsembleListEnsembleSchema = z305.z.object({
  data: z305.z.array(EnsembleListEnsembleItemSchema).describe("The list of Ensemble summaries.")
}).describe("Response containing a list of Ensembles.").meta({ title: "ensemble.ListEnsemble" });
var EnsembleUsageEnsembleSchema = z305.z.object({
  requests: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Total number of requests made with this Ensemble."),
  completion_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Total completion tokens generated across all agents."),
  prompt_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Total prompt tokens processed across all agents."),
  total_cost: z305.z.number().meta({ format: "double" }).describe("Total cost incurred.")
}).describe("Usage statistics for an Ensemble.").meta({ title: "ensemble.UsageEnsemble" });

// src/ensemble/http.ts
function ensembleListEnsembles(client, options) {
  return client.get_unary("/ensembles", void 0, options);
}
function ensembleGetEnsemble(client, ensembleId, options) {
  return client.get_unary(
    `/ensembles/${ensembleId}`,
    void 0,
    options
  );
}
function ensembleGetEnsembleUsage(client, ensembleId, options) {
  return client.get_unary(
    `/ensembles/${ensembleId}/usage`,
    void 0,
    options
  );
}
var FunctionsExpressionAnyOfInputSchemaSchema = z305.z.object({
  anyOf: z305.z.array(z305.z.lazy(() => FunctionsExpressionInputSchemaSchema)).describe("The possible schemas that the input can match.")
}).describe("Schema for a union of possible types - input must match at least one.").meta({ title: "functions.expression.AnyOfInputSchema" });
var FunctionsExpressionArrayInputSchemaTypeSchema = z305.z.literal("array").meta({ title: "functions.expression.ArrayInputSchemaType" });

// src/functions/expression/arrayInputSchema.ts
var FunctionsExpressionArrayInputSchemaSchema = z305.z.object({
  type: FunctionsExpressionArrayInputSchemaTypeSchema,
  description: z305.z.string().nullable().describe("Human-readable description of the array.").optional(),
  minItems: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Minimum number of items required.").optional(),
  maxItems: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Maximum number of items allowed.").optional(),
  items: z305.z.lazy(() => FunctionsExpressionInputSchemaSchema).describe("Schema for each item in the array.")
}).describe("Schema for an array input.").meta({ title: "functions.expression.ArrayInputSchema" });
var FunctionsExpressionAudioInputSchemaTypeSchema = z305.z.literal("audio").meta({ title: "functions.expression.AudioInputSchemaType" });

// src/functions/expression/audioInputSchema.ts
var FunctionsExpressionAudioInputSchemaSchema = z305.z.object({
  type: FunctionsExpressionAudioInputSchemaTypeSchema,
  description: z305.z.string().nullable().describe("Human-readable description of the expected audio.").optional()
}).describe("Schema for an audio input.").meta({ title: "functions.expression.AudioInputSchema" });
var FunctionsExpressionBooleanInputSchemaTypeSchema = z305.z.literal("boolean").meta({ title: "functions.expression.BooleanInputSchemaType" });

// src/functions/expression/booleanInputSchema.ts
var FunctionsExpressionBooleanInputSchemaSchema = z305.z.object({
  type: FunctionsExpressionBooleanInputSchemaTypeSchema,
  description: z305.z.string().nullable().describe("Human-readable description of the boolean.").optional()
}).describe("Schema for a boolean input.").meta({ title: "functions.expression.BooleanInputSchema" });
var FunctionsExpressionFileInputSchemaTypeSchema = z305.z.literal("file").meta({ title: "functions.expression.FileInputSchemaType" });

// src/functions/expression/fileInputSchema.ts
var FunctionsExpressionFileInputSchemaSchema = z305.z.object({
  type: FunctionsExpressionFileInputSchemaTypeSchema,
  description: z305.z.string().nullable().describe("Human-readable description of the expected file.").optional()
}).describe("Schema for a file input.").meta({ title: "functions.expression.FileInputSchema" });
var FunctionsExpressionImageInputSchemaTypeSchema = z305.z.literal("image").meta({ title: "functions.expression.ImageInputSchemaType" });

// src/functions/expression/imageInputSchema.ts
var FunctionsExpressionImageInputSchemaSchema = z305.z.object({
  type: FunctionsExpressionImageInputSchemaTypeSchema,
  description: z305.z.string().nullable().describe("Human-readable description of the expected image.").optional()
}).describe("Schema for an image input (URL or base64-encoded).").meta({ title: "functions.expression.ImageInputSchema" });
var FunctionsExpressionIntegerInputSchemaTypeSchema = z305.z.literal("integer").meta({ title: "functions.expression.IntegerInputSchemaType" });

// src/functions/expression/integerInputSchema.ts
var FunctionsExpressionIntegerInputSchemaSchema = z305.z.object({
  type: FunctionsExpressionIntegerInputSchemaTypeSchema,
  description: z305.z.string().nullable().describe("Human-readable description of the integer.").optional(),
  minimum: z305.z.number().int().meta({ format: "int64" }).nullable().describe("Minimum allowed value (inclusive).").optional(),
  maximum: z305.z.number().int().meta({ format: "int64" }).nullable().describe("Maximum allowed value (inclusive).").optional()
}).describe("Schema for an integer input.").meta({ title: "functions.expression.IntegerInputSchema" });
var FunctionsExpressionNumberInputSchemaTypeSchema = z305.z.literal("number").meta({ title: "functions.expression.NumberInputSchemaType" });

// src/functions/expression/numberInputSchema.ts
var FunctionsExpressionNumberInputSchemaSchema = z305.z.object({
  type: FunctionsExpressionNumberInputSchemaTypeSchema,
  description: z305.z.string().nullable().describe("Human-readable description of the number.").optional(),
  minimum: z305.z.number().meta({ format: "double" }).nullable().describe("Minimum allowed value (inclusive).").optional(),
  maximum: z305.z.number().meta({ format: "double" }).nullable().describe("Maximum allowed value (inclusive).").optional()
}).describe("Schema for a floating-point number input.").meta({ title: "functions.expression.NumberInputSchema" });
var FunctionsExpressionStringInputSchemaTypeSchema = z305.z.literal("string").meta({ title: "functions.expression.StringInputSchemaType" });

// src/functions/expression/stringInputSchema.ts
var FunctionsExpressionStringInputSchemaSchema = z305.z.object({
  type: FunctionsExpressionStringInputSchemaTypeSchema,
  description: z305.z.string().nullable().describe("Human-readable description of the string.").optional(),
  enum: z305.z.array(z305.z.string()).nullable().describe("If provided, the string must be one of these values.").optional()
}).describe("Schema for a string input.").meta({ title: "functions.expression.StringInputSchema" });
var FunctionsExpressionVideoInputSchemaTypeSchema = z305.z.literal("video").meta({ title: "functions.expression.VideoInputSchemaType" });

// src/functions/expression/videoInputSchema.ts
var FunctionsExpressionVideoInputSchemaSchema = z305.z.object({
  type: FunctionsExpressionVideoInputSchemaTypeSchema,
  description: z305.z.string().nullable().describe("Human-readable description of the expected video.").optional()
}).describe("Schema for a video input (URL or base64-encoded).").meta({ title: "functions.expression.VideoInputSchema" });

// src/functions/expression/inputSchema.ts
var FunctionsExpressionInputSchemaSchema = z305.z.union([z305.z.lazy(() => FunctionsExpressionAnyOfInputSchemaSchema).describe("A union of schemas - input must match at least one."), z305.z.lazy(() => FunctionsExpressionObjectInputSchemaSchema).describe("An object with named properties."), z305.z.lazy(() => FunctionsExpressionArrayInputSchemaSchema).describe("An array of items."), FunctionsExpressionStringInputSchemaSchema.describe("A string value."), FunctionsExpressionIntegerInputSchemaSchema.describe("An integer value."), FunctionsExpressionNumberInputSchemaSchema.describe("A floating-point number."), FunctionsExpressionBooleanInputSchemaSchema.describe("A boolean value."), FunctionsExpressionImageInputSchemaSchema.describe("An image (URL or base64)."), FunctionsExpressionAudioInputSchemaSchema.describe("Audio content."), FunctionsExpressionVideoInputSchemaSchema.describe("Video content."), FunctionsExpressionFileInputSchemaSchema.describe("A file.")]).describe("Schema for validating Function input.\n\nDefines the expected structure and constraints for input data.\nUsed by remote Functions to document and validate their inputs.").meta({ title: "functions.expression.InputSchema" });
var FunctionsExpressionObjectInputSchemaTypeSchema = z305.z.literal("object").meta({ title: "functions.expression.ObjectInputSchemaType" });

// src/functions/expression/objectInputSchema.ts
var FunctionsExpressionObjectInputSchemaSchema = z305.z.object({
  type: FunctionsExpressionObjectInputSchemaTypeSchema,
  description: z305.z.string().nullable().describe("Human-readable description of the object.").optional(),
  properties: z305.z.record(z305.z.string(), z305.z.lazy(() => FunctionsExpressionInputSchemaSchema)).describe("Schema for each property in the object."),
  required: z305.z.array(z305.z.string()).nullable().describe("List of property names that must be present.").optional()
}).describe("Schema for an object input with named properties.").meta({ title: "functions.expression.ObjectInputSchema" });

// src/functions/alpha_scalar/placeholderScalarFunctionTaskExpression.ts
var FunctionsAlphaScalarPlaceholderScalarFunctionTaskExpressionSchema = z305.z.object({
  depth: z305.z.number().int().min(0).meta({ format: "uint64" }),
  min_branch_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  max_branch_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  min_leaf_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  max_leaf_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  name: z305.z.string(),
  spec: z305.z.string(),
  input_schema: FunctionsExpressionObjectInputSchemaSchema,
  skip: FunctionsExpressionExpressionSchema.nullable().optional(),
  input: FunctionsExpressionExpressionSchema
}).meta({ title: "functions.alpha_scalar.PlaceholderScalarFunctionTaskExpression" });
var FunctionsRemoteSchema = z305.z.union([z305.z.literal("github").describe("GitHub repository."), z305.z.literal("filesystem").describe("Local filesystem."), z305.z.literal("mock").describe("Mock (for testing).")]).describe("The remote source where a function or profile is hosted.").meta({ title: "functions.Remote" });

// src/functions/alpha_scalar/scalarFunctionTaskExpression.ts
var FunctionsAlphaScalarScalarFunctionTaskExpressionSchema = z305.z.object({
  remote: FunctionsRemoteSchema,
  owner: z305.z.string(),
  repository: z305.z.string(),
  commit: z305.z.string(),
  skip: FunctionsExpressionExpressionSchema.nullable().optional(),
  input: FunctionsExpressionExpressionSchema
}).meta({ title: "functions.alpha_scalar.ScalarFunctionTaskExpression" });

// src/functions/alpha_scalar/branchTaskExpression.ts
var FunctionsAlphaScalarBranchTaskExpressionSchema = z305.z.union([FunctionsAlphaScalarScalarFunctionTaskExpressionSchema.extend({
  type: z305.z.literal("alpha.scalar.function")
}), FunctionsAlphaScalarPlaceholderScalarFunctionTaskExpressionSchema.extend({
  type: z305.z.literal("placeholder.alpha.scalar.function")
})]).meta({ title: "functions.alpha_scalar.BranchTaskExpression" });
var FunctionsAlphaScalarVectorCompletionTaskExpressionSchema = z305.z.object({
  skip: FunctionsExpressionExpressionSchema.nullable().optional(),
  messages: FunctionsExpressionExpressionSchema,
  responses: z305.z.array(AgentCompletionsMessageRichContentSchema)
}).meta({ title: "functions.alpha_scalar.VectorCompletionTaskExpression" });

// src/functions/alpha_scalar/leafTaskExpression.ts
var FunctionsAlphaScalarLeafTaskExpressionSchema = z305.z.union([FunctionsAlphaScalarVectorCompletionTaskExpressionSchema.extend({
  type: z305.z.literal("vector.completion")
})]).meta({ title: "functions.alpha_scalar.LeafTaskExpression" });

// src/functions/alpha_scalar/inlineFunction.ts
var FunctionsAlphaScalarInlineFunctionSchema = z305.z.union([z305.z.object({
  tasks: z305.z.array(FunctionsAlphaScalarBranchTaskExpressionSchema),
  type: z305.z.literal("alpha.scalar.branch.function")
}), z305.z.object({
  tasks: z305.z.array(FunctionsAlphaScalarLeafTaskExpressionSchema),
  type: z305.z.literal("alpha.scalar.leaf.function")
})]).meta({ title: "functions.alpha_scalar.InlineFunction" });
var FunctionsAlphaScalarPartialPlaceholderScalarFunctionTaskExpressionSchema = z305.z.object({
  spec: z305.z.string(),
  input_schema: FunctionsExpressionObjectInputSchemaSchema,
  skip: FunctionsExpressionExpressionSchema.nullable().optional(),
  input: FunctionsExpressionExpressionSchema
}).meta({ title: "functions.alpha_scalar.PartialPlaceholderScalarFunctionTaskExpression" });

// src/functions/alpha_scalar/partialPlaceholderBranchTaskExpression.ts
var FunctionsAlphaScalarPartialPlaceholderBranchTaskExpressionSchema = z305.z.union([FunctionsAlphaScalarPartialPlaceholderScalarFunctionTaskExpressionSchema.extend({
  type: z305.z.literal("placeholder.alpha.scalar.function")
})]).meta({ title: "functions.alpha_scalar.PartialPlaceholderBranchTaskExpression" });
var FunctionsAlphaScalarRemoteFunctionSchema = z305.z.union([z305.z.object({
  description: z305.z.string(),
  input_schema: FunctionsExpressionObjectInputSchemaSchema,
  tasks: z305.z.array(FunctionsAlphaScalarBranchTaskExpressionSchema),
  type: z305.z.literal("alpha.scalar.branch.function")
}), z305.z.object({
  description: z305.z.string(),
  input_schema: FunctionsExpressionObjectInputSchemaSchema,
  tasks: z305.z.array(FunctionsAlphaScalarLeafTaskExpressionSchema),
  type: z305.z.literal("alpha.scalar.leaf.function")
})]).meta({ title: "functions.alpha_scalar.RemoteFunction" });
var FunctionsAlphaVectorExpressionVectorFunctionInputSchemaSchema = z305.z.object({
  context: FunctionsExpressionObjectInputSchemaSchema.nullable().optional(),
  items: FunctionsExpressionInputSchemaSchema
}).meta({ title: "functions.alpha_vector.expression.VectorFunctionInputSchema" });
var FunctionsExpressionInputValueSchema = z305.z.union([AgentCompletionsMessageRichContentPartSchema.describe("Rich content (image, audio, video, file)."), z305.z.record(z305.z.string(), z305.z.lazy(() => FunctionsExpressionInputValueSchema)).describe("An object with string keys."), z305.z.array(z305.z.lazy(() => FunctionsExpressionInputValueSchema)).describe("An array of values."), z305.z.string().describe("A string value."), z305.z.number().int().meta({ format: "int64" }).describe("An integer value."), z305.z.number().meta({ format: "double" }).describe("A floating-point number."), z305.z.boolean().describe("A boolean value.")]).describe("A concrete input value (post-compilation).\n\nRepresents any JSON-like value that can be passed to a Function,\nincluding rich content types (images, audio, video, files).").meta({ title: "functions.expression.InputValue" });

// src/functions/alpha_vector/expression/vectorFunctionInputValue.ts
var FunctionsAlphaVectorExpressionVectorFunctionInputValueSchema = z305.z.object({
  context: z305.z.record(z305.z.string(), FunctionsExpressionInputValueSchema).nullable().optional(),
  items: z305.z.array(FunctionsExpressionInputValueSchema)
}).meta({ title: "functions.alpha_vector.expression.VectorFunctionInputValue" });
var FunctionsAlphaVectorExpressionVectorFunctionInputValueExpressionSchema = z305.z.object({
  context: FunctionsExpressionExpressionSchema.nullable().optional(),
  items: FunctionsExpressionExpressionSchema
}).meta({ title: "functions.alpha_vector.expression.VectorFunctionInputValueExpression" });
var FunctionsAlphaVectorPlaceholderScalarFunctionTaskExpressionSchema = z305.z.object({
  depth: z305.z.number().int().min(0).meta({ format: "uint64" }),
  min_branch_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  max_branch_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  min_leaf_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  max_leaf_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  name: z305.z.string(),
  spec: z305.z.string(),
  input_schema: FunctionsExpressionObjectInputSchemaSchema,
  skip: FunctionsExpressionExpressionSchema.nullable().optional(),
  input: FunctionsExpressionExpressionSchema
}).meta({ title: "functions.alpha_vector.PlaceholderScalarFunctionTaskExpression" });
var FunctionsAlphaVectorPlaceholderVectorFunctionTaskExpressionSchema = z305.z.object({
  depth: z305.z.number().int().min(0).meta({ format: "uint64" }),
  min_branch_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  max_branch_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  min_leaf_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  max_leaf_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  name: z305.z.string(),
  spec: z305.z.string(),
  input_schema: FunctionsAlphaVectorExpressionVectorFunctionInputSchemaSchema,
  skip: FunctionsExpressionExpressionSchema.nullable().optional(),
  input: FunctionsAlphaVectorExpressionVectorFunctionInputValueExpressionSchema
}).meta({ title: "functions.alpha_vector.PlaceholderVectorFunctionTaskExpression" });
var FunctionsAlphaVectorScalarFunctionTaskExpressionSchema = z305.z.object({
  remote: FunctionsRemoteSchema,
  owner: z305.z.string(),
  repository: z305.z.string(),
  commit: z305.z.string(),
  skip: FunctionsExpressionExpressionSchema.nullable().optional(),
  input: FunctionsExpressionExpressionSchema
}).meta({ title: "functions.alpha_vector.ScalarFunctionTaskExpression" });
var FunctionsAlphaVectorVectorFunctionTaskExpressionSchema = z305.z.object({
  remote: FunctionsRemoteSchema,
  owner: z305.z.string(),
  repository: z305.z.string(),
  commit: z305.z.string(),
  skip: FunctionsExpressionExpressionSchema.nullable().optional(),
  input: FunctionsAlphaVectorExpressionVectorFunctionInputValueExpressionSchema
}).meta({ title: "functions.alpha_vector.VectorFunctionTaskExpression" });

// src/functions/alpha_vector/branchTaskExpression.ts
var FunctionsAlphaVectorBranchTaskExpressionSchema = z305.z.union([FunctionsAlphaVectorScalarFunctionTaskExpressionSchema.extend({
  type: z305.z.literal("alpha.scalar.function")
}), FunctionsAlphaVectorVectorFunctionTaskExpressionSchema.extend({
  type: z305.z.literal("alpha.vector.function")
}), FunctionsAlphaVectorPlaceholderScalarFunctionTaskExpressionSchema.extend({
  type: z305.z.literal("placeholder.alpha.scalar.function")
}), FunctionsAlphaVectorPlaceholderVectorFunctionTaskExpressionSchema.extend({
  type: z305.z.literal("placeholder.alpha.vector.function")
})]).meta({ title: "functions.alpha_vector.BranchTaskExpression" });
var FunctionsAlphaVectorVectorCompletionTaskExpressionSchema = z305.z.object({
  skip: FunctionsExpressionExpressionSchema.nullable().optional(),
  messages: FunctionsExpressionExpressionSchema,
  responses: FunctionsExpressionExpressionSchema
}).meta({ title: "functions.alpha_vector.VectorCompletionTaskExpression" });

// src/functions/alpha_vector/leafTaskExpression.ts
var FunctionsAlphaVectorLeafTaskExpressionSchema = z305.z.union([FunctionsAlphaVectorVectorCompletionTaskExpressionSchema.extend({
  type: z305.z.literal("vector.completion")
})]).meta({ title: "functions.alpha_vector.LeafTaskExpression" });

// src/functions/alpha_vector/inlineFunction.ts
var FunctionsAlphaVectorInlineFunctionSchema = z305.z.union([z305.z.object({
  tasks: z305.z.array(FunctionsAlphaVectorBranchTaskExpressionSchema),
  type: z305.z.literal("alpha.vector.branch.function")
}), z305.z.object({
  tasks: z305.z.array(FunctionsAlphaVectorLeafTaskExpressionSchema),
  type: z305.z.literal("alpha.vector.leaf.function")
})]).meta({ title: "functions.alpha_vector.InlineFunction" });
var FunctionsAlphaVectorPartialPlaceholderScalarFunctionTaskExpressionSchema = z305.z.object({
  spec: z305.z.string(),
  input_schema: FunctionsExpressionObjectInputSchemaSchema,
  skip: FunctionsExpressionExpressionSchema.nullable().optional(),
  input: FunctionsExpressionExpressionSchema
}).meta({ title: "functions.alpha_vector.PartialPlaceholderScalarFunctionTaskExpression" });
var FunctionsAlphaVectorPartialPlaceholderVectorFunctionTaskExpressionSchema = z305.z.object({
  spec: z305.z.string(),
  input_schema: FunctionsAlphaVectorExpressionVectorFunctionInputSchemaSchema,
  skip: FunctionsExpressionExpressionSchema.nullable().optional(),
  input: FunctionsAlphaVectorExpressionVectorFunctionInputValueExpressionSchema
}).meta({ title: "functions.alpha_vector.PartialPlaceholderVectorFunctionTaskExpression" });

// src/functions/alpha_vector/partialPlaceholderBranchTaskExpression.ts
var FunctionsAlphaVectorPartialPlaceholderBranchTaskExpressionSchema = z305.z.union([FunctionsAlphaVectorPartialPlaceholderScalarFunctionTaskExpressionSchema.extend({
  type: z305.z.literal("placeholder.alpha.scalar.function")
}), FunctionsAlphaVectorPartialPlaceholderVectorFunctionTaskExpressionSchema.extend({
  type: z305.z.literal("placeholder.alpha.vector.function")
})]).meta({ title: "functions.alpha_vector.PartialPlaceholderBranchTaskExpression" });
var FunctionsAlphaVectorRemoteFunctionSchema = z305.z.union([z305.z.object({
  description: z305.z.string(),
  input_schema: FunctionsAlphaVectorExpressionVectorFunctionInputSchemaSchema,
  tasks: z305.z.array(FunctionsAlphaVectorBranchTaskExpressionSchema),
  type: z305.z.literal("alpha.vector.branch.function")
}), z305.z.object({
  description: z305.z.string(),
  input_schema: FunctionsAlphaVectorExpressionVectorFunctionInputSchemaSchema,
  tasks: z305.z.array(FunctionsAlphaVectorLeafTaskExpressionSchema),
  type: z305.z.literal("alpha.vector.leaf.function")
})]).meta({ title: "functions.alpha_vector.RemoteFunction" });
var FunctionsCheckScalarFieldsValidationSchema = z305.z.object({
  input_schema: FunctionsExpressionInputSchemaSchema
}).describe("The fields needed to validate a scalar function's input behavior.").meta({ title: "functions.check.ScalarFieldsValidation" });
var FunctionsCheckVectorFieldsValidationSchema = z305.z.object({
  input_schema: FunctionsExpressionInputSchemaSchema,
  output_length: FunctionsExpressionExpressionSchema,
  input_split: FunctionsExpressionExpressionSchema,
  input_merge: FunctionsExpressionExpressionSchema
}).describe("The 4 fields needed to validate a vector function's split/merge behavior.").meta({ title: "functions.check.VectorFieldsValidation" });
var FunctionsExecutionsRequestReasoningSchema = z305.z.object({
  agent: AgentCompletionsRequestAgentSchema.describe("The primary agent to use for generating reasoning summaries."),
  agents: z305.z.array(AgentCompletionsRequestAgentSchema).nullable().describe("Fallback agents tried in order if the primary is rate-limited or errors.").optional()
}).describe("Configuration for generating reasoning summaries during execution.\n\nWhen enabled, an LLM summarizes the execution's reasoning process.").meta({ title: "functions.executions.request.Reasoning" });
var FunctionsExecutionsRequestStrategySchema = z305.z.union([z305.z.object({
  type: z305.z.literal("default")
}).describe("Scalar or Vector"), z305.z.object({
  pool: z305.z.number().int().min(0).meta({ format: "uint" }).nullable().describe("How many vector responses for each execution").optional(),
  rounds: z305.z.number().int().min(0).meta({ format: "uint" }).nullable().describe("How many sequential rounds of comparison").optional(),
  type: z305.z.literal("swiss_system")
}).describe("Vector")]).meta({ title: "functions.executions.request.Strategy" });
var FunctionsPlaceholderScalarFunctionTaskExpressionSchema = z305.z.object({
  input_schema: FunctionsExpressionInputSchemaSchema.describe("JSON Schema defining the expected input structure."),
  skip: FunctionsExpressionExpressionSchema.nullable().describe("If this expression evaluates to true, skip the task. Receives: `input`.").optional(),
  map: FunctionsExpressionExpressionSchema.nullable().describe("Expression that evaluates to the number of mapped task instances.\nEach instance receives `map` as an integer index (0-based).").optional(),
  input: FunctionsExpressionWithExpressionFunctionsExpressionInputValueExpressionSchema.describe("Expression for the input to pass to the placeholder function.\nReceives: `input`, `map` (if mapped)."),
  output: FunctionsExpressionExpressionSchema.describe("Expression to transform the fixed 0.5 output.\nReceives: `input`, `output` as `Scalar(0.5)`.")
}).describe("Expression for a placeholder scalar function task (pre-compilation).\n\nLike [`ScalarFunctionTaskExpression`] but without owner/repository/commit.\nAlways produces a fixed output of 0.5.").meta({ title: "functions.PlaceholderScalarFunctionTaskExpression" });
var FunctionsPlaceholderVectorFunctionTaskExpressionSchema = z305.z.object({
  input_schema: FunctionsExpressionInputSchemaSchema.describe("JSON Schema defining the expected input structure."),
  output_length: FunctionsExpressionExpressionSchema.describe("Expression computing the expected output vector length.\nReceives: `input`."),
  input_split: FunctionsExpressionExpressionSchema.describe("Expression transforming input into sub-inputs for swiss system.\nReceives: `input`."),
  input_merge: FunctionsExpressionExpressionSchema.describe("Expression merging sub-inputs back into one input.\nReceives: `input` (as an array)."),
  skip: FunctionsExpressionExpressionSchema.nullable().describe("If this expression evaluates to true, skip the task. Receives: `input`.").optional(),
  map: FunctionsExpressionExpressionSchema.nullable().describe("Expression that evaluates to the number of mapped task instances.\nEach instance receives `map` as an integer index (0-based).").optional(),
  input: FunctionsExpressionWithExpressionFunctionsExpressionInputValueExpressionSchema.describe("Expression for the input to pass to the placeholder function.\nReceives: `input`, `map` (if mapped)."),
  output: FunctionsExpressionExpressionSchema.describe("Expression to transform the equalized vector output.\nReceives: `input`, `output` as `Vector(equalized)`.")
}).describe("Expression for a placeholder vector function task (pre-compilation).\n\nLike [`VectorFunctionTaskExpression`] but without owner/repository/commit.\nAlways produces an equalized vector of length `output_length`.").meta({ title: "functions.PlaceholderVectorFunctionTaskExpression" });
var FunctionsScalarFunctionTaskExpressionSchema = z305.z.object({
  remote: FunctionsRemoteSchema.describe("The remote source where the function is hosted."),
  owner: z305.z.string().describe("Repository owner."),
  repository: z305.z.string().describe("Repository name."),
  commit: z305.z.string().describe("Git commit SHA for the function version."),
  skip: FunctionsExpressionExpressionSchema.nullable().describe("If this expression evaluates to true, skip the task. Receives: `input`.").optional(),
  map: FunctionsExpressionExpressionSchema.nullable().describe("Expression that evaluates to the number of mapped task instances.\nEach instance receives `map` as an integer index (0-based).").optional(),
  input: FunctionsExpressionWithExpressionFunctionsExpressionInputValueExpressionSchema.describe("Expression for the input to pass to the function.\nReceives: `input`, `map` (if mapped)."),
  output: FunctionsExpressionExpressionSchema.describe("Expression to transform the task result into a valid function output.\n\nReceives `output` which is one of 4 variants:\n- `Scalar(Decimal)` - a single score\n- `Vector(Vec<Decimal>)` - a vector of scores\n- `Vectors(Vec<Vec<Decimal>>)` - multiple vectors (from mapped tasks)\n- `Err(Value)` - an error\n\nThe expression must return a `TaskOutputOwned` that is valid for the parent function's type:\n- For scalar functions: must return `Scalar(value)` where value is in [0, 1]\n- For vector functions: must return `Vector(values)` where values sum to ~1 and match the expected length\n\nThe function's final output is computed as a weighted average of all task outputs using\nprofile weights. If a function has only one task, that task's output becomes the function's\noutput directly.")
}).describe("Expression for a task that calls a scalar function (pre-compilation).").meta({ title: "functions.ScalarFunctionTaskExpression" });
var FunctionsVectorCompletionTaskExpressionSchema = z305.z.object({
  skip: FunctionsExpressionExpressionSchema.nullable().describe("If this expression evaluates to true, skip the task. Receives: `input`.").optional(),
  map: FunctionsExpressionExpressionSchema.nullable().describe("Expression that evaluates to the number of mapped task instances.\nEach instance receives `map` as an integer index (0-based).").optional(),
  messages: FunctionsExpressionWithExpressionArrayOfFunctionsExpressionWithExpressionAgentCompletionsMessageMessageExpressionSchema.describe("Expression for the conversation messages (the prompt).\nReceives: `input`, `map` (if mapped)."),
  responses: FunctionsExpressionWithExpressionArrayOfFunctionsExpressionWithExpressionAgentCompletionsMessageRichContentExpressionSchema.describe("Expression for the possible responses the LLMs can vote for.\nReceives: `input`, `map` (if mapped)."),
  output: FunctionsExpressionExpressionSchema.describe("Expression to transform the task result into a valid function output.\n\nReceives `output` as the task's raw result (typically `Vector(scores)`).\n\nThe expression must return a `TaskOutputOwned` that is valid for the parent function's type:\n- For scalar functions: must return `Scalar(value)` where value is in [0, 1]\n- For vector functions: must return `Vector(values)` where values sum to ~1 and match the expected length\n\nThe function's final output is computed as a weighted average of all task outputs using\nprofile weights. If a function has only one task, that task's output becomes the function's\noutput directly.")
}).describe("Expression for a task that runs a vector completion (pre-compilation).").meta({ title: "functions.VectorCompletionTaskExpression" });
var FunctionsVectorFunctionTaskExpressionSchema = z305.z.object({
  remote: FunctionsRemoteSchema.describe("The remote source where the function is hosted."),
  owner: z305.z.string().describe("Repository owner."),
  repository: z305.z.string().describe("Repository name."),
  commit: z305.z.string().describe("Git commit SHA for the function version."),
  skip: FunctionsExpressionExpressionSchema.nullable().describe("If this expression evaluates to true, skip the task. Receives: `input`.").optional(),
  map: FunctionsExpressionExpressionSchema.nullable().describe("Expression that evaluates to the number of mapped task instances.\nEach instance receives `map` as an integer index (0-based).").optional(),
  input: FunctionsExpressionWithExpressionFunctionsExpressionInputValueExpressionSchema.describe("Expression for the input to pass to the function.\nReceives: `input`, `map` (if mapped)."),
  output: FunctionsExpressionExpressionSchema.describe("Expression to transform the task result into a valid function output.\n\nReceives `output` which is one of 4 variants:\n- `Scalar(Decimal)` - a single score\n- `Vector(Vec<Decimal>)` - a vector of scores\n- `Vectors(Vec<Vec<Decimal>>)` - multiple vectors (from mapped tasks)\n- `Err(Value)` - an error\n\nThe expression must return a `TaskOutputOwned` that is valid for the parent function's type:\n- For scalar functions: must return `Scalar(value)` where value is in [0, 1]\n- For vector functions: must return `Vector(values)` where values sum to ~1 and match the expected length\n\nThe function's final output is computed as a weighted average of all task outputs using\nprofile weights. If a function has only one task, that task's output becomes the function's\noutput directly.")
}).describe("Expression for a task that calls a vector function (pre-compilation).").meta({ title: "functions.VectorFunctionTaskExpression" });

// src/functions/taskExpression.ts
var FunctionsTaskExpressionSchema = z305.z.union([FunctionsScalarFunctionTaskExpressionSchema.extend({
  type: z305.z.literal("scalar.function")
}), FunctionsVectorFunctionTaskExpressionSchema.extend({
  type: z305.z.literal("vector.function")
}), FunctionsVectorCompletionTaskExpressionSchema.extend({
  type: z305.z.literal("vector.completion")
}), FunctionsPlaceholderScalarFunctionTaskExpressionSchema.extend({
  type: z305.z.literal("placeholder.scalar.function")
}), FunctionsPlaceholderVectorFunctionTaskExpressionSchema.extend({
  type: z305.z.literal("placeholder.vector.function")
})]).describe("A task definition with expressions (pre-compilation).\n\nTask expressions contain dynamic fields (JMESPath or Starlark) that are\nresolved against input data during compilation. Use [`compile`](Self::compile)\nto produce a concrete [`Task`].").meta({ title: "functions.TaskExpression" });

// src/functions/inlineFunction.ts
var FunctionsInlineFunctionSchema = z305.z.union([z305.z.object({
  tasks: z305.z.array(FunctionsTaskExpressionSchema).describe("The list of tasks to execute. Tasks with a `map` expression are\nexpanded into multiple instances. Each instance is compiled with\n`map` set to the current integer index.\nReceives: `input`, `map` (if mapped)."),
  type: z305.z.literal("scalar.function")
}).describe("Produces a single score in [0, 1]."), z305.z.object({
  tasks: z305.z.array(FunctionsTaskExpressionSchema).describe("The list of tasks to execute. Tasks with a `map` expression are\nexpanded into multiple instances. Each instance is compiled with\n`map` set to the current integer index.\nReceives: `input`, `map` (if mapped)."),
  input_split: FunctionsExpressionExpressionSchema.nullable().describe("Expression transforming input into an input array of the output_length\nWhen the Function is executed with any input from the array,\nThe output_length should be 1.\nReceives: `input`.\nOnly required if the request uses a strategy that needs input splitting.").optional(),
  input_merge: FunctionsExpressionExpressionSchema.nullable().describe("Expression transforming an array of inputs computed by `input_split`\ninto a single Input object for the Function.\nReceives: `input` (as an array).\nOnly required if the request uses a strategy that needs input splitting.").optional(),
  type: z305.z.literal("vector.function")
}).describe("Produces a vector of scores that sums to 1.")]).describe("An inline function definition without metadata.\n\nUsed when embedding function logic directly in requests rather than\nreferencing a remote function. Lacks description and input\nschema fields.").meta({ title: "functions.InlineFunction" });
var VectorCompletionsRequestEnsembleSchema = z305.z.union([z305.z.string().describe("Reference an existing Ensemble by its ID."), EnsembleEnsembleBaseSchema.describe("Provide an inline Ensemble definition.")]).describe('Specifies which Ensemble to use for a vector completion.\n\nEnsembles can be referenced by ID or provided inline. The untagged\ndeserialization allows either a string ID or a full [`EnsembleBase`]\ndefinition in JSON.\n\n# Examples\n\nBy ID:\n```json\n"ensemble": "ens_abc123"\n```\n\nInline definition:\n```json\n"ensemble": {\n  "llms": [\n    {"model": "openai/gpt-4o", "output_mode": "json_schema", "count": 2},\n    {"model": "google/gemini-3.0-pro", "output_mode": "tool_call"}\n  ]\n}\n```\n\n[`EnsembleBase`]: crate::ensemble::EnsembleBase').meta({ title: "vector.completions.request.Ensemble" });
var VectorCompletionsRequestProfileEntrySchema = z305.z.object({
  weight: z305.z.number().meta({ format: "double" }).describe("The weight for this agent in the ensemble. Must be in [0, 1]."),
  invert: z305.z.boolean().nullable().describe("If true, invert this agent's vote distribution before combining.\n\nWhen omitted or false, the vote distribution is used as-is.").optional()
}).describe("An entry in a profile with an explicit weight and optional invert flag.").meta({ title: "vector.completions.request.ProfileEntry" });

// src/vector/completions/request/profile.ts
var VectorCompletionsRequestProfileSchema = z305.z.union([z305.z.array(z305.z.number().meta({ format: "double" })).describe("Simple vector of decimal weights."), z305.z.array(VectorCompletionsRequestProfileEntrySchema).describe("Vector of entries with optional invert flags.")]).describe("Profile weights for a vector completion.\n\nPreviously this was a simple `Vec<Decimal>`. To support per-agent inversion\nwhile remaining backwards compatible, the field is now an untagged enum:\n\n- `Weights(Vec<Decimal>)` - legacy representation (no inversion)\n- `Entries(Vec<ProfileEntry>)` - weights with optional per-agent `invert`").meta({ title: "vector.completions.request.Profile" });

// src/functions/inlineAutoProfile.ts
var FunctionsInlineAutoProfileSchema = z305.z.object({
  ensemble: VectorCompletionsRequestEnsembleSchema.describe("The ensemble to use for all vector completion tasks."),
  profile: VectorCompletionsRequestProfileSchema.describe("Weights for each agent in the ensemble.")
}).describe("An inline auto profile definition without metadata.\n\nApplies a single ensemble and weights to every vector completion task\nin the function, with equal task weights.").meta({ title: "functions.InlineAutoProfile" });
var FunctionsTaskProfileSchema = z305.z.union([z305.z.object({
  remote: FunctionsRemoteSchema.describe("The remote source where the profile is hosted."),
  owner: z305.z.string().describe("Repository owner."),
  repository: z305.z.string().describe("Repository name."),
  commit: z305.z.string().nullable().describe("Git commit SHA. Highly recommended for remote profiles to\nensure compatibility if the referenced profile's shape changes.").optional()
}).describe("Profile for a nested function task (references another profile)."), z305.z.lazy(() => FunctionsInlineProfileSchema).describe("Inline profile for a task (tasks-based or auto)."), z305.z.record(z305.z.string(), JsonValueSchema).describe("Placeholder task \u2014 no configuration needed, output is fixed.")]).describe("Configuration for a single task within a Profile.\n\nEach variant corresponds to a task type in the Function definition.").meta({ title: "functions.TaskProfile" });

// src/functions/inlineTasksProfile.ts
var FunctionsInlineTasksProfileSchema = z305.z.object({
  tasks: z305.z.array(z305.z.lazy(() => FunctionsTaskProfileSchema)).describe("Configuration for each task in the corresponding Function."),
  profile: VectorCompletionsRequestProfileSchema.describe("Weights for each Task in the corresponding Function.\n\nMust have the same length as `tasks`. Can be either:\n- A vector of decimals (legacy representation), or\n- A vector of objects with `weight` and optional `invert` fields.")
}).describe("An inline tasks-based profile definition without metadata.").meta({ title: "functions.InlineTasksProfile" });

// src/functions/inlineProfile.ts
var FunctionsInlineProfileSchema = z305.z.union([z305.z.lazy(() => FunctionsInlineTasksProfileSchema).describe("Tasks-based profile with per-task configuration."), FunctionsInlineAutoProfileSchema.describe("Auto profile that applies a single ensemble+weights to all vector completion tasks.")]).describe("An inline profile, either tasks-based or auto.").meta({ title: "functions.InlineProfile" });

// src/functions/executions/request/functionInlineProfileInlineRequestBody.ts
var FunctionsExecutionsRequestFunctionInlineProfileInlineRequestBodySchema = z305.z.object({
  function: FunctionsInlineFunctionSchema.describe("The inline Function definition."),
  profile: FunctionsInlineProfileSchema.describe("The inline Profile definition."),
  retry_token: z305.z.string().nullable().describe("If present, reuses votes from a previous execution with this token.").optional(),
  from_cache: z305.z.boolean().nullable().describe("If true, uses cached votes when available.").optional(),
  reasoning: FunctionsExecutionsRequestReasoningSchema.nullable().describe("Reasoning summary configuration.").optional(),
  strategy: FunctionsExecutionsRequestStrategySchema.nullable().describe("Execution strategy.\nDefaults to `Default` strategy if not specified.").optional(),
  input: FunctionsExpressionInputValueSchema.describe("The input data to pass to the Function."),
  provider: AgentCompletionsRequestProviderSchema.nullable().describe("Provider routing preferences.").optional(),
  seed: z305.z.number().int().meta({ format: "int64" }).nullable().describe("Random seed for deterministic results.").optional(),
  stream: z305.z.boolean().nullable().describe("Whether to stream the response.").optional(),
  mcp_server_authorization: z305.z.record(z305.z.string(), z305.z.string()).nullable().describe("Map from MCP server URL to authorization header value.").optional()
}).describe("Request body for inline Function with inline Profile.").meta({ title: "functions.executions.request.FunctionInlineProfileInlineRequestBody" });
var FunctionsExecutionsRequestFunctionInlineProfileRemoteRequestBodySchema = z305.z.object({
  function: FunctionsInlineFunctionSchema.describe("The inline Function definition."),
  retry_token: z305.z.string().nullable().describe("If present, reuses votes from a previous execution with this token.").optional(),
  from_cache: z305.z.boolean().nullable().describe("If true, uses cached votes when available.").optional(),
  reasoning: FunctionsExecutionsRequestReasoningSchema.nullable().describe("Reasoning summary configuration.").optional(),
  strategy: FunctionsExecutionsRequestStrategySchema.nullable().describe("Execution strategy.\nDefaults to `Default` strategy if not specified.").optional(),
  input: FunctionsExpressionInputValueSchema.describe("The input data to pass to the Function."),
  provider: AgentCompletionsRequestProviderSchema.nullable().describe("Provider routing preferences.").optional(),
  seed: z305.z.number().int().meta({ format: "int64" }).nullable().describe("Random seed for deterministic results.").optional(),
  stream: z305.z.boolean().nullable().describe("Whether to stream the response.").optional(),
  mcp_server_authorization: z305.z.record(z305.z.string(), z305.z.string()).nullable().describe("Map from MCP server URL to authorization header value.").optional()
}).describe("Request body for inline Function with remote Profile.").meta({ title: "functions.executions.request.FunctionInlineProfileRemoteRequestBody" });
var FunctionsExecutionsRequestFunctionRemoteProfileInlineRequestBodySchema = z305.z.object({
  profile: FunctionsInlineProfileSchema.describe("The inline Profile definition."),
  retry_token: z305.z.string().nullable().describe("If present, reuses votes from a previous execution with this token.").optional(),
  from_cache: z305.z.boolean().nullable().describe("If true, uses cached votes when available.").optional(),
  reasoning: FunctionsExecutionsRequestReasoningSchema.nullable().describe("Reasoning summary configuration.").optional(),
  strategy: FunctionsExecutionsRequestStrategySchema.nullable().describe("Execution strategy.\nDefaults to `Default` strategy if not specified.").optional(),
  input: FunctionsExpressionInputValueSchema.describe("The input data to pass to the Function."),
  provider: AgentCompletionsRequestProviderSchema.nullable().describe("Provider routing preferences.").optional(),
  seed: z305.z.number().int().meta({ format: "int64" }).nullable().describe("Random seed for deterministic results.").optional(),
  stream: z305.z.boolean().nullable().describe("Whether to stream the response.").optional(),
  mcp_server_authorization: z305.z.record(z305.z.string(), z305.z.string()).nullable().describe("Map from MCP server URL to authorization header value.").optional()
}).describe("Request body for remote Function with inline Profile.").meta({ title: "functions.executions.request.FunctionRemoteProfileInlineRequestBody" });
var FunctionsExecutionsRequestFunctionRemoteProfileRemoteRequestBodySchema = z305.z.object({
  retry_token: z305.z.string().nullable().describe("If present, reuses votes from a previous execution with this token.").optional(),
  from_cache: z305.z.boolean().nullable().describe("If true, uses cached votes when available.").optional(),
  reasoning: FunctionsExecutionsRequestReasoningSchema.nullable().describe("Reasoning summary configuration.").optional(),
  strategy: FunctionsExecutionsRequestStrategySchema.nullable().describe("Execution strategy.\nDefaults to `Default` strategy if not specified.").optional(),
  input: FunctionsExpressionInputValueSchema.describe("The input data to pass to the Function."),
  provider: AgentCompletionsRequestProviderSchema.nullable().describe("Provider routing preferences.").optional(),
  seed: z305.z.number().int().meta({ format: "int64" }).nullable().describe("Random seed for deterministic results.").optional(),
  stream: z305.z.boolean().nullable().describe("Whether to stream the response.").optional(),
  mcp_server_authorization: z305.z.record(z305.z.string(), z305.z.string()).nullable().describe("Map from MCP server URL to authorization header value.").optional()
}).describe("Base request body with common execution parameters.\n\nUsed directly for remote Function + remote Profile, or flattened into\nother request body types.").meta({ title: "functions.executions.request.FunctionRemoteProfileRemoteRequestBody" });

// src/functions/executions/request/functionExecutionCreateParams.ts
var FunctionsExecutionsRequestFunctionExecutionCreateParamsSchema = z305.z.union([FunctionsExecutionsRequestFunctionInlineProfileInlineRequestBodySchema.describe("Inline Function with inline Profile."), FunctionsExecutionsRequestFunctionInlineProfileRemoteRequestBodySchema.describe("Inline Function with remote Profile."), FunctionsExecutionsRequestFunctionRemoteProfileInlineRequestBodySchema.describe("Remote Function with inline Profile."), FunctionsExecutionsRequestFunctionRemoteProfileRemoteRequestBodySchema.describe("Remote Function with remote Profile.")]).describe("Parameters for creating a function execution.\n\nSupports four combinations based on whether the Function and Profile\nare provided inline or referenced from remote repositories.").meta({ title: "functions.executions.request.FunctionExecutionCreateParams" });
var FunctionsExecutionsRequestFunctionInlineProfileRemoteRequestPathSchema = z305.z.object({
  premote: FunctionsRemoteSchema.describe("Profile remote source."),
  powner: z305.z.string().describe("Profile repository owner."),
  prepository: z305.z.string().describe("Profile repository name."),
  pcommit: z305.z.string().nullable().describe("Profile Git commit SHA (optional).").optional()
}).describe("Path parameters for inline Function with remote Profile.").meta({ title: "functions.executions.request.FunctionInlineProfileRemoteRequestPath" });
var FunctionsExecutionsRequestFunctionRemoteProfileInlineRequestPathSchema = z305.z.object({
  fremote: FunctionsRemoteSchema.describe("Function remote source."),
  fowner: z305.z.string().describe("Function repository owner."),
  frepository: z305.z.string().describe("Function repository name."),
  fcommit: z305.z.string().nullable().describe("Function Git commit SHA (optional).").optional()
}).describe("Path parameters for remote Function with inline Profile.").meta({ title: "functions.executions.request.FunctionRemoteProfileInlineRequestPath" });
var FunctionsExecutionsRequestFunctionRemoteProfileRemoteRequestPathSchema = z305.z.object({
  fremote: FunctionsRemoteSchema.describe("Function remote source."),
  fowner: z305.z.string().describe("Function repository owner."),
  frepository: z305.z.string().describe("Function repository name."),
  fcommit: z305.z.string().nullable().describe("Function Git commit SHA (optional).").optional(),
  premote: FunctionsRemoteSchema.describe("Profile remote source."),
  powner: z305.z.string().describe("Profile repository owner."),
  prepository: z305.z.string().describe("Profile repository name."),
  pcommit: z305.z.string().nullable().describe("Profile Git commit SHA (optional).").optional()
}).describe("Path parameters for remote Function with remote Profile.").meta({ title: "functions.executions.request.FunctionRemoteProfileRemoteRequestPath" });
var FunctionsExecutionsRequestRequestSchema = z305.z.union([z305.z.object({
  body: FunctionsExecutionsRequestFunctionInlineProfileInlineRequestBodySchema
}), z305.z.object({
  path: FunctionsExecutionsRequestFunctionInlineProfileRemoteRequestPathSchema,
  body: FunctionsExecutionsRequestFunctionInlineProfileRemoteRequestBodySchema
}), z305.z.object({
  path: FunctionsExecutionsRequestFunctionRemoteProfileInlineRequestPathSchema,
  body: FunctionsExecutionsRequestFunctionRemoteProfileInlineRequestBodySchema
}), z305.z.object({
  path: FunctionsExecutionsRequestFunctionRemoteProfileRemoteRequestPathSchema,
  body: FunctionsExecutionsRequestFunctionRemoteProfileRemoteRequestBodySchema
})]).describe("Internal request representation with path and body separated.\n\nUsed internally to route requests to the appropriate API endpoint.").meta({ title: "functions.executions.request.Request" });
var FunctionsExecutionsResponseStreamingObjectSchema = z305.z.enum(["scalar.function.execution.chunk", "vector.function.execution.chunk"]).meta({ title: "functions.executions.response.streaming.Object" });
var FunctionsExecutionsResponseStreamingReasoningSummaryChunkSchema = z305.z.object({
  id: z305.z.string(),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }),
  messages: z305.z.array(AgentCompletionsResponseStreamingMessageChunkSchema),
  object: AgentCompletionsResponseStreamingObjectSchema.describe('The object type (always "agent.completion.chunk").'),
  usage: AgentCompletionsResponseUsageSchema.nullable().describe("Token usage (only present in the final chunk).").optional(),
  upstream: AgentUpstreamSchema.describe("Upstream provider"),
  error: ResponseErrorSchema.nullable().optional()
}).describe("A chunk of a streaming agent completion response.\n\nMultiple chunks are received via Server-Sent Events and can be\naccumulated into a complete [`AgentCompletion`](response::unary::AgentCompletion)\nusing the [`push`](Self::push) method.").meta({ title: "functions.executions.response.streaming.ReasoningSummaryChunk" });
var FunctionsExpressionTaskOutputOwnedSchema = z305.z.union([z305.z.number().meta({ format: "double" }).describe("A single scalar score."), z305.z.array(z305.z.number().meta({ format: "double" })).describe("A vector of scores."), z305.z.array(z305.z.array(z305.z.number().meta({ format: "double" }))).describe("Multiple vectors of scores (from mapped tasks)."), JsonValueSchema.describe("An error occurred during execution.")]).describe("Owned task output variants.").meta({ title: "functions.expression.TaskOutputOwned" });

// src/functions/executions/response/streaming/functionExecutionTaskChunk.ts
var FunctionsExecutionsResponseStreamingFunctionExecutionTaskChunkSchema = z305.z.object({
  index: z305.z.number().int().min(0).meta({ format: "uint64" }),
  task_index: z305.z.number().int().min(0).meta({ format: "uint64" }),
  task_path: z305.z.array(z305.z.number().int().min(0).meta({ format: "uint64" })),
  swiss_pool_index: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().optional(),
  swiss_round: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().optional(),
  id: z305.z.string(),
  tasks: z305.z.array(z305.z.lazy(() => FunctionsExecutionsResponseStreamingTaskChunkSchema)),
  tasks_errors: z305.z.boolean().nullable().optional(),
  reasoning: FunctionsExecutionsResponseStreamingReasoningSummaryChunkSchema.nullable().optional(),
  output: FunctionsExpressionTaskOutputOwnedSchema.nullable().optional(),
  error: ResponseErrorSchema.nullable().optional(),
  retry_token: z305.z.string().nullable().optional(),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }),
  function: z305.z.string().nullable().optional(),
  profile: z305.z.string().nullable().optional(),
  object: FunctionsExecutionsResponseStreamingObjectSchema,
  usage: AgentCompletionsResponseUsageSchema.nullable().optional()
}).meta({ title: "functions.executions.response.streaming.FunctionExecutionTaskChunk" });
var VectorCompletionsResponseStreamingAgentCompletionChunkSchema = z305.z.object({
  index: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Index used to correlate chunks from the same completion."),
  id: z305.z.string(),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }),
  messages: z305.z.array(AgentCompletionsResponseStreamingMessageChunkSchema),
  object: AgentCompletionsResponseStreamingObjectSchema.describe('The object type (always "agent.completion.chunk").'),
  usage: AgentCompletionsResponseUsageSchema.nullable().describe("Token usage (only present in the final chunk).").optional(),
  upstream: AgentUpstreamSchema.describe("Upstream provider"),
  error: ResponseErrorSchema.nullable().describe("Error details if this completion failed.").optional()
}).describe("A streaming agent completion chunk from a single agent within a vector completion.\n\nThe `index` field is used to correlate chunks belonging to the same\nunderlying completion when accumulating via [`push`](Self::push).").meta({ title: "vector.completions.response.streaming.AgentCompletionChunk" });
var VectorCompletionsResponseStreamingObjectSchema = z305.z.union([z305.z.literal("vector.completion.chunk").describe("A streaming vector completion chunk.")]).describe('Object type for streaming vector completion chunks.\n\nSerializes to `"vector.completion.chunk"` in JSON.').meta({ title: "vector.completions.response.streaming.Object" });
var VectorCompletionsResponseVoteSchema = z305.z.object({
  agent: z305.z.string().describe("The agent that produced this vote (content-addressed ID)."),
  ensemble_index: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Index of the agent configuration within the ensemble."),
  flat_ensemble_index: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Flattened index accounting for agent counts in the ensemble."),
  prompt_id: z305.z.string().describe("Content hash of the request messages (for caching/deduplication)."),
  responses_ids: z305.z.array(z305.z.string()).describe("Content hashes of each response option in the request."),
  vote: z305.z.array(z305.z.number().meta({ format: "double" })).describe("The vote distribution. Each index corresponds to a response from the\nrequest. Typically one element is 1.0 (selected) and the rest are 0.0."),
  weight: z305.z.number().meta({ format: "double" }).describe("The weight applied to this vote when computing final scores."),
  retry: z305.z.boolean().nullable().describe("If true, this vote was reused from a previous request via the `retry`\nparameter. All fields reflect the original request's values.").optional(),
  from_cache: z305.z.boolean().nullable().describe("If true, this vote was retrieved from cache rather than generated fresh.").optional()
}).describe("A single LLM's vote in a vector completion.\n\nEach LLM in the ensemble produces a vote indicating which response(s) it\nselected. Votes are weighted according to the profile and combined to\nproduce the final scores.\n\n# Vote Format\n\nThe `vote` field is a vector of decimals corresponding to the responses\nin the request. Typically one element is 1.0 and the rest are 0.0 (discrete\nselection), but when `top_logprobs` is used, votes may be probability\ndistributions.").meta({ title: "vector.completions.response.Vote" });

// src/functions/executions/response/streaming/vectorCompletionTaskChunk.ts
var FunctionsExecutionsResponseStreamingVectorCompletionTaskChunkSchema = z305.z.object({
  index: z305.z.number().int().min(0).meta({ format: "uint64" }),
  task_index: z305.z.number().int().min(0).meta({ format: "uint64" }),
  task_path: z305.z.array(z305.z.number().int().min(0).meta({ format: "uint64" })),
  id: z305.z.string().describe("Unique identifier for this vector completion."),
  completions: z305.z.array(VectorCompletionsResponseStreamingAgentCompletionChunkSchema).describe("Incremental agent completion chunks from each agent."),
  votes: z305.z.array(VectorCompletionsResponseVoteSchema).describe("Votes received so far. New votes are appended in subsequent chunks."),
  scores: z305.z.array(z305.z.number().meta({ format: "double" })).describe("Current weighted scores. Updated as new votes arrive."),
  weights: z305.z.array(z305.z.number().meta({ format: "double" })).describe("Current weight distribution across responses. Updated as new votes arrive."),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Unix timestamp when the completion was created."),
  ensemble: z305.z.string().describe("ID of the ensemble used for this completion."),
  object: VectorCompletionsResponseStreamingObjectSchema.describe('Object type identifier (`"vector.completion.chunk"`).'),
  usage: AgentCompletionsResponseUsageSchema.nullable().describe("Aggregated usage statistics. Typically present only in the final chunk.").optional(),
  error: ResponseErrorSchema.nullable().optional()
}).describe("A chunk in a streaming vector completion response.\n\nEach chunk contains incremental updates to the completion. Use the\n[`push`](Self::push) method to accumulate chunks into a complete response.").meta({ title: "functions.executions.response.streaming.VectorCompletionTaskChunk" });

// src/functions/executions/response/streaming/taskChunk.ts
var FunctionsExecutionsResponseStreamingTaskChunkSchema = z305.z.union([z305.z.lazy(() => FunctionsExecutionsResponseStreamingFunctionExecutionTaskChunkSchema), FunctionsExecutionsResponseStreamingVectorCompletionTaskChunkSchema]).meta({ title: "functions.executions.response.streaming.TaskChunk" });

// src/functions/executions/response/streaming/functionExecutionChunk.ts
var FunctionsExecutionsResponseStreamingFunctionExecutionChunkSchema = z305.z.object({
  id: z305.z.string(),
  tasks: z305.z.array(FunctionsExecutionsResponseStreamingTaskChunkSchema),
  tasks_errors: z305.z.boolean().nullable().optional(),
  reasoning: FunctionsExecutionsResponseStreamingReasoningSummaryChunkSchema.nullable().optional(),
  output: FunctionsExpressionTaskOutputOwnedSchema.nullable().optional(),
  error: ResponseErrorSchema.nullable().optional(),
  retry_token: z305.z.string().nullable().optional(),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }),
  function: z305.z.string().nullable().optional(),
  profile: z305.z.string().nullable().optional(),
  object: FunctionsExecutionsResponseStreamingObjectSchema,
  usage: AgentCompletionsResponseUsageSchema.nullable().optional()
}).meta({ title: "functions.executions.response.streaming.FunctionExecutionChunk" });

// src/functions/executions/response/streaming/reasoningSummaryChunkMerged.ts
function functionsExecutionsResponseStreamingReasoningSummaryChunkMerged(a, b) {
  let changed = false;
  const [messages, c1] = agentCompletionsResponseStreamingMessageChunkMergedList(a.messages, b.messages);
  if (c1) changed = true;
  let usage = a.usage;
  if (a.usage != null && b.usage != null) {
    const [merged, c] = agentCompletionsResponseUsageMerged(a.usage, b.usage);
    usage = merged;
    if (c) changed = true;
  } else if (b.usage != null) {
    usage = b.usage;
    changed = true;
  }
  let error = a.error;
  if (a.error == null && b.error != null) {
    error = b.error;
    changed = true;
  }
  if (!changed) return [a, false];
  return [{
    id: a.id,
    created: a.created,
    messages,
    object: a.object,
    ...usage != null ? { usage } : {},
    upstream: a.upstream,
    ...error != null ? { error } : {}
  }, true];
}

// src/vector/completions/response/streaming/agentCompletionChunkMerged.ts
function vectorCompletionsResponseStreamingAgentCompletionChunkMerged(a, b) {
  let changed = false;
  const [messages, c1] = agentCompletionsResponseStreamingMessageChunkMergedList(a.messages, b.messages);
  if (c1) changed = true;
  let usage = a.usage;
  if (a.usage != null && b.usage != null) {
    const [merged, c] = agentCompletionsResponseUsageMerged(a.usage, b.usage);
    usage = merged;
    if (c) changed = true;
  } else if (b.usage != null) {
    usage = b.usage;
    changed = true;
  }
  let error = a.error;
  if (a.error == null && b.error != null) {
    error = b.error;
    changed = true;
  }
  if (!changed) return [a, false];
  return [{
    index: a.index,
    id: a.id,
    created: a.created,
    messages,
    object: a.object,
    ...usage != null ? { usage } : {},
    upstream: a.upstream,
    ...error != null ? { error } : {}
  }, true];
}
function vectorCompletionsResponseStreamingAgentCompletionChunkMergedList(a, b) {
  let changed = false;
  const result = [...a];
  for (const bItem of b) {
    const existingIdx = result.findIndex((x) => x.index === bItem.index);
    if (existingIdx !== -1) {
      const [merged, c] = vectorCompletionsResponseStreamingAgentCompletionChunkMerged(result[existingIdx], bItem);
      if (c) {
        result[existingIdx] = merged;
        changed = true;
      }
    } else {
      result.push(bItem);
      changed = true;
    }
  }
  if (!changed) return [a, false];
  return [result, true];
}

// src/vector/completions/response/voteMerged.ts
function vectorCompletionsResponseVoteMergedList(a, b) {
  if (b.length === 0) return [a, false];
  return [[...a, ...b], true];
}

// src/functions/executions/response/streaming/vectorCompletionTaskChunkMerged.ts
function functionsExecutionsResponseStreamingVectorCompletionTaskChunkMerged(a, b) {
  let changed = false;
  const [completions, c1] = vectorCompletionsResponseStreamingAgentCompletionChunkMergedList(a.completions, b.completions);
  if (c1) changed = true;
  const [votes, c2] = vectorCompletionsResponseVoteMergedList(a.votes, b.votes);
  if (c2) changed = true;
  const [scores, c3] = mergedNumberArray(a.scores, b.scores);
  if (c3) changed = true;
  const [weights, c4] = mergedNumberArray(a.weights, b.weights);
  if (c4) changed = true;
  let usage = a.usage;
  if (a.usage != null && b.usage != null) {
    const [merged, c] = agentCompletionsResponseUsageMerged(a.usage, b.usage);
    usage = merged;
    if (c) changed = true;
  } else if (b.usage != null) {
    usage = b.usage;
    changed = true;
  }
  let error = a.error;
  if (a.error == null && b.error != null) {
    error = b.error;
    changed = true;
  }
  if (!changed) return [a, false];
  return [{
    index: a.index,
    task_index: a.task_index,
    task_path: a.task_path,
    id: a.id,
    completions,
    votes,
    scores,
    weights,
    created: a.created,
    ensemble: a.ensemble,
    object: a.object,
    ...usage != null ? { usage } : {},
    ...error != null ? { error } : {}
  }, true];
}

// src/functions/executions/response/streaming/taskChunkMerged.ts
function isVectorCompletionTaskChunk(chunk) {
  return "scores" in chunk;
}
function taskChunkIndex(chunk) {
  return chunk.index;
}
function functionsExecutionsResponseStreamingFunctionExecutionTaskChunkMerged(a, b) {
  let changed = false;
  const [tasks, c1] = functionsExecutionsResponseStreamingTaskChunkMergedList(a.tasks, b.tasks);
  if (c1) changed = true;
  let tasks_errors = a.tasks_errors;
  if (b.tasks_errors === true) {
    if (a.tasks_errors !== true) changed = true;
    tasks_errors = true;
  }
  let reasoning = a.reasoning;
  if (a.reasoning != null && b.reasoning != null) {
    const [merged, c] = functionsExecutionsResponseStreamingReasoningSummaryChunkMerged(a.reasoning, b.reasoning);
    reasoning = merged;
    if (c) changed = true;
  } else if (b.reasoning != null) {
    reasoning = b.reasoning;
    changed = true;
  }
  let output = a.output;
  if (b.output != null) {
    output = b.output;
    changed = true;
  }
  let retry_token = a.retry_token;
  if (b.retry_token != null) {
    retry_token = b.retry_token;
    changed = true;
  }
  let error = a.error;
  if (b.error != null) {
    error = b.error;
    changed = true;
  }
  let usage = a.usage;
  if (a.usage != null && b.usage != null) {
    const [merged, c] = agentCompletionsResponseUsageMerged(a.usage, b.usage);
    usage = merged;
    if (c) changed = true;
  } else if (b.usage != null) {
    usage = b.usage;
    changed = true;
  }
  if (!changed) return [a, false];
  return [{
    index: a.index,
    task_index: a.task_index,
    task_path: a.task_path,
    ...a.swiss_pool_index != null ? { swiss_pool_index: a.swiss_pool_index } : {},
    ...a.swiss_round != null ? { swiss_round: a.swiss_round } : {},
    id: a.id,
    tasks,
    ...tasks_errors != null ? { tasks_errors } : {},
    ...reasoning != null ? { reasoning } : {},
    ...output != null ? { output } : {},
    ...error != null ? { error } : {},
    ...retry_token != null ? { retry_token } : {},
    created: a.created,
    ...a.function !== void 0 ? { function: a.function } : {},
    ...a.profile !== void 0 ? { profile: a.profile } : {},
    object: a.object,
    ...usage != null ? { usage } : {}
  }, true];
}
function functionsExecutionsResponseStreamingTaskChunkMerged(a, b) {
  if (isVectorCompletionTaskChunk(a) && isVectorCompletionTaskChunk(b)) {
    return functionsExecutionsResponseStreamingVectorCompletionTaskChunkMerged(a, b);
  }
  if (!isVectorCompletionTaskChunk(a) && !isVectorCompletionTaskChunk(b)) {
    return functionsExecutionsResponseStreamingFunctionExecutionTaskChunkMerged(a, b);
  }
  return [a, false];
}
function functionsExecutionsResponseStreamingTaskChunkMergedList(a, b) {
  let changed = false;
  const result = [...a];
  for (const bItem of b) {
    const bIndex = taskChunkIndex(bItem);
    const existingIdx = result.findIndex((x) => taskChunkIndex(x) === bIndex);
    if (existingIdx !== -1) {
      const [merged, c] = functionsExecutionsResponseStreamingTaskChunkMerged(result[existingIdx], bItem);
      if (c) {
        result[existingIdx] = merged;
        changed = true;
      }
    } else {
      result.push(bItem);
      changed = true;
    }
  }
  if (!changed) return [a, false];
  return [result, true];
}

// src/functions/executions/response/streaming/functionExecutionChunkMerged.ts
function functionsExecutionsResponseStreamingFunctionExecutionChunkMerged(a, b) {
  let changed = false;
  const [tasks, c1] = functionsExecutionsResponseStreamingTaskChunkMergedList(a.tasks, b.tasks);
  if (c1) changed = true;
  let tasks_errors = a.tasks_errors;
  if (b.tasks_errors === true) {
    if (a.tasks_errors !== true) changed = true;
    tasks_errors = true;
  }
  let reasoning = a.reasoning;
  if (a.reasoning != null && b.reasoning != null) {
    const [merged, c] = functionsExecutionsResponseStreamingReasoningSummaryChunkMerged(a.reasoning, b.reasoning);
    reasoning = merged;
    if (c) changed = true;
  } else if (b.reasoning != null) {
    reasoning = b.reasoning;
    changed = true;
  }
  let output = a.output;
  if (b.output != null) {
    output = b.output;
    changed = true;
  }
  let retry_token = a.retry_token;
  if (b.retry_token != null) {
    retry_token = b.retry_token;
    changed = true;
  }
  let error = a.error;
  if (b.error != null) {
    error = b.error;
    changed = true;
  }
  let usage = a.usage;
  if (a.usage != null && b.usage != null) {
    const [merged, c] = agentCompletionsResponseUsageMerged(a.usage, b.usage);
    usage = merged;
    if (c) changed = true;
  } else if (b.usage != null) {
    usage = b.usage;
    changed = true;
  }
  if (!changed) return [a, false];
  return [{
    id: a.id,
    tasks,
    ...tasks_errors != null ? { tasks_errors } : {},
    ...reasoning != null ? { reasoning } : {},
    ...output != null ? { output } : {},
    ...error != null ? { error } : {},
    ...retry_token != null ? { retry_token } : {},
    created: a.created,
    ...a.function !== void 0 ? { function: a.function } : {},
    ...a.profile !== void 0 ? { profile: a.profile } : {},
    object: a.object,
    ...usage != null ? { usage } : {}
  }, true];
}
var FunctionsExecutionsResponseUnaryObjectSchema = z305.z.enum(["scalar.function.execution", "vector.function.execution"]).meta({ title: "functions.executions.response.unary.Object" });
var FunctionsExecutionsResponseUnaryReasoningSummarySchema = z305.z.object({
  id: z305.z.string(),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }),
  messages: z305.z.array(AgentCompletionsResponseUnaryMessageSchema),
  object: AgentCompletionsResponseUnaryObjectSchema.describe('The object type (always "agent.completion").'),
  usage: AgentCompletionsResponseUsageSchema,
  upstream: AgentUpstreamSchema.describe("Upstream provider"),
  error: ResponseErrorSchema.nullable().optional()
}).describe("A complete agent completion response.").meta({ title: "functions.executions.response.unary.ReasoningSummary" });
var FunctionsExecutionsResponseUnaryFunctionExecutionTaskSchema = z305.z.object({
  index: z305.z.number().int().min(0).meta({ format: "uint64" }),
  task_index: z305.z.number().int().min(0).meta({ format: "uint64" }),
  task_path: z305.z.array(z305.z.number().int().min(0).meta({ format: "uint64" })),
  swiss_pool_index: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().optional(),
  swiss_round: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().optional(),
  id: z305.z.string().describe("Unique identifier for this execution."),
  tasks: z305.z.array(z305.z.lazy(() => FunctionsExecutionsResponseUnaryTaskSchema)).describe("Results from each task in the function."),
  tasks_errors: z305.z.boolean().describe("Whether any tasks encountered errors."),
  reasoning: FunctionsExecutionsResponseUnaryReasoningSummarySchema.nullable().describe("Reasoning summary if reasoning was enabled.").optional(),
  output: FunctionsExpressionTaskOutputOwnedSchema.describe("The final output (scalar or vector score)."),
  error: ResponseErrorSchema.nullable().describe("Error details if the execution failed.").optional(),
  retry_token: z305.z.string().nullable().describe("Token for retrying this execution with cached votes.").optional(),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Unix timestamp when the execution was created."),
  function: z305.z.string().nullable().describe("ID of the function used (if remote).").optional(),
  profile: z305.z.string().nullable().describe("ID of the profile used (if remote).").optional(),
  object: FunctionsExecutionsResponseUnaryObjectSchema.describe("Object type identifier."),
  usage: AgentCompletionsResponseUsageSchema.describe("Aggregated token and cost usage.")
}).describe("A complete function execution response (non-streaming).").meta({ title: "functions.executions.response.unary.FunctionExecutionTask" });
var VectorCompletionsResponseUnaryAgentCompletionSchema = z305.z.object({
  index: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Index of this completion within the vector completion."),
  id: z305.z.string(),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }),
  messages: z305.z.array(AgentCompletionsResponseUnaryMessageSchema),
  object: AgentCompletionsResponseUnaryObjectSchema.describe('The object type (always "agent.completion").'),
  usage: AgentCompletionsResponseUsageSchema,
  upstream: AgentUpstreamSchema.describe("Upstream provider"),
  error: ResponseErrorSchema.nullable().describe("Error details if this completion failed.").optional()
}).describe("A agent completion from a single agent within a vector completion.\n\nWraps the standard agent completion response with an index to identify\nwhich agent in the ensemble produced it.").meta({ title: "vector.completions.response.unary.AgentCompletion" });
var VectorCompletionsResponseUnaryObjectSchema = z305.z.union([z305.z.literal("vector.completion").describe("A complete vector completion response.")]).describe('Object type for unary vector completion responses.\n\nSerializes to `"vector.completion"` in JSON.').meta({ title: "vector.completions.response.unary.Object" });

// src/functions/executions/response/unary/vectorCompletionTask.ts
var FunctionsExecutionsResponseUnaryVectorCompletionTaskSchema = z305.z.object({
  index: z305.z.number().int().min(0).meta({ format: "uint64" }),
  task_index: z305.z.number().int().min(0).meta({ format: "uint64" }),
  task_path: z305.z.array(z305.z.number().int().min(0).meta({ format: "uint64" })),
  id: z305.z.string().describe("Unique identifier for this vector completion."),
  completions: z305.z.array(VectorCompletionsResponseUnaryAgentCompletionSchema).describe("The underlying agent completions from each agent in the ensemble."),
  votes: z305.z.array(VectorCompletionsResponseVoteSchema).describe("Individual votes from each agent, showing their selections."),
  scores: z305.z.array(z305.z.number().meta({ format: "double" })).describe("Final weighted scores for each response option. Sums to 1."),
  weights: z305.z.array(z305.z.number().meta({ format: "double" })).describe("Total weight allocated to each response option. Same length as `scores`.\nFor discrete votes, an LLM's full weight goes to its selected response.\nFor probabilistic votes, the weight is divided according to the distribution."),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Unix timestamp when the completion was created."),
  ensemble: z305.z.string().describe("ID of the ensemble used for this completion."),
  object: VectorCompletionsResponseUnaryObjectSchema.describe('Object type identifier (`"vector.completion"`).'),
  usage: AgentCompletionsResponseUsageSchema.describe("Aggregated token and cost usage across all completions."),
  error: ResponseErrorSchema.nullable().optional()
}).describe("A complete vector completion response (non-streaming).\n\nContains the final scores, all votes from the ensemble, and the underlying\nagent completions that produced those votes.").meta({ title: "functions.executions.response.unary.VectorCompletionTask" });

// src/functions/executions/response/unary/task.ts
var FunctionsExecutionsResponseUnaryTaskSchema = z305.z.union([z305.z.lazy(() => FunctionsExecutionsResponseUnaryFunctionExecutionTaskSchema), FunctionsExecutionsResponseUnaryVectorCompletionTaskSchema]).meta({ title: "functions.executions.response.unary.Task" });

// src/functions/executions/response/unary/functionExecution.ts
var FunctionsExecutionsResponseUnaryFunctionExecutionSchema = z305.z.object({
  id: z305.z.string().describe("Unique identifier for this execution."),
  tasks: z305.z.array(FunctionsExecutionsResponseUnaryTaskSchema).describe("Results from each task in the function."),
  tasks_errors: z305.z.boolean().describe("Whether any tasks encountered errors."),
  reasoning: FunctionsExecutionsResponseUnaryReasoningSummarySchema.nullable().describe("Reasoning summary if reasoning was enabled.").optional(),
  output: FunctionsExpressionTaskOutputOwnedSchema.describe("The final output (scalar or vector score)."),
  error: ResponseErrorSchema.nullable().describe("Error details if the execution failed.").optional(),
  retry_token: z305.z.string().nullable().describe("Token for retrying this execution with cached votes.").optional(),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Unix timestamp when the execution was created."),
  function: z305.z.string().nullable().describe("ID of the function used (if remote).").optional(),
  profile: z305.z.string().nullable().describe("ID of the profile used (if remote).").optional(),
  object: FunctionsExecutionsResponseUnaryObjectSchema.describe("Object type identifier."),
  usage: AgentCompletionsResponseUsageSchema.describe("Aggregated token and cost usage.")
}).describe("A complete function execution response (non-streaming).").meta({ title: "functions.executions.response.unary.FunctionExecution" });
var FunctionsExecutionsRetryTokenSchema = z305.z.array(z305.z.string().nullable()).describe("Token that enables reusing votes from a previous function execution.\n\nContains identifiers for each task's votes that can be reused in a\nsubsequent execution. Serialized as base64-encoded JSON.").meta({ title: "functions.executions.RetryToken" });

// src/functions/executions/http.ts
function buildExecutionPath(request) {
  if (!("path" in request)) {
    return "/functions";
  }
  const { path } = request;
  if ("fremote" in path && "premote" in path) {
    let url2 = `/functions/${path.fremote}/${path.fowner}/${path.frepository}`;
    if (path.fcommit != null) url2 += `/${path.fcommit}`;
    url2 += `/profiles/${path.premote}/${path.powner}/${path.prepository}`;
    if (path.pcommit != null) url2 += `/${path.pcommit}`;
    return url2;
  }
  if ("fremote" in path) {
    let url2 = `/functions/${path.fremote}/${path.fowner}/${path.frepository}`;
    if (path.fcommit != null) url2 += `/${path.fcommit}`;
    return url2;
  }
  let url = `/functions/profiles/${path.premote}/${path.powner}/${path.prepository}`;
  if (path.pcommit != null) url += `/${path.pcommit}`;
  return url;
}
function functionsExecutionsCreateFunctionExecution(client, request, options) {
  const path = buildExecutionPath(request);
  if (request.body.stream) {
    return client.post_streaming(
      path,
      request.body,
      options
    );
  }
  return client.post_unary(
    path,
    request.body,
    options
  );
}
var FunctionsExpressionOneOrManyStringSchema = z305.z.union([z305.z.string().describe("A single value."), z305.z.array(z305.z.string()).describe("Multiple values (from array expressions).")]).describe("Result of an expression that may produce one or many values.").meta({ title: "functions.expression.OneOrMany.string" });
var FunctionsExpressionParamsOwnedSchema = z305.z.object({
  input: FunctionsExpressionInputValueSchema.describe("The function's input data."),
  output: FunctionsExpressionTaskOutputOwnedSchema.nullable().describe("Results from executed tasks. Only populated for task output expressions.").optional(),
  map: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Current map index. Only populated for mapped task expressions.").optional()
}).describe("Owned version of expression parameters.").meta({ title: "functions.expression.ParamsOwned" });
var FunctionsExpressionTaskOutputRefSchema = z305.z.union([z305.z.number().meta({ format: "double" }).describe("A single scalar score."), z305.z.array(z305.z.number().meta({ format: "double" })).describe("A vector of scores."), z305.z.array(z305.z.array(z305.z.number().meta({ format: "double" }))).describe("Multiple vectors of scores (from mapped tasks)."), JsonValueSchema.describe("An error occurred during execution.")]).describe("Borrowed task output variants.").meta({ title: "functions.expression.TaskOutputRef" });

// src/functions/expression/taskOutput.ts
var FunctionsExpressionTaskOutputSchema = z305.z.union([FunctionsExpressionTaskOutputOwnedSchema.describe("Owned version."), FunctionsExpressionTaskOutputRefSchema.describe("Borrowed version.")]).describe("Output from an executed task.").meta({ title: "functions.expression.TaskOutput" });

// src/functions/expression/paramsRef.ts
var FunctionsExpressionParamsRefSchema = z305.z.object({
  input: FunctionsExpressionInputValueSchema.describe("The function's input data."),
  output: FunctionsExpressionTaskOutputSchema.nullable().describe("Results from executed tasks. Only populated for task output expressions.").optional(),
  map: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Current map index. Only populated for mapped task expressions.").optional()
}).describe("Borrowed version of expression parameters.").meta({ title: "functions.expression.ParamsRef" });

// src/functions/expression/params.ts
var FunctionsExpressionParamsSchema = z305.z.union([FunctionsExpressionParamsOwnedSchema.describe("Owned version (for deserialization)."), FunctionsExpressionParamsRefSchema.describe("Borrowed version (for efficient evaluation).")]).describe("Context for evaluating expressions (JMESPath or Starlark).\n\nContains all data accessible within expressions: `input`, `output`, and `map`.").meta({ title: "functions.expression.Params" });
var FunctionsInventionsStateAlphaScalarBranchStateSchema = z305.z.object({
  depth: z305.z.number().int().min(0).meta({ format: "uint64" }),
  min_branch_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  max_branch_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  min_leaf_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  max_leaf_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  name: z305.z.string(),
  spec: z305.z.string(),
  essay: z305.z.string().nullable().optional(),
  input_schema: FunctionsExpressionObjectInputSchemaSchema.nullable().optional(),
  essay_tasks: z305.z.string().nullable().optional(),
  tasks: z305.z.array(FunctionsAlphaScalarBranchTaskExpressionSchema).nullable().optional(),
  tasks_length: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().optional(),
  description: z305.z.string().nullable().optional(),
  readme: z305.z.string().nullable().optional()
}).meta({ title: "functions.inventions.state.AlphaScalarBranchState" });
var FunctionsInventionsStateAlphaScalarLeafStateSchema = z305.z.object({
  depth: z305.z.number().int().min(0).meta({ format: "uint64" }),
  min_branch_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  max_branch_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  min_leaf_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  max_leaf_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  name: z305.z.string(),
  spec: z305.z.string(),
  essay: z305.z.string().nullable().optional(),
  input_schema: FunctionsExpressionObjectInputSchemaSchema.nullable().optional(),
  essay_tasks: z305.z.string().nullable().optional(),
  tasks: z305.z.array(FunctionsAlphaScalarLeafTaskExpressionSchema).nullable().optional(),
  tasks_length: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().optional(),
  description: z305.z.string().nullable().optional(),
  readme: z305.z.string().nullable().optional()
}).meta({ title: "functions.inventions.state.AlphaScalarLeafState" });
var FunctionsInventionsStateAlphaScalarStateSchema = z305.z.object({
  depth: z305.z.number().int().min(0).meta({ format: "uint64" }),
  min_branch_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  max_branch_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  min_leaf_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  max_leaf_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  name: z305.z.string(),
  spec: z305.z.string(),
  input_schema: FunctionsExpressionObjectInputSchemaSchema.nullable().optional()
}).meta({ title: "functions.inventions.state.AlphaScalarState" });
var FunctionsInventionsStateAlphaVectorBranchStateSchema = z305.z.object({
  depth: z305.z.number().int().min(0).meta({ format: "uint64" }),
  min_branch_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  max_branch_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  min_leaf_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  max_leaf_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  name: z305.z.string(),
  spec: z305.z.string(),
  essay: z305.z.string().nullable().optional(),
  input_schema: FunctionsAlphaVectorExpressionVectorFunctionInputSchemaSchema.nullable().optional(),
  essay_tasks: z305.z.string().nullable().optional(),
  tasks: z305.z.array(FunctionsAlphaVectorBranchTaskExpressionSchema).nullable().optional(),
  tasks_length: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().optional(),
  description: z305.z.string().nullable().optional(),
  readme: z305.z.string().nullable().optional()
}).meta({ title: "functions.inventions.state.AlphaVectorBranchState" });
var FunctionsInventionsStateAlphaVectorLeafStateSchema = z305.z.object({
  depth: z305.z.number().int().min(0).meta({ format: "uint64" }),
  min_branch_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  max_branch_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  min_leaf_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  max_leaf_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  name: z305.z.string(),
  spec: z305.z.string(),
  essay: z305.z.string().nullable().optional(),
  input_schema: FunctionsAlphaVectorExpressionVectorFunctionInputSchemaSchema.nullable().optional(),
  essay_tasks: z305.z.string().nullable().optional(),
  tasks: z305.z.array(FunctionsAlphaVectorLeafTaskExpressionSchema).nullable().optional(),
  tasks_length: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().optional(),
  description: z305.z.string().nullable().optional(),
  readme: z305.z.string().nullable().optional()
}).meta({ title: "functions.inventions.state.AlphaVectorLeafState" });
var FunctionsInventionsStateAlphaVectorStateSchema = z305.z.object({
  depth: z305.z.number().int().min(0).meta({ format: "uint64" }),
  min_branch_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  max_branch_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  min_leaf_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  max_leaf_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  name: z305.z.string(),
  spec: z305.z.string(),
  input_schema: FunctionsAlphaVectorExpressionVectorFunctionInputSchemaSchema.nullable().optional()
}).meta({ title: "functions.inventions.state.AlphaVectorState" });

// src/functions/inventions/state/paramsState.ts
var FunctionsInventionsStateParamsStateSchema = z305.z.union([FunctionsInventionsStateAlphaScalarBranchStateSchema.extend({
  type: z305.z.literal("alpha.scalar.branch.function")
}), FunctionsInventionsStateAlphaScalarLeafStateSchema.extend({
  type: z305.z.literal("alpha.scalar.leaf.function")
}), FunctionsInventionsStateAlphaVectorBranchStateSchema.extend({
  type: z305.z.literal("alpha.vector.branch.function")
}), FunctionsInventionsStateAlphaVectorLeafStateSchema.extend({
  type: z305.z.literal("alpha.vector.leaf.function")
}), FunctionsInventionsStateAlphaScalarStateSchema.extend({
  type: z305.z.literal("alpha.scalar.function")
}), FunctionsInventionsStateAlphaVectorStateSchema.extend({
  type: z305.z.literal("alpha.vector.function")
})]).meta({ title: "functions.inventions.state.ParamsState" });

// src/functions/inventions/recursive/request/functionInventionRecursiveCreateParams.ts
var FunctionsInventionsRecursiveRequestFunctionInventionRecursiveCreateParamsSchema = z305.z.object({
  remote: FunctionsRemoteSchema,
  name: z305.z.string(),
  state: FunctionsInventionsStateParamsStateSchema,
  provider: AgentCompletionsRequestProviderSchema.nullable().optional(),
  agent: AgentCompletionsRequestAgentSchema,
  agents: z305.z.array(AgentCompletionsRequestAgentSchema).nullable().optional(),
  seed: z305.z.number().int().meta({ format: "int64" }).nullable().optional(),
  stream: z305.z.boolean().nullable().optional(),
  max_step_retries: z305.z.number().int().min(0).meta({ format: "uint32" }).nullable().describe("Maximum number of retries per invention step.\nEach step is one agent completion (which itself may loop internally\nvia tool calls). If the step's validation still fails after the\nagent loop ends, the step is retried up to this many times.\nDefaults to 3 if not specified.").optional(),
  mcp_server_authorization: z305.z.record(z305.z.string(), z305.z.string()).nullable().describe("Map from MCP server URL to authorization header value.").optional()
}).meta({ title: "functions.inventions.recursive.request.FunctionInventionRecursiveCreateParams" });
var FunctionsAlphaRemoteFunctionSchema = z305.z.union([FunctionsAlphaScalarRemoteFunctionSchema, FunctionsAlphaVectorRemoteFunctionSchema]).meta({ title: "functions.AlphaRemoteFunction" });
var FunctionsRemoteFunctionSchema = z305.z.union([z305.z.object({
  description: z305.z.string().describe("Human-readable description of what the function does."),
  input_schema: FunctionsExpressionInputSchemaSchema.describe("JSON Schema defining the expected input structure."),
  tasks: z305.z.array(FunctionsTaskExpressionSchema).describe("The list of tasks to execute. Tasks with a `map` expression are\nexpanded into multiple instances. Each instance is compiled with\n`map` set to the current integer index.\nReceives: `input`, `map` (if mapped)."),
  type: z305.z.literal("scalar.function")
}).describe("Produces a single score in [0, 1]."), z305.z.object({
  description: z305.z.string().describe("Human-readable description of what the function does."),
  input_schema: FunctionsExpressionInputSchemaSchema.describe("JSON Schema defining the expected input structure."),
  tasks: z305.z.array(FunctionsTaskExpressionSchema).describe("The list of tasks to execute. Tasks with a `map` expression are\nexpanded into multiple instances. Each instance is compiled with\n`map` set to the current integer index.\nReceives: `input`, `map` (if mapped)."),
  output_length: FunctionsExpressionExpressionSchema.describe("Expression computing the expected output vector length for task outputs.\nReceives: `input`."),
  input_split: FunctionsExpressionExpressionSchema.describe("Expression transforming input into an input array of the output_length\nWhen the Function is executed with any input from the array,\nThe output_length should be 1.\nReceives: `input`."),
  input_merge: FunctionsExpressionExpressionSchema.describe("Expression transforming an array of inputs computed by `input_split`\ninto a single Input object for the Function.\nReceives: `input` (as an array)."),
  type: z305.z.literal("vector.function")
}).describe("Produces a vector of scores that sums to 1.")]).describe("A remote function with full metadata.\n\nRemote functions are stored as `function.json` in repositories and\nreferenced by `remote/owner/repository`. They include documentation fields\nthat inline functions lack.").meta({ title: "functions.RemoteFunction" });

// src/functions/fullRemoteFunction.ts
var FunctionsFullRemoteFunctionSchema = z305.z.union([FunctionsAlphaRemoteFunctionSchema, FunctionsRemoteFunctionSchema]).meta({ title: "functions.FullRemoteFunction" });
var FunctionsInventionsResponseStreamingAgentCompletionChunkSchema = z305.z.object({
  index: z305.z.number().int().min(0).meta({ format: "uint64" }),
  id: z305.z.string(),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }),
  messages: z305.z.array(AgentCompletionsResponseStreamingMessageChunkSchema),
  object: AgentCompletionsResponseStreamingObjectSchema.describe('The object type (always "agent.completion.chunk").'),
  usage: AgentCompletionsResponseUsageSchema.nullable().describe("Token usage (only present in the final chunk).").optional(),
  upstream: AgentUpstreamSchema.describe("Upstream provider"),
  error: ResponseErrorSchema.nullable().describe("Error details if this completion failed.").optional()
}).describe("A chunk of a streaming agent completion response.\n\nMultiple chunks are received via Server-Sent Events and can be\naccumulated into a complete [`AgentCompletion`](response::unary::AgentCompletion)\nusing the [`push`](Self::push) method.").meta({ title: "functions.inventions.response.streaming.AgentCompletionChunk" });
var FunctionsInventionsResponseStreamingObjectSchema = z305.z.enum(["alpha.scalar.function.invention.chunk", "alpha.vector.function.invention.chunk"]).meta({ title: "functions.inventions.response.streaming.Object" });
var FunctionsInventionsStateStateSchema = z305.z.union([FunctionsInventionsStateAlphaScalarBranchStateSchema.extend({
  type: z305.z.literal("alpha.scalar.branch.function")
}), FunctionsInventionsStateAlphaScalarLeafStateSchema.extend({
  type: z305.z.literal("alpha.scalar.leaf.function")
}), FunctionsInventionsStateAlphaVectorBranchStateSchema.extend({
  type: z305.z.literal("alpha.vector.branch.function")
}), FunctionsInventionsStateAlphaVectorLeafStateSchema.extend({
  type: z305.z.literal("alpha.vector.leaf.function")
})]).meta({ title: "functions.inventions.state.State" });
var FunctionsRemoteFunctionPathSchema = z305.z.object({
  remote: FunctionsRemoteSchema,
  owner: z305.z.string(),
  repository: z305.z.string(),
  commit: z305.z.string()
}).meta({ title: "functions.RemoteFunctionPath" });

// src/functions/inventions/recursive/response/streaming/functionInventionChunk.ts
var FunctionsInventionsRecursiveResponseStreamingFunctionInventionChunkSchema = z305.z.object({
  index: z305.z.number().int().min(0).meta({ format: "uint64" }),
  id: z305.z.string(),
  completions: z305.z.array(FunctionsInventionsResponseStreamingAgentCompletionChunkSchema),
  state: FunctionsInventionsStateStateSchema.nullable().optional(),
  path: FunctionsRemoteFunctionPathSchema.nullable().optional(),
  function: FunctionsFullRemoteFunctionSchema.nullable().optional(),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }),
  object: FunctionsInventionsResponseStreamingObjectSchema,
  usage: AgentCompletionsResponseUsageSchema.nullable().optional(),
  error: ResponseErrorSchema.nullable().optional()
}).meta({ title: "functions.inventions.recursive.response.streaming.FunctionInventionChunk" });
var FunctionsInventionsRecursiveResponseStreamingObjectSchema = z305.z.enum(["alpha.scalar.function.invention.recursive.chunk", "alpha.vector.function.invention.recursive.chunk"]).meta({ title: "functions.inventions.recursive.response.streaming.Object" });

// src/functions/inventions/recursive/response/streaming/functionInventionRecursiveChunk.ts
var FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunkSchema = z305.z.object({
  id: z305.z.string(),
  inventions: z305.z.array(FunctionsInventionsRecursiveResponseStreamingFunctionInventionChunkSchema),
  inventions_errors: z305.z.boolean().nullable().optional(),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }),
  object: FunctionsInventionsRecursiveResponseStreamingObjectSchema,
  usage: AgentCompletionsResponseUsageSchema.nullable().optional()
}).meta({ title: "functions.inventions.recursive.response.streaming.FunctionInventionRecursiveChunk" });

// src/functions/inventions/response/streaming/agentCompletionChunkMerged.ts
function functionsInventionsResponseStreamingAgentCompletionChunkMerged(a, b) {
  let changed = false;
  const [messages, c1] = agentCompletionsResponseStreamingMessageChunkMergedList(a.messages, b.messages);
  if (c1) changed = true;
  let usage = a.usage;
  if (a.usage != null && b.usage != null) {
    const [merged, c] = agentCompletionsResponseUsageMerged(a.usage, b.usage);
    usage = merged;
    if (c) changed = true;
  } else if (b.usage != null) {
    usage = b.usage;
    changed = true;
  }
  let error = a.error;
  if (a.error == null && b.error != null) {
    error = b.error;
    changed = true;
  }
  if (!changed) return [a, false];
  return [{
    index: a.index,
    id: a.id,
    created: a.created,
    messages,
    object: a.object,
    ...usage != null ? { usage } : {},
    upstream: a.upstream,
    ...error != null ? { error } : {}
  }, true];
}
function functionsInventionsResponseStreamingAgentCompletionChunkMergedList(a, b) {
  let changed = false;
  const result = [...a];
  for (const bItem of b) {
    const existingIdx = result.findIndex((x) => x.index === bItem.index);
    if (existingIdx !== -1) {
      const [merged, c] = functionsInventionsResponseStreamingAgentCompletionChunkMerged(result[existingIdx], bItem);
      if (c) {
        result[existingIdx] = merged;
        changed = true;
      }
    } else {
      result.push(bItem);
      changed = true;
    }
  }
  if (!changed) return [a, false];
  return [result, true];
}

// src/functions/inventions/recursive/response/streaming/functionInventionChunkMerged.ts
function functionsInventionsRecursiveResponseStreamingFunctionInventionChunkMerged(a, b) {
  let changed = false;
  const [completions, c1] = functionsInventionsResponseStreamingAgentCompletionChunkMergedList(a.completions, b.completions);
  if (c1) changed = true;
  let state = a.state;
  if (b.state != null) {
    state = b.state;
    changed = true;
  }
  let path = a.path;
  if (b.path != null) {
    path = b.path;
    changed = true;
  }
  let fn = a.function;
  if (b.function != null) {
    fn = b.function;
    changed = true;
  }
  let usage = a.usage;
  if (a.usage != null && b.usage != null) {
    const [merged, c] = agentCompletionsResponseUsageMerged(a.usage, b.usage);
    usage = merged;
    if (c) changed = true;
  } else if (b.usage != null) {
    usage = b.usage;
    changed = true;
  }
  let error = a.error;
  if (b.error != null) {
    error = b.error;
    changed = true;
  }
  if (!changed) return [a, false];
  return [{
    index: a.index,
    id: a.id,
    completions,
    ...state != null ? { state } : {},
    ...path != null ? { path } : {},
    ...fn != null ? { function: fn } : {},
    created: a.created,
    object: a.object,
    ...usage != null ? { usage } : {},
    ...error != null ? { error } : {}
  }, true];
}
function functionsInventionsRecursiveResponseStreamingFunctionInventionChunkMergedList(a, b) {
  let changed = false;
  const result = [...a];
  for (const bItem of b) {
    const existingIdx = result.findIndex((x) => x.index === bItem.index);
    if (existingIdx !== -1) {
      const [merged, c] = functionsInventionsRecursiveResponseStreamingFunctionInventionChunkMerged(result[existingIdx], bItem);
      if (c) {
        result[existingIdx] = merged;
        changed = true;
      }
    } else {
      result.push(bItem);
      changed = true;
    }
  }
  if (!changed) return [a, false];
  return [result, true];
}

// src/functions/inventions/recursive/response/streaming/functionInventionRecursiveChunkMerged.ts
function functionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunkMerged(a, b) {
  let changed = false;
  const [inventions, c1] = functionsInventionsRecursiveResponseStreamingFunctionInventionChunkMergedList(a.inventions, b.inventions);
  if (c1) changed = true;
  let inventions_errors = a.inventions_errors;
  if (b.inventions_errors === true) {
    if (a.inventions_errors !== true) changed = true;
    inventions_errors = true;
  }
  let usage = a.usage;
  if (a.usage != null && b.usage != null) {
    const [merged, c] = agentCompletionsResponseUsageMerged(a.usage, b.usage);
    usage = merged;
    if (c) changed = true;
  } else if (b.usage != null) {
    usage = b.usage;
    changed = true;
  }
  if (!changed) return [a, false];
  return [{
    id: a.id,
    inventions,
    ...inventions_errors != null ? { inventions_errors } : {},
    created: a.created,
    object: a.object,
    ...usage != null ? { usage } : {}
  }, true];
}
var FunctionsInventionsResponseUnaryAgentCompletionSchema = z305.z.object({
  index: z305.z.number().int().min(0).meta({ format: "uint64" }),
  id: z305.z.string(),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }),
  messages: z305.z.array(AgentCompletionsResponseUnaryMessageSchema),
  object: AgentCompletionsResponseUnaryObjectSchema.describe('The object type (always "agent.completion").'),
  usage: AgentCompletionsResponseUsageSchema,
  upstream: AgentUpstreamSchema.describe("Upstream provider"),
  error: ResponseErrorSchema.nullable().describe("Error details if this completion failed.").optional()
}).describe("A complete agent completion response.").meta({ title: "functions.inventions.response.unary.AgentCompletion" });
var FunctionsInventionsResponseUnaryObjectSchema = z305.z.enum(["alpha.scalar.function.invention", "alpha.vector.function.invention"]).meta({ title: "functions.inventions.response.unary.Object" });

// src/functions/inventions/recursive/response/unary/functionInvention.ts
var FunctionsInventionsRecursiveResponseUnaryFunctionInventionSchema = z305.z.object({
  index: z305.z.number().int().min(0).meta({ format: "uint64" }),
  id: z305.z.string(),
  completions: z305.z.array(FunctionsInventionsResponseUnaryAgentCompletionSchema),
  state: FunctionsInventionsStateStateSchema,
  path: FunctionsRemoteFunctionPathSchema.nullable().optional(),
  function: FunctionsFullRemoteFunctionSchema.nullable().optional(),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }),
  object: FunctionsInventionsResponseUnaryObjectSchema,
  usage: AgentCompletionsResponseUsageSchema,
  error: ResponseErrorSchema.nullable().optional()
}).meta({ title: "functions.inventions.recursive.response.unary.FunctionInvention" });
var FunctionsInventionsRecursiveResponseUnaryObjectSchema = z305.z.enum(["alpha.scalar.function.invention.recursive", "alpha.vector.function.invention.recursive"]).meta({ title: "functions.inventions.recursive.response.unary.Object" });

// src/functions/inventions/recursive/response/unary/functionInventionRecursive.ts
var FunctionsInventionsRecursiveResponseUnaryFunctionInventionRecursiveSchema = z305.z.object({
  id: z305.z.string(),
  inventions: z305.z.array(FunctionsInventionsRecursiveResponseUnaryFunctionInventionSchema),
  inventions_errors: z305.z.boolean(),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }),
  object: FunctionsInventionsRecursiveResponseUnaryObjectSchema,
  usage: AgentCompletionsResponseUsageSchema
}).meta({ title: "functions.inventions.recursive.response.unary.FunctionInventionRecursive" });
var FunctionsInventionsRecursiveRequestFunctionInventionRecursiveCreateParamsStreamingSchema = FunctionsInventionsRecursiveRequestFunctionInventionRecursiveCreateParamsSchema.extend({
  stream: z305__default.default.literal(true)
});
var FunctionsInventionsRecursiveRequestFunctionInventionRecursiveCreateParamsUnarySchema = FunctionsInventionsRecursiveRequestFunctionInventionRecursiveCreateParamsSchema.extend({
  stream: z305__default.default.literal(false).optional().nullable()
});
function functionsInventionsRecursiveCreateFunctionInventionRecursive(client, body, options) {
  if (body.stream) {
    return client.post_streaming(
      "/functions/inventions/recursive",
      body,
      options
    );
  }
  return client.post_unary(
    "/functions/inventions/recursive",
    body,
    options
  );
}
var FunctionsInventionsRequestFunctionInventionCreateParamsSchema = z305.z.object({
  remote: FunctionsRemoteSchema.nullable().optional(),
  overwrite: z305.z.boolean().nullable().optional(),
  state: FunctionsInventionsStateParamsStateSchema,
  provider: AgentCompletionsRequestProviderSchema.nullable().optional(),
  agent: AgentCompletionsRequestAgentSchema,
  agents: z305.z.array(AgentCompletionsRequestAgentSchema).nullable().optional(),
  seed: z305.z.number().int().meta({ format: "int64" }).nullable().optional(),
  stream: z305.z.boolean().nullable().optional(),
  max_step_retries: z305.z.number().int().min(0).meta({ format: "uint32" }).nullable().describe("Maximum number of retries per invention step.\nEach step is one agent completion (which itself may loop internally\nvia tool calls). If the step's validation still fails after the\nagent loop ends, the step is retried up to this many times.\nDefaults to 3 if not specified.").optional(),
  mcp_server_authorization: z305.z.record(z305.z.string(), z305.z.string()).nullable().describe("Map from MCP server URL to authorization header value.").optional()
}).meta({ title: "functions.inventions.request.FunctionInventionCreateParams" });
var FunctionsInventionsResponseStreamingFunctionInventionChunkSchema = z305.z.object({
  id: z305.z.string(),
  completions: z305.z.array(FunctionsInventionsResponseStreamingAgentCompletionChunkSchema),
  state: FunctionsInventionsStateStateSchema.nullable().optional(),
  path: FunctionsRemoteFunctionPathSchema.nullable().optional(),
  function: FunctionsFullRemoteFunctionSchema.nullable().optional(),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }),
  object: FunctionsInventionsResponseStreamingObjectSchema,
  usage: AgentCompletionsResponseUsageSchema.nullable().optional(),
  error: ResponseErrorSchema.nullable().optional()
}).meta({ title: "functions.inventions.response.streaming.FunctionInventionChunk" });

// src/functions/inventions/response/streaming/functionInventionChunkMerged.ts
function functionsInventionsResponseStreamingFunctionInventionChunkMerged(a, b) {
  let changed = false;
  const [completions, c1] = functionsInventionsResponseStreamingAgentCompletionChunkMergedList(a.completions, b.completions);
  if (c1) changed = true;
  let state = a.state;
  if (b.state != null) {
    state = b.state;
    changed = true;
  }
  let path = a.path;
  if (b.path != null) {
    path = b.path;
    changed = true;
  }
  let fn = a.function;
  if (b.function != null) {
    fn = b.function;
    changed = true;
  }
  let usage = a.usage;
  if (a.usage != null && b.usage != null) {
    const [merged, c] = agentCompletionsResponseUsageMerged(a.usage, b.usage);
    usage = merged;
    if (c) changed = true;
  } else if (b.usage != null) {
    usage = b.usage;
    changed = true;
  }
  let error = a.error;
  if (b.error != null) {
    error = b.error;
    changed = true;
  }
  if (!changed) return [a, false];
  return [{
    id: a.id,
    completions,
    ...state != null ? { state } : {},
    ...path != null ? { path } : {},
    ...fn != null ? { function: fn } : {},
    created: a.created,
    object: a.object,
    ...usage != null ? { usage } : {},
    ...error != null ? { error } : {}
  }, true];
}
var FunctionsInventionsResponseUnaryFunctionInventionSchema = z305.z.object({
  id: z305.z.string(),
  completions: z305.z.array(FunctionsInventionsResponseUnaryAgentCompletionSchema),
  state: FunctionsInventionsStateStateSchema,
  path: FunctionsRemoteFunctionPathSchema.nullable().optional(),
  function: FunctionsFullRemoteFunctionSchema.nullable().optional(),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }),
  object: FunctionsInventionsResponseUnaryObjectSchema,
  usage: AgentCompletionsResponseUsageSchema,
  error: ResponseErrorSchema.nullable().optional()
}).meta({ title: "functions.inventions.response.unary.FunctionInvention" });
var FunctionsInventionsStateParamsSchema = z305.z.object({
  depth: z305.z.number().int().min(0).meta({ format: "uint64" }),
  min_branch_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  max_branch_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  min_leaf_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  max_leaf_width: z305.z.number().int().min(0).meta({ format: "uint64" }),
  name: z305.z.string(),
  spec: z305.z.string()
}).meta({ title: "functions.inventions.state.Params" });
var FunctionsInventionsDescriptionObjectSchema = z305.z.object({
  description: z305.z.string()
}).meta({ title: "functions.inventions.DescriptionObject" });
var FunctionsInventionsEssayObjectSchema = z305.z.object({
  essay: z305.z.string()
}).meta({ title: "functions.inventions.EssayObject" });
var FunctionsInventionsEssayTasksObjectSchema = z305.z.object({
  essay_tasks: z305.z.string()
}).meta({ title: "functions.inventions.EssayTasksObject" });
var FunctionsInventionsIndexObjectSchema = z305.z.object({
  index: z305.z.number().int().min(0).meta({ format: "uint64" })
}).meta({ title: "functions.inventions.IndexObject" });
var FunctionsInventionsTasksLengthObjectSchema = z305.z.object({
  tasks_length: z305.z.number().int().min(0).meta({ format: "uint64" })
}).meta({ title: "functions.inventions.TasksLengthObject" });
var FunctionsInventionsRequestFunctionInventionCreateParamsStreamingSchema = FunctionsInventionsRequestFunctionInventionCreateParamsSchema.extend({
  stream: z305__default.default.literal(true)
});
var FunctionsInventionsRequestFunctionInventionCreateParamsUnarySchema = FunctionsInventionsRequestFunctionInventionCreateParamsSchema.extend({
  stream: z305__default.default.literal(false).optional().nullable()
});
function functionsInventionsCreateFunctionInvention(client, body, options) {
  if (body.stream) {
    return client.post_streaming(
      "/functions/inventions",
      body,
      options
    );
  }
  return client.post_unary(
    "/functions/inventions",
    body,
    options
  );
}
var FunctionsProfilesComputationsRequestTargetSchema = z305.z.union([z305.z.object({
  value: z305.z.number().meta({ format: "double" }),
  type: z305.z.literal("scalar")
}), z305.z.object({
  value: z305.z.array(z305.z.number().meta({ format: "double" })),
  type: z305.z.literal("vector")
}), z305.z.object({
  value: z305.z.number().int().min(0).meta({ format: "uint" }),
  type: z305.z.literal("vector_winner")
})]).meta({ title: "functions.profiles.computations.request.Target" });

// src/functions/profiles/computations/request/datasetItem.ts
var FunctionsProfilesComputationsRequestDatasetItemSchema = z305.z.object({
  input: FunctionsExpressionInputValueSchema,
  target: FunctionsProfilesComputationsRequestTargetSchema
}).meta({ title: "functions.profiles.computations.request.DatasetItem" });
var FunctionsProfilesComputationsRequestFunctionInlineRequestBodySchema = z305.z.object({
  function: FunctionsInlineFunctionSchema,
  retry_token: z305.z.string().nullable().optional(),
  from_cache: z305.z.boolean().nullable().optional(),
  max_retries: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().optional(),
  n: z305.z.number().int().min(0).meta({ format: "uint64" }),
  dataset: z305.z.array(FunctionsProfilesComputationsRequestDatasetItemSchema),
  ensemble: VectorCompletionsRequestEnsembleSchema,
  provider: AgentCompletionsRequestProviderSchema.nullable().optional(),
  seed: z305.z.number().int().meta({ format: "int64" }).nullable().optional(),
  stream: z305.z.boolean().nullable().optional(),
  mcp_server_authorization: z305.z.record(z305.z.string(), z305.z.string()).nullable().describe("Map from MCP server URL to authorization header value.").optional()
}).meta({ title: "functions.profiles.computations.request.FunctionInlineRequestBody" });
var FunctionsProfilesComputationsRequestFunctionRemoteRequestBodySchema = z305.z.object({
  retry_token: z305.z.string().nullable().optional(),
  from_cache: z305.z.boolean().nullable().optional(),
  max_retries: z305.z.number().int().min(0).meta({ format: "uint64" }).nullable().optional(),
  n: z305.z.number().int().min(0).meta({ format: "uint64" }),
  dataset: z305.z.array(FunctionsProfilesComputationsRequestDatasetItemSchema),
  ensemble: VectorCompletionsRequestEnsembleSchema,
  provider: AgentCompletionsRequestProviderSchema.nullable().optional(),
  seed: z305.z.number().int().meta({ format: "int64" }).nullable().optional(),
  stream: z305.z.boolean().nullable().optional(),
  mcp_server_authorization: z305.z.record(z305.z.string(), z305.z.string()).nullable().describe("Map from MCP server URL to authorization header value.").optional()
}).meta({ title: "functions.profiles.computations.request.FunctionRemoteRequestBody" });

// src/functions/profiles/computations/request/functionProfileComputationCreateParams.ts
var FunctionsProfilesComputationsRequestFunctionProfileComputationCreateParamsSchema = z305.z.union([FunctionsProfilesComputationsRequestFunctionInlineRequestBodySchema, FunctionsProfilesComputationsRequestFunctionRemoteRequestBodySchema]).meta({ title: "functions.profiles.computations.request.FunctionProfileComputationCreateParams" });
var FunctionsProfilesComputationsRequestFunctionRemoteRequestPathSchema = z305.z.object({
  fremote: FunctionsRemoteSchema,
  fowner: z305.z.string(),
  frepository: z305.z.string(),
  fcommit: z305.z.string().nullable().optional()
}).meta({ title: "functions.profiles.computations.request.FunctionRemoteRequestPath" });
var FunctionsProfilesComputationsRequestRequestSchema = z305.z.union([z305.z.object({
  body: FunctionsProfilesComputationsRequestFunctionInlineRequestBodySchema
}), z305.z.object({
  path: FunctionsProfilesComputationsRequestFunctionRemoteRequestPathSchema,
  body: FunctionsProfilesComputationsRequestFunctionRemoteRequestBodySchema
})]).meta({ title: "functions.profiles.computations.request.Request" });
var FunctionsProfilesComputationsResponseStreamingFunctionExecutionChunkSchema = z305.z.object({
  index: z305.z.number().int().min(0).meta({ format: "uint64" }),
  dataset: z305.z.number().int().min(0).meta({ format: "uint64" }),
  n: z305.z.number().int().min(0).meta({ format: "uint64" }),
  retry: z305.z.number().int().min(0).meta({ format: "uint64" }),
  id: z305.z.string(),
  tasks: z305.z.array(FunctionsExecutionsResponseStreamingTaskChunkSchema),
  tasks_errors: z305.z.boolean().nullable().optional(),
  reasoning: FunctionsExecutionsResponseStreamingReasoningSummaryChunkSchema.nullable().optional(),
  output: FunctionsExpressionTaskOutputOwnedSchema.nullable().optional(),
  error: ResponseErrorSchema.nullable().optional(),
  retry_token: z305.z.string().nullable().optional(),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }),
  function: z305.z.string().nullable().optional(),
  profile: z305.z.string().nullable().optional(),
  object: FunctionsExecutionsResponseStreamingObjectSchema,
  usage: AgentCompletionsResponseUsageSchema.nullable().optional()
}).meta({ title: "functions.profiles.computations.response.streaming.FunctionExecutionChunk" });
var FunctionsProfilesComputationsResponseFittingStatsSchema = z305.z.object({
  loss: z305.z.number().meta({ format: "double" }),
  executions: z305.z.number().int().min(0).meta({ format: "uint" }),
  starts: z305.z.number().int().min(0).meta({ format: "uint" }),
  rounds: z305.z.number().int().min(0).meta({ format: "uint" }),
  errors: z305.z.number().int().min(0).meta({ format: "uint" })
}).meta({ title: "functions.profiles.computations.response.FittingStats" });
var FunctionsProfilesComputationsResponseStreamingObjectSchema = z305.z.literal("function.profile.computation.chunk").meta({ title: "functions.profiles.computations.response.streaming.Object" });

// src/functions/profiles/computations/response/streaming/functionProfileComputationChunk.ts
var FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunkSchema = z305.z.object({
  id: z305.z.string(),
  executions: z305.z.array(FunctionsProfilesComputationsResponseStreamingFunctionExecutionChunkSchema),
  executions_errors: z305.z.boolean().nullable().optional(),
  profile: FunctionsInlineTasksProfileSchema.nullable().optional(),
  fitting_stats: FunctionsProfilesComputationsResponseFittingStatsSchema.nullable().optional(),
  retry_token: z305.z.string().nullable().optional(),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }),
  function: z305.z.string().nullable().optional(),
  object: FunctionsProfilesComputationsResponseStreamingObjectSchema,
  usage: AgentCompletionsResponseUsageSchema.nullable().optional()
}).meta({ title: "functions.profiles.computations.response.streaming.FunctionProfileComputationChunk" });

// src/functions/profiles/computations/response/streaming/functionExecutionChunkMerged.ts
function functionsProfilesComputationsResponseStreamingFunctionExecutionChunkMerged(a, b) {
  let changed = false;
  const [tasks, c1] = functionsExecutionsResponseStreamingTaskChunkMergedList(a.tasks, b.tasks);
  if (c1) changed = true;
  let tasks_errors = a.tasks_errors;
  if (b.tasks_errors === true) {
    if (a.tasks_errors !== true) changed = true;
    tasks_errors = true;
  }
  let reasoning = a.reasoning;
  if (a.reasoning != null && b.reasoning != null) {
    const [merged, c] = functionsExecutionsResponseStreamingReasoningSummaryChunkMerged(a.reasoning, b.reasoning);
    reasoning = merged;
    if (c) changed = true;
  } else if (b.reasoning != null) {
    reasoning = b.reasoning;
    changed = true;
  }
  let output = a.output;
  if (b.output != null) {
    output = b.output;
    changed = true;
  }
  let retry_token = a.retry_token;
  if (b.retry_token != null) {
    retry_token = b.retry_token;
    changed = true;
  }
  let error = a.error;
  if (b.error != null) {
    error = b.error;
    changed = true;
  }
  let usage = a.usage;
  if (a.usage != null && b.usage != null) {
    const [merged, c] = agentCompletionsResponseUsageMerged(a.usage, b.usage);
    usage = merged;
    if (c) changed = true;
  } else if (b.usage != null) {
    usage = b.usage;
    changed = true;
  }
  if (!changed) return [a, false];
  return [{
    index: a.index,
    dataset: a.dataset,
    n: a.n,
    retry: a.retry,
    id: a.id,
    tasks,
    ...tasks_errors != null ? { tasks_errors } : {},
    ...reasoning != null ? { reasoning } : {},
    ...output != null ? { output } : {},
    ...error != null ? { error } : {},
    ...retry_token != null ? { retry_token } : {},
    created: a.created,
    function: a.function,
    profile: a.profile,
    object: a.object,
    ...usage != null ? { usage } : {}
  }, true];
}
function functionsProfilesComputationsResponseStreamingFunctionExecutionChunkMergedList(a, b) {
  let changed = false;
  const result = [...a];
  for (const bItem of b) {
    const existingIdx = result.findIndex((x) => x.index === bItem.index);
    if (existingIdx !== -1) {
      const [merged, c] = functionsProfilesComputationsResponseStreamingFunctionExecutionChunkMerged(result[existingIdx], bItem);
      if (c) {
        result[existingIdx] = merged;
        changed = true;
      }
    } else {
      result.push(bItem);
      changed = true;
    }
  }
  if (!changed) return [a, false];
  return [result, true];
}

// src/functions/profiles/computations/response/streaming/functionProfileComputationChunkMerged.ts
function functionsProfilesComputationsResponseStreamingFunctionProfileComputationChunkMerged(a, b) {
  let changed = false;
  const [executions, c1] = functionsProfilesComputationsResponseStreamingFunctionExecutionChunkMergedList(a.executions, b.executions);
  if (c1) changed = true;
  let executions_errors = a.executions_errors;
  if (b.executions_errors === true) {
    if (a.executions_errors !== true) changed = true;
    executions_errors = true;
  }
  let profile = a.profile;
  if (b.profile != null) {
    profile = b.profile;
    changed = true;
  }
  let fitting_stats = a.fitting_stats;
  if (b.fitting_stats != null) {
    fitting_stats = b.fitting_stats;
    changed = true;
  }
  let retry_token = a.retry_token;
  if (b.retry_token != null) {
    retry_token = b.retry_token;
    changed = true;
  }
  let usage = a.usage;
  if (a.usage != null && b.usage != null) {
    const [merged, c] = agentCompletionsResponseUsageMerged(a.usage, b.usage);
    usage = merged;
    if (c) changed = true;
  } else if (b.usage != null) {
    usage = b.usage;
    changed = true;
  }
  if (!changed) return [a, false];
  return [{
    ...a,
    executions,
    ...executions_errors != null ? { executions_errors } : {},
    ...profile != null ? { profile } : {},
    ...fitting_stats != null ? { fitting_stats } : {},
    ...retry_token != null ? { retry_token } : {},
    ...usage != null ? { usage } : {}
  }, true];
}
var FunctionsProfilesComputationsResponseUnaryFunctionExecutionSchema = z305.z.object({
  index: z305.z.number().int().min(0).meta({ format: "uint64" }),
  dataset: z305.z.number().int().min(0).meta({ format: "uint64" }),
  n: z305.z.number().int().min(0).meta({ format: "uint64" }),
  retry: z305.z.number().int().min(0).meta({ format: "uint64" }),
  id: z305.z.string().describe("Unique identifier for this execution."),
  tasks: z305.z.array(FunctionsExecutionsResponseUnaryTaskSchema).describe("Results from each task in the function."),
  tasks_errors: z305.z.boolean().describe("Whether any tasks encountered errors."),
  reasoning: FunctionsExecutionsResponseUnaryReasoningSummarySchema.nullable().describe("Reasoning summary if reasoning was enabled.").optional(),
  output: FunctionsExpressionTaskOutputOwnedSchema.describe("The final output (scalar or vector score)."),
  error: ResponseErrorSchema.nullable().describe("Error details if the execution failed.").optional(),
  retry_token: z305.z.string().nullable().describe("Token for retrying this execution with cached votes.").optional(),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Unix timestamp when the execution was created."),
  function: z305.z.string().nullable().describe("ID of the function used (if remote).").optional(),
  profile: z305.z.string().nullable().describe("ID of the profile used (if remote).").optional(),
  object: FunctionsExecutionsResponseUnaryObjectSchema.describe("Object type identifier."),
  usage: AgentCompletionsResponseUsageSchema.describe("Aggregated token and cost usage.")
}).describe("A complete function execution response (non-streaming).").meta({ title: "functions.profiles.computations.response.unary.FunctionExecution" });
var FunctionsProfilesComputationsResponseUnaryObjectSchema = z305.z.literal("function.profile.computation").meta({ title: "functions.profiles.computations.response.unary.Object" });

// src/functions/profiles/computations/response/unary/functionProfileComputation.ts
var FunctionsProfilesComputationsResponseUnaryFunctionProfileComputationSchema = z305.z.object({
  id: z305.z.string(),
  executions: z305.z.array(FunctionsProfilesComputationsResponseUnaryFunctionExecutionSchema),
  executions_errors: z305.z.boolean(),
  profile: FunctionsInlineTasksProfileSchema,
  fitting_stats: FunctionsProfilesComputationsResponseFittingStatsSchema,
  retry_token: z305.z.string().nullable().optional(),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }),
  function: z305.z.string().nullable().optional(),
  object: FunctionsProfilesComputationsResponseUnaryObjectSchema,
  usage: AgentCompletionsResponseUsageSchema
}).meta({ title: "functions.profiles.computations.response.unary.FunctionProfileComputation" });
var FunctionsProfilesComputationsRetryTokenSchema = z305.z.array(z305.z.string().nullable()).meta({ title: "functions.profiles.computations.RetryToken" });

// src/functions/profiles/computations/http.ts
function buildComputationPath(request) {
  if (!("path" in request)) {
    return "/functions/profiles/compute";
  }
  const { path } = request;
  let url = `/functions/${path.fremote}/${path.fowner}/${path.frepository}`;
  if (path.fcommit != null) url += `/${path.fcommit}`;
  url += "/profiles/compute";
  return url;
}
function functionsProfilesComputationsComputeProfile(client, request, options) {
  const path = buildComputationPath(request);
  if (request.body.stream) {
    return client.post_streaming(
      path,
      request.body,
      options
    );
  }
  return client.post_unary(
    path,
    request.body,
    options
  );
}
var FunctionsRemoteAutoProfileSchema = z305.z.object({
  description: z305.z.string().describe("Human-readable description of the profile."),
  ensemble: VectorCompletionsRequestEnsembleSchema.describe("The ensemble to use for all vector completion tasks."),
  profile: VectorCompletionsRequestProfileSchema.describe("Weights for each agent in the ensemble.")
}).describe("A remote auto profile with full metadata.\n\nApplies a single ensemble and weights to every vector completion task\nin the function, with equal task weights.").meta({ title: "functions.RemoteAutoProfile" });
var FunctionsRemoteTasksProfileSchema = z305.z.object({
  description: z305.z.string().describe("Human-readable description of the profile."),
  tasks: z305.z.array(FunctionsTaskProfileSchema).describe("Configuration for each task in the corresponding Function."),
  profile: VectorCompletionsRequestProfileSchema.describe("Weights for each Task in the corresponding Function.\n\nMust have the same length as `tasks`. Can be either:\n- A vector of decimals (legacy representation), or\n- A vector of objects with `weight` and optional `invert` fields.")
}).describe("A remote tasks-based profile with full metadata.\n\nStored as `profile.json` in repositories and referenced by\n`remote/owner/repository`.").meta({ title: "functions.RemoteTasksProfile" });

// src/functions/profiles/getProfile.ts
var FunctionsProfilesGetProfileSchema = z305.z.union([FunctionsRemoteTasksProfileSchema.describe("Tasks-based profile with per-task configuration."), FunctionsRemoteAutoProfileSchema.describe("Auto profile that applies a single ensemble+weights to all vector completion tasks.")]).and(z305.z.object({
  remote: FunctionsRemoteSchema,
  owner: z305.z.string(),
  repository: z305.z.string(),
  commit: z305.z.string()
})).describe("A remote profile, either tasks-based or auto.").meta({ title: "functions.profiles.GetProfile" });
var FunctionsProfilesListProfileItemSchema = z305.z.object({
  remote: FunctionsRemoteSchema.describe("The remote source where the profile is hosted."),
  owner: z305.z.string().describe("Repository owner."),
  repository: z305.z.string().describe("Repository name."),
  commit: z305.z.string().describe("Git commit SHA.")
}).describe("A profile in a list response.").meta({ title: "functions.profiles.ListProfileItem" });

// src/functions/profiles/listProfile.ts
var FunctionsProfilesListProfileSchema = z305.z.object({
  data: z305.z.array(FunctionsProfilesListProfileItemSchema).describe("List of available profiles.")
}).describe("Response from listing profiles.").meta({ title: "functions.profiles.ListProfile" });
var FunctionsProfilesListProfilesSourceSchema = z305.z.enum(["all", "mock", "filesystem", "objectiveai"]).describe("Source filter for listing profiles.").meta({ title: "functions.profiles.ListProfilesSource" });

// src/functions/profiles/listProfilesQueryParameters.ts
var FunctionsProfilesListProfilesQueryParametersSchema = z305.z.object({
  source: FunctionsProfilesListProfilesSourceSchema.nullable().describe("Optional source filter for listing profiles.").optional()
}).describe("Query parameters for the list profiles endpoint.").meta({ title: "functions.profiles.ListProfilesQueryParameters" });
var FunctionsProfilesUsageProfileSchema = z305.z.object({
  requests: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Total number of requests made with this profile."),
  completion_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Total completion tokens used."),
  prompt_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Total prompt tokens used."),
  total_cost: z305.z.number().meta({ format: "double" }).describe("Total cost incurred.")
}).describe("Usage statistics for a profile.").meta({ title: "functions.profiles.UsageProfile" });

// src/functions/profiles/http.ts
function functionsProfilesListProfiles(client, source, options) {
  const path = source != null ? `/functions/profiles?source=${source}` : "/functions/profiles";
  return client.get_unary(path, void 0, options);
}
function functionsProfilesGetProfile(client, remote, owner, repository, commit, options) {
  const path = commit != null ? `/functions/profiles/${remote}/${owner}/${repository}/${commit}` : `/functions/profiles/${remote}/${owner}/${repository}`;
  return client.get_unary(path, void 0, options);
}
function functionsProfilesGetProfileUsage(client, premote, powner, prepository, pcommit, options) {
  const path = pcommit != null ? `/functions/profiles/${premote}/${powner}/${prepository}/${pcommit}/usage` : `/functions/profiles/${premote}/${powner}/${prepository}/usage`;
  return client.get_unary(path, void 0, options);
}
var FunctionsAlphaInlineFunctionSchema = z305.z.union([FunctionsAlphaScalarInlineFunctionSchema, FunctionsAlphaVectorInlineFunctionSchema]).meta({ title: "functions.AlphaInlineFunction" });
var FunctionsPlaceholderScalarFunctionTaskSchema = z305.z.object({
  input_schema: FunctionsExpressionInputSchemaSchema.describe("JSON Schema defining the expected input structure."),
  input: FunctionsExpressionInputValueSchema.describe("The resolved input."),
  output: FunctionsExpressionExpressionSchema.describe("Expression to transform the fixed 0.5 output.")
}).describe("A compiled placeholder scalar function task.\n\nAlways produces `Scalar(0.5)` before the output expression\nis applied.").meta({ title: "functions.PlaceholderScalarFunctionTask" });
var FunctionsPlaceholderVectorFunctionTaskSchema = z305.z.object({
  input_schema: FunctionsExpressionInputSchemaSchema.describe("JSON Schema defining the expected input structure."),
  output_length: FunctionsExpressionExpressionSchema.describe("Expression computing the expected output vector length."),
  input_split: FunctionsExpressionExpressionSchema.describe("Expression transforming input into sub-inputs for swiss system."),
  input_merge: FunctionsExpressionExpressionSchema.describe("Expression merging sub-inputs back into one input."),
  input: FunctionsExpressionInputValueSchema.describe("The resolved input."),
  output: FunctionsExpressionExpressionSchema.describe("Expression to transform the equalized vector output.")
}).describe("A compiled placeholder vector function task.\n\nAlways produces `Vector(vec![1/N; output_length])` before\nthe output expression is applied.").meta({ title: "functions.PlaceholderVectorFunctionTask" });
var FunctionsScalarFunctionTaskSchema = z305.z.object({
  remote: FunctionsRemoteSchema.describe("The remote source where the function is hosted."),
  owner: z305.z.string().describe("Repository owner."),
  repository: z305.z.string().describe("Repository name."),
  commit: z305.z.string().describe("Git commit SHA for the function version."),
  input: FunctionsExpressionInputValueSchema.describe("The resolved input to pass to the function."),
  output: FunctionsExpressionExpressionSchema.describe("Expression to transform the task result into a valid function output.\n\nReceives `output` as the nested function's result (Scalar or Vector).\nMust return a `TaskOutputOwned` valid for the parent function's type (scalar or vector).\nSee [`ScalarFunctionTaskExpression::output`] for full documentation.")
}).describe("A compiled scalar function task ready for execution.").meta({ title: "functions.ScalarFunctionTask" });
var FunctionsVectorCompletionTaskSchema = z305.z.object({
  messages: z305.z.array(AgentCompletionsMessageMessageSchema).describe("The resolved conversation messages."),
  responses: z305.z.array(AgentCompletionsMessageRichContentSchema).describe("The resolved response options the LLMs can vote for."),
  output: FunctionsExpressionExpressionSchema.describe("Expression to transform the task result into a valid function output.\n\nReceives `output` as the task's raw result (typically `Vector(scores)`).\nMust return a `TaskOutputOwned` valid for the parent function's type (scalar or vector).\nSee [`VectorCompletionTaskExpression::output`] for full documentation.")
}).describe("A compiled vector completion task ready for execution.").meta({ title: "functions.VectorCompletionTask" });
var FunctionsVectorFunctionTaskSchema = z305.z.object({
  remote: FunctionsRemoteSchema.describe("The remote source where the function is hosted."),
  owner: z305.z.string().describe("Repository owner."),
  repository: z305.z.string().describe("Repository name."),
  commit: z305.z.string().describe("Git commit SHA for the function version."),
  input: FunctionsExpressionInputValueSchema.describe("The resolved input to pass to the function."),
  output: FunctionsExpressionExpressionSchema.describe("Expression to transform the task result into a valid function output.\n\nReceives `output` as the nested function's result (Scalar or Vector).\nMust return a `TaskOutputOwned` valid for the parent function's type (scalar or vector).\nSee [`VectorFunctionTaskExpression::output`] for full documentation.")
}).describe("A compiled vector function task ready for execution.").meta({ title: "functions.VectorFunctionTask" });

// src/functions/task.ts
var FunctionsTaskSchema = z305.z.union([FunctionsScalarFunctionTaskSchema.extend({
  type: z305.z.literal("scalar.function")
}).describe("Calls a scalar function (produces a single score)."), FunctionsVectorFunctionTaskSchema.extend({
  type: z305.z.literal("vector.function")
}).describe("Calls a vector function (produces a vector of scores)."), FunctionsVectorCompletionTaskSchema.extend({
  type: z305.z.literal("vector.completion")
}).describe("Runs a vector completion."), FunctionsPlaceholderScalarFunctionTaskSchema.extend({
  type: z305.z.literal("placeholder.scalar.function")
}).describe("Placeholder scalar function (always outputs 0.5)."), FunctionsPlaceholderVectorFunctionTaskSchema.extend({
  type: z305.z.literal("placeholder.vector.function")
}).describe("Placeholder vector function (always outputs equalized vector).")]).describe("A compiled task ready for execution.\n\nProduced by compiling a [`TaskExpression`] against input data. All\nexpressions have been resolved to concrete values.").meta({ title: "functions.Task" });

// src/functions/compiledTask.ts
var FunctionsCompiledTaskSchema = z305.z.union([FunctionsTaskSchema.describe("A single task (no mapping)."), z305.z.array(FunctionsTaskSchema).describe("Multiple task instances from mapped execution.")]).describe("The result of compiling a task expression.\n\nTasks without a `map` field compile to a single task. Tasks with a `map`\nexpression are expanded into multiple tasks, one per integer index from\n0 to the evaluated count.").meta({ title: "functions.CompiledTask" });
var FunctionsFullInlineFunctionSchema = z305.z.union([FunctionsAlphaInlineFunctionSchema, FunctionsInlineFunctionSchema]).meta({ title: "functions.FullInlineFunction" });
var FunctionsFunctionSchema = z305.z.union([FunctionsRemoteFunctionSchema.describe("A remote function with metadata (description, schema, etc.)."), FunctionsInlineFunctionSchema.describe("An inline function definition without metadata.")]).describe("A Function definition, either remote or inline.\n\nFunctions are composable scoring pipelines that transform structured input\ninto scores. Each task has an `output` expression that transforms its raw result\ninto a `TaskOutputOwned`. The function's final output is the weighted average of\nall task outputs using profile weights.\n\nUse [`compile_tasks`](Self::compile_tasks) to preview how task expressions resolve\nfor given inputs.").meta({ title: "functions.Function" });
var FunctionsFunctionTypeSchema = z305.z.enum(["scalar.function", "vector.function"]).meta({ title: "functions.FunctionType" });
var FunctionsGetFunctionSchema = z305.z.union([z305.z.object({
  description: z305.z.string().describe("Human-readable description of what the function does."),
  input_schema: FunctionsExpressionInputSchemaSchema.describe("JSON Schema defining the expected input structure."),
  tasks: z305.z.array(FunctionsTaskExpressionSchema).describe("The list of tasks to execute. Tasks with a `map` expression are\nexpanded into multiple instances. Each instance is compiled with\n`map` set to the current integer index.\nReceives: `input`, `map` (if mapped)."),
  type: z305.z.literal("scalar.function")
}).describe("Produces a single score in [0, 1]."), z305.z.object({
  description: z305.z.string().describe("Human-readable description of what the function does."),
  input_schema: FunctionsExpressionInputSchemaSchema.describe("JSON Schema defining the expected input structure."),
  tasks: z305.z.array(FunctionsTaskExpressionSchema).describe("The list of tasks to execute. Tasks with a `map` expression are\nexpanded into multiple instances. Each instance is compiled with\n`map` set to the current integer index.\nReceives: `input`, `map` (if mapped)."),
  output_length: FunctionsExpressionExpressionSchema.describe("Expression computing the expected output vector length for task outputs.\nReceives: `input`."),
  input_split: FunctionsExpressionExpressionSchema.describe("Expression transforming input into an input array of the output_length\nWhen the Function is executed with any input from the array,\nThe output_length should be 1.\nReceives: `input`."),
  input_merge: FunctionsExpressionExpressionSchema.describe("Expression transforming an array of inputs computed by `input_split`\ninto a single Input object for the Function.\nReceives: `input` (as an array)."),
  type: z305.z.literal("vector.function")
}).describe("Produces a vector of scores that sums to 1.")]).and(z305.z.object({
  remote: FunctionsRemoteSchema,
  owner: z305.z.string(),
  repository: z305.z.string(),
  commit: z305.z.string()
})).describe("A remote function with full metadata.\n\nRemote functions are stored as `function.json` in repositories and\nreferenced by `remote/owner/repository`. They include documentation fields\nthat inline functions lack.").meta({ title: "functions.GetFunction" });
var FunctionsGetFunctionProfilePairSchema = z305.z.object({
  function: FunctionsGetFunctionSchema.describe("The function."),
  profile: FunctionsProfilesGetProfileSchema.describe("The profile.")
}).describe("Response from getting a function-profile pair.").meta({ title: "functions.GetFunctionProfilePair" });
var FunctionsListFunctionItemSchema = z305.z.object({
  remote: FunctionsRemoteSchema.describe("The remote source where the function is hosted."),
  owner: z305.z.string().describe("Repository owner."),
  repository: z305.z.string().describe("Repository name."),
  commit: z305.z.string().describe("Git commit SHA.")
}).describe("A function in a list response.").meta({ title: "functions.ListFunctionItem" });

// src/functions/listFunction.ts
var FunctionsListFunctionSchema = z305.z.object({
  data: z305.z.array(FunctionsListFunctionItemSchema).describe("List of available functions.")
}).describe("Response from listing functions.").meta({ title: "functions.ListFunction" });
var FunctionsListFunctionProfilePairItemSchema = z305.z.object({
  function: FunctionsListFunctionItemSchema.describe("The function."),
  profile: FunctionsProfilesListProfileItemSchema.describe("The profile.")
}).describe("A function-profile pair in a list response.").meta({ title: "functions.ListFunctionProfilePairItem" });

// src/functions/listFunctionProfilePair.ts
var FunctionsListFunctionProfilePairSchema = z305.z.object({
  data: z305.z.array(FunctionsListFunctionProfilePairItemSchema).describe("List of available function-profile pairs.")
}).describe("Response from listing function-profile pairs.").meta({ title: "functions.ListFunctionProfilePair" });
var FunctionsListFunctionProfilePairsSourceSchema = z305.z.literal("objectiveai").describe("Source filter for listing function-profile pairs.").meta({ title: "functions.ListFunctionProfilePairsSource" });

// src/functions/listFunctionProfilePairsQueryParameters.ts
var FunctionsListFunctionProfilePairsQueryParametersSchema = z305.z.object({
  source: FunctionsListFunctionProfilePairsSourceSchema.nullable().describe("Optional source filter for listing function-profile pairs.").optional()
}).describe("Query parameters for the list function-profile pairs endpoint.").meta({ title: "functions.ListFunctionProfilePairsQueryParameters" });
var FunctionsListFunctionsSourceSchema = z305.z.enum(["all", "mock", "filesystem", "objectiveai"]).describe("Source filter for listing functions.").meta({ title: "functions.ListFunctionsSource" });

// src/functions/listFunctionsQueryParameters.ts
var FunctionsListFunctionsQueryParametersSchema = z305.z.object({
  source: FunctionsListFunctionsSourceSchema.nullable().describe("Optional source filter for listing functions.").optional()
}).describe("Query parameters for the list functions endpoint.").meta({ title: "functions.ListFunctionsQueryParameters" });
var FunctionsRemoteProfileSchema = z305.z.union([FunctionsRemoteTasksProfileSchema.describe("Tasks-based profile with per-task configuration."), FunctionsRemoteAutoProfileSchema.describe("Auto profile that applies a single ensemble+weights to all vector completion tasks.")]).describe("A remote profile, either tasks-based or auto.").meta({ title: "functions.RemoteProfile" });

// src/functions/profile.ts
var FunctionsProfileSchema = z305.z.union([FunctionsRemoteProfileSchema.describe("A remote profile with metadata."), FunctionsInlineProfileSchema.describe("An inline profile definition.")]).describe("A Profile definition, either remote or inline.\n\nProfiles contain the weights and nested configurations needed to execute\na Function. They correspond to a Function's task structure.").meta({ title: "functions.Profile" });
var FunctionsUsageFunctionSchema = z305.z.object({
  requests: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Total number of requests made with this function."),
  completion_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Total completion tokens used."),
  prompt_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Total prompt tokens used."),
  total_cost: z305.z.number().meta({ format: "double" }).describe("Total cost incurred.")
}).describe("Usage statistics for a function.").meta({ title: "functions.UsageFunction" });
var FunctionsUsageFunctionProfilePairSchema = z305.z.object({
  requests: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Total number of requests made with this function-profile pair."),
  completion_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Total completion tokens used."),
  prompt_tokens: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Total prompt tokens used."),
  total_cost: z305.z.number().meta({ format: "double" }).describe("Total cost incurred.")
}).describe("Usage statistics for a function-profile pair.").meta({ title: "functions.UsageFunctionProfilePair" });

// src/functions/http.ts
function functionsListFunctions(client, source, options) {
  const path = source != null ? `/functions?source=${source}` : "/functions";
  return client.get_unary(path, void 0, options);
}
function functionsGetFunction(client, remote, owner, repository, commit, options) {
  const path = commit != null ? `/functions/${remote}/${owner}/${repository}/${commit}` : `/functions/${remote}/${owner}/${repository}`;
  return client.get_unary(path, void 0, options);
}
function functionsGetFunctionUsage(client, fremote, fowner, frepository, fcommit, options) {
  const path = fcommit != null ? `/functions/${fremote}/${fowner}/${frepository}/${fcommit}/usage` : `/functions/${fremote}/${fowner}/${frepository}/usage`;
  return client.get_unary(path, void 0, options);
}
function functionsListFunctionProfilePairs(client, source, options) {
  const path = source != null ? `/functions/profiles/pairs?source=${source}` : "/functions/profiles/pairs";
  return client.get_unary(path, void 0, options);
}
function functionsGetFunctionProfilePairUsage(client, fremote, fowner, frepository, fcommit, premote, powner, prepository, pcommit, options) {
  let path = `/functions/${fremote}/${fowner}/${frepository}`;
  if (fcommit != null) path += `/${fcommit}`;
  path += `/profiles/${premote}/${powner}/${prepository}`;
  if (pcommit != null) path += `/${pcommit}`;
  path += "/usage";
  return client.get_unary(path, void 0, options);
}
var VectorCompletionsCacheCacheVoteSchema = z305.z.object({
  vote: VectorCompletionsResponseVoteSchema.nullable().optional()
}).meta({ title: "vector.completions.cache.CacheVote" });
var VectorCompletionsCacheCacheVoteRequestOwnedSchema = z305.z.object({
  agent: AgentCompletionsRequestAgentSchema,
  agents: z305.z.array(AgentCompletionsRequestAgentSchema).nullable().optional(),
  messages: z305.z.array(AgentCompletionsMessageMessageSchema),
  responses: z305.z.array(AgentCompletionsMessageRichContentSchema)
}).meta({ title: "vector.completions.cache.CacheVoteRequestOwned" });
var VectorCompletionsCacheCacheVoteRequestRefSchema = z305.z.object({
  agent: AgentCompletionsRequestAgentSchema,
  agents: z305.z.array(AgentCompletionsRequestAgentSchema).nullable().optional(),
  messages: z305.z.array(AgentCompletionsMessageMessageSchema),
  responses: z305.z.array(AgentCompletionsMessageRichContentSchema)
}).meta({ title: "vector.completions.cache.CacheVoteRequestRef" });

// src/vector/completions/cache/cacheVoteRequest.ts
var VectorCompletionsCacheCacheVoteRequestSchema = z305.z.union([z305.z.object({
  Ref: VectorCompletionsCacheCacheVoteRequestRefSchema
}).strict(), z305.z.object({
  Owned: VectorCompletionsCacheCacheVoteRequestOwnedSchema
}).strict()]).meta({ title: "vector.completions.cache.CacheVoteRequest" });
var VectorCompletionsCacheCompletionVotesSchema = z305.z.object({
  data: z305.z.array(VectorCompletionsResponseVoteSchema).nullable().optional()
}).meta({ title: "vector.completions.cache.CompletionVotes" });

// src/vector/completions/cache/http.ts
function vectorCompletionsCacheGetCompletionVotes(client, id, options) {
  return client.get_unary(
    `/vector/completions/${id}`,
    void 0,
    options
  );
}
function vectorCompletionsCacheGetCacheVote(client, body, options) {
  return client.post_unary(
    "/vector/completions/cache",
    body,
    options
  );
}
var VectorCompletionsRequestVectorCompletionCreateParamsSchema = z305.z.object({
  retry: z305.z.string().nullable().describe("If present, reuses votes from a previous request with this ID.").optional(),
  from_cache: z305.z.boolean().nullable().describe("If true, uses cached votes when available.").optional(),
  messages: z305.z.array(AgentCompletionsMessageMessageSchema).describe("The conversation messages (the prompt)."),
  provider: AgentCompletionsRequestProviderSchema.nullable().describe("Provider routing preferences.").optional(),
  ensemble: VectorCompletionsRequestEnsembleSchema.describe("The Ensemble of agents to use."),
  profile: VectorCompletionsRequestProfileSchema.describe("The profile weights for each agent in the ensemble.\n\nMust have the same length as the total agent count in the ensemble.\nCan be either:\n- A vector of decimals (legacy representation), or\n- A vector of objects with `weight` and optional `invert` fields."),
  seed: z305.z.number().int().meta({ format: "int64" }).nullable().describe("Random seed for deterministic results.").optional(),
  stream: z305.z.boolean().nullable().describe("Whether to stream the response.").optional(),
  responses: z305.z.array(AgentCompletionsMessageRichContentSchema).describe("The possible responses the LLMs can vote for."),
  mcp_server_authorization: z305.z.record(z305.z.string(), z305.z.string()).nullable().describe("Map from MCP server URL to authorization header value.").optional()
}).describe("Parameters for creating a vector completion.\n\nVector completions run multiple agent completions (one per LLM in the\nensemble), force each to vote for one of the predefined responses, and\ncombine votes using the provided profile weights to produce final scores.").meta({ title: "vector.completions.request.VectorCompletionCreateParams" });
var VectorCompletionsResponseStreamingVectorCompletionChunkSchema = z305.z.object({
  id: z305.z.string().describe("Unique identifier for this vector completion."),
  completions: z305.z.array(VectorCompletionsResponseStreamingAgentCompletionChunkSchema).describe("Incremental agent completion chunks from each agent."),
  votes: z305.z.array(VectorCompletionsResponseVoteSchema).describe("Votes received so far. New votes are appended in subsequent chunks."),
  scores: z305.z.array(z305.z.number().meta({ format: "double" })).describe("Current weighted scores. Updated as new votes arrive."),
  weights: z305.z.array(z305.z.number().meta({ format: "double" })).describe("Current weight distribution across responses. Updated as new votes arrive."),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Unix timestamp when the completion was created."),
  ensemble: z305.z.string().describe("ID of the ensemble used for this completion."),
  object: VectorCompletionsResponseStreamingObjectSchema.describe('Object type identifier (`"vector.completion.chunk"`).'),
  usage: AgentCompletionsResponseUsageSchema.nullable().describe("Aggregated usage statistics. Typically present only in the final chunk.").optional()
}).describe("A chunk in a streaming vector completion response.\n\nEach chunk contains incremental updates to the completion. Use the\n[`push`](Self::push) method to accumulate chunks into a complete response.").meta({ title: "vector.completions.response.streaming.VectorCompletionChunk" });

// src/vector/completions/response/streaming/vectorCompletionChunkMerged.ts
function vectorCompletionsResponseStreamingVectorCompletionChunkMerged(a, b) {
  let changed = false;
  const [completions, c1] = vectorCompletionsResponseStreamingAgentCompletionChunkMergedList(a.completions, b.completions);
  if (c1) changed = true;
  const [votes, c2] = vectorCompletionsResponseVoteMergedList(a.votes, b.votes);
  if (c2) changed = true;
  const [scores, c3] = mergedNumberArray(a.scores, b.scores);
  if (c3) changed = true;
  const [weights, c4] = mergedNumberArray(a.weights, b.weights);
  if (c4) changed = true;
  let usage = a.usage;
  if (a.usage != null && b.usage != null) {
    const [merged, c] = agentCompletionsResponseUsageMerged(a.usage, b.usage);
    usage = merged;
    if (c) changed = true;
  } else if (b.usage != null) {
    usage = b.usage;
    changed = true;
  }
  if (!changed) return [a, false];
  return [{
    id: a.id,
    completions,
    votes,
    scores,
    weights,
    created: a.created,
    ensemble: a.ensemble,
    object: a.object,
    ...usage != null ? { usage } : {}
  }, true];
}
var VectorCompletionsResponseUnaryVectorCompletionSchema = z305.z.object({
  id: z305.z.string().describe("Unique identifier for this vector completion."),
  completions: z305.z.array(VectorCompletionsResponseUnaryAgentCompletionSchema).describe("The underlying agent completions from each agent in the ensemble."),
  votes: z305.z.array(VectorCompletionsResponseVoteSchema).describe("Individual votes from each agent, showing their selections."),
  scores: z305.z.array(z305.z.number().meta({ format: "double" })).describe("Final weighted scores for each response option. Sums to 1."),
  weights: z305.z.array(z305.z.number().meta({ format: "double" })).describe("Total weight allocated to each response option. Same length as `scores`.\nFor discrete votes, an LLM's full weight goes to its selected response.\nFor probabilistic votes, the weight is divided according to the distribution."),
  created: z305.z.number().int().min(0).meta({ format: "uint64" }).describe("Unix timestamp when the completion was created."),
  ensemble: z305.z.string().describe("ID of the ensemble used for this completion."),
  object: VectorCompletionsResponseUnaryObjectSchema.describe('Object type identifier (`"vector.completion"`).'),
  usage: AgentCompletionsResponseUsageSchema.describe("Aggregated token and cost usage across all completions.")
}).describe("A complete vector completion response (non-streaming).\n\nContains the final scores, all votes from the ensemble, and the underlying\nagent completions that produced those votes.").meta({ title: "vector.completions.response.unary.VectorCompletion" });
var VectorCompletionsVectorResponsesSchema = z305.z.array(AgentCompletionsMessageRichContentSchema).describe('The list of response options in a vector completion request.\n\nEach element is a [`RichContent`] value that an LLM can vote for.\nResponses can be plain text strings or multi-part content containing\ntext, images, audio, video, or files.\n\n# Minimum Length\n\nA vector completion requires at least 2 responses to vote between.\n\n# Examples\n\nPlain text responses:\n```json\n["Yes", "No", "Maybe"]\n```\n\nMultimodal responses:\n```json\n[\n  [{"type": "text", "text": "Option A"}, {"type": "image_url", "image_url": {"url": "https://example.com/a.png"}}],\n  [{"type": "text", "text": "Option B"}, {"type": "image_url", "image_url": {"url": "https://example.com/b.png"}}]\n]\n```').meta({ title: "vector.completions.VectorResponses" });
var VectorCompletionsRequestVectorCompletionCreateParamsStreamingSchema = VectorCompletionsRequestVectorCompletionCreateParamsSchema.extend({
  stream: z305__default.default.literal(true)
});
var VectorCompletionsRequestVectorCompletionCreateParamsUnarySchema = VectorCompletionsRequestVectorCompletionCreateParamsSchema.extend({
  stream: z305__default.default.literal(false).optional().nullable()
});
function vectorCompletionsCreateVectorCompletion(client, body, options) {
  if (body.stream) {
    return client.post_streaming(
      "/vector/completions",
      body,
      options
    );
  }
  return client.post_unary(
    "/vector/completions",
    body,
    options
  );
}

// src/error.ts
var ObjectiveAIFetchError = class extends Error {
  constructor(codeOrBody, rawBody) {
    let body;
    if (typeof codeOrBody !== "number") {
      body = codeOrBody;
    } else if (rawBody === null || rawBody === void 0) {
      body = { code: codeOrBody, message: null };
    } else {
      let parsed;
      try {
        parsed = JSON.parse(rawBody);
      } catch {
        body = { code: codeOrBody, message: rawBody };
        super(JSON.stringify(body));
        this.name = "ObjectiveAIFetchError";
        this.body = body;
        return;
      }
      if (isResponseError(parsed)) {
        body = parsed;
      } else {
        body = { code: codeOrBody, message: parsed };
      }
    }
    super(JSON.stringify(body));
    this.name = "ObjectiveAIFetchError";
    this.body = body;
  }
  /**
   * Convenience getter for the error code.
   */
  get code() {
    return this.body.code;
  }
  /**
   * Serialize to ResponseError JSON format.
   */
  toJSON() {
    return this.body;
  }
};
function isResponseError(obj) {
  return typeof obj === "object" && obj !== null && "code" in obj && typeof obj.code === "number" && "message" in obj;
}

// src/stream.ts
var Stream = class {
  constructor(response, controller) {
    this.buffer = "";
    this.done = false;
    if (!response.body) {
      throw new Error("Response body is null");
    }
    this.reader = response.body.getReader();
    this.decoder = new TextDecoder();
    this.controller = controller ?? null;
  }
  /**
   * Abort the stream.
   */
  abort() {
    this.controller?.abort();
  }
  async *[Symbol.asyncIterator]() {
    try {
      while (!this.done) {
        const { value, done } = await this.reader.read();
        if (done) {
          this.done = true;
          if (this.buffer.trim()) {
            const events = this.parseSSE(this.buffer);
            for (const event of events) {
              if (event !== null) {
                yield event;
              }
            }
          }
          break;
        }
        this.buffer += this.decoder.decode(value, { stream: true });
        const parts = this.buffer.split(/\n\n/);
        this.buffer = parts.pop() ?? "";
        for (const part of parts) {
          const events = this.parseSSE(part);
          for (const event of events) {
            if (event !== null) {
              yield event;
            }
          }
        }
      }
    } finally {
      this.reader.releaseLock();
    }
  }
  /**
   * Parse SSE format and extract data.
   * Returns null for [DONE] or empty events.
   *
   * SSE format:
   * - Lines starting with `:` are comments and are ignored
   * - `data:` lines contain the event payload
   * - Empty lines separate events
   * - We only process `data:` fields, ignoring `event:`, `id:`, `retry:`
   */
  parseSSE(text) {
    const results = [];
    const lines = text.split("\n");
    let hasDataLine = false;
    for (const line of lines) {
      if (!line) {
        continue;
      }
      if (line.startsWith(":")) {
        continue;
      }
      if (line.startsWith("data:")) {
        hasDataLine = true;
        const data = line.slice(5).trim();
        if (data === "[DONE]") {
          this.done = true;
          continue;
        }
        if (!data) {
          continue;
        }
        const parsed = JSON.parse(data);
        if (isResponseError(parsed)) {
          throw new ObjectiveAIFetchError(parsed);
        }
        results.push(parsed);
      }
    }
    if (!hasDataLine && results.length === 0) {
      return [];
    }
    return results;
  }
  /**
   * Collect all events into an array.
   */
  async toArray() {
    const results = [];
    for await (const item of this) {
      results.push(item);
    }
    return results;
  }
};

// src/client.ts
function readEnv(env) {
  if (typeof globalThis.process !== "undefined") {
    return globalThis.process.env?.[env]?.trim() ?? void 0;
  }
  if (typeof globalThis.Deno !== "undefined") {
    return globalThis.Deno.env?.get?.(env)?.trim();
  }
  return void 0;
}
var ObjectiveAIOptionsSchema = z305__default.default.object({
  apiKey: z305__default.default.string().nullish().describe("API key for authentication. Falls back to OBJECTIVEAI_API_KEY env var."),
  apiBase: z305__default.default.string().nullish().describe(
    "Base URL for the API. Falls back to OBJECTIVEAI_API_BASE env var, then https://api.objective-ai.io"
  ),
  userAgent: z305__default.default.string().nullish().describe("User-Agent header. Falls back to USER_AGENT env var."),
  xTitle: z305__default.default.string().nullish().describe("X-Title header. Falls back to X_TITLE env var."),
  httpReferer: z305__default.default.string().nullish().describe("HTTP-Referer header. Falls back to HTTP_REFERER env var."),
  xGithubAuthorization: z305__default.default.string().nullish().describe("X-GITHUB-AUTHORIZATION header for GitHub-hosted function/profile access."),
  xOpenrouterAuthorization: z305__default.default.string().nullish().describe("X-OPENROUTER-AUTHORIZATION header for BYOK (Bring Your Own Key) support."),
  xMcpAuthorization: z305__default.default.record(z305__default.default.string(), z305__default.default.string()).nullish().describe("X-MCP-AUTHORIZATION header (JSON-encoded map of MCP authorization headers).")
}).describe("Options for the ObjectiveAI client.");
var RequestOptionsSchema = z305__default.default.object({
  headers: z305__default.default.union([
    z305__default.default.instanceof(Headers),
    z305__default.default.record(z305__default.default.string(), z305__default.default.string()),
    z305__default.default.array(z305__default.default.tuple([z305__default.default.string(), z305__default.default.string()]))
  ]).nullish().describe("Additional headers to include in the request."),
  signal: z305__default.default.instanceof(AbortSignal).nullish().describe("AbortSignal for cancelling the request.")
}).describe("Options for individual requests.");
var ObjectiveAI = class {
  constructor(options) {
    this.apiKey = options?.apiKey ?? readEnv("OBJECTIVEAI_API_KEY") ?? void 0;
    this.apiBase = options?.apiBase ?? readEnv("OBJECTIVEAI_API_BASE") ?? "https://api.objective-ai.io";
    this.userAgent = options?.userAgent ?? readEnv("USER_AGENT") ?? void 0;
    this.xTitle = options?.xTitle ?? readEnv("X_TITLE") ?? void 0;
    this.httpReferer = options?.httpReferer ?? readEnv("HTTP_REFERER") ?? void 0;
    this.xGithubAuthorization = options?.xGithubAuthorization ?? void 0;
    this.xOpenrouterAuthorization = options?.xOpenrouterAuthorization ?? void 0;
    this.xMcpAuthorization = options?.xMcpAuthorization ?? void 0;
  }
  /**
   * Build headers for a request.
   */
  buildHeaders(options) {
    const headers = new Headers();
    headers.set("Content-Type", "application/json");
    if (this.apiKey) {
      headers.set("Authorization", `Bearer ${this.apiKey}`);
    }
    if (this.userAgent) {
      headers.set("User-Agent", this.userAgent);
    }
    if (this.xTitle) {
      headers.set("X-Title", this.xTitle);
    }
    if (this.httpReferer) {
      headers.set("HTTP-Referer", this.httpReferer);
    }
    if (this.xGithubAuthorization) {
      headers.set("X-GITHUB-AUTHORIZATION", this.xGithubAuthorization);
    }
    if (this.xOpenrouterAuthorization) {
      headers.set("X-OPENROUTER-AUTHORIZATION", this.xOpenrouterAuthorization);
    }
    if (this.xMcpAuthorization) {
      headers.set("X-MCP-AUTHORIZATION", JSON.stringify(this.xMcpAuthorization));
    }
    if (options?.headers) {
      const optHeaders = options.headers;
      if (optHeaders instanceof Headers) {
        optHeaders.forEach((value, key) => headers.set(key, value));
      } else if (Array.isArray(optHeaders)) {
        for (const [key, value] of optHeaders) {
          headers.set(key, value);
        }
      } else {
        for (const [key, value] of Object.entries(optHeaders)) {
          headers.set(key, value);
        }
      }
    }
    return headers;
  }
  /**
   * Build the full URL for a path.
   */
  buildUrl(path) {
    const base = this.apiBase.endsWith("/") ? this.apiBase.slice(0, -1) : this.apiBase;
    const normalizedPath = path.startsWith("/") ? path : `/${path}`;
    return `${base}${normalizedPath}`;
  }
  /**
   * Handle error responses, extracting the body.
   */
  async handleErrorResponse(response) {
    let rawBody;
    try {
      rawBody = await response.text();
    } catch {
      rawBody = null;
    }
    return new ObjectiveAIFetchError(response.status, rawBody);
  }
  /**
   * Perform a GET request and return the parsed JSON response.
   */
  async get_unary(path, body, options) {
    const response = await fetch(this.buildUrl(path), {
      method: "GET",
      headers: this.buildHeaders(options),
      body: body !== void 0 ? JSON.stringify(body) : void 0,
      signal: options?.signal ?? void 0
    });
    if (!response.ok) {
      throw await this.handleErrorResponse(response);
    }
    return await response.json();
  }
  /**
   * Perform a POST request and return the parsed JSON response.
   */
  async post_unary(path, body, options) {
    const response = await fetch(this.buildUrl(path), {
      method: "POST",
      headers: this.buildHeaders(options),
      body: body !== void 0 ? JSON.stringify(body) : void 0,
      signal: options?.signal ?? void 0
    });
    if (!response.ok) {
      throw await this.handleErrorResponse(response);
    }
    return await response.json();
  }
  /**
   * Perform a DELETE request and return the parsed JSON response.
   */
  async delete_unary(path, body, options) {
    const response = await fetch(this.buildUrl(path), {
      method: "DELETE",
      headers: this.buildHeaders(options),
      body: body !== void 0 ? JSON.stringify(body) : void 0,
      signal: options?.signal ?? void 0
    });
    if (!response.ok) {
      throw await this.handleErrorResponse(response);
    }
    return await response.json();
  }
  /**
   * Perform a GET request and return an SSE stream.
   */
  async get_streaming(path, body, options) {
    const headers = this.buildHeaders(options);
    headers.set("Accept", "text/event-stream");
    const controller = new AbortController();
    const signal = options?.signal;
    if (signal) {
      signal.addEventListener("abort", () => controller.abort());
    }
    const response = await fetch(this.buildUrl(path), {
      method: "GET",
      headers,
      body: body !== void 0 ? JSON.stringify(body) : void 0,
      signal: controller.signal
    });
    if (!response.ok) {
      throw await this.handleErrorResponse(response);
    }
    return new Stream(response, controller);
  }
  /**
   * Perform a POST request and return an SSE stream.
   */
  async post_streaming(path, body, options) {
    const headers = this.buildHeaders(options);
    headers.set("Accept", "text/event-stream");
    const controller = new AbortController();
    const signal = options?.signal;
    if (signal) {
      signal.addEventListener("abort", () => controller.abort());
    }
    const response = await fetch(this.buildUrl(path), {
      method: "POST",
      headers,
      body: body !== void 0 ? JSON.stringify(body) : void 0,
      signal: controller.signal
    });
    if (!response.ok) {
      throw await this.handleErrorResponse(response);
    }
    return new Stream(response, controller);
  }
  /**
   * Perform a DELETE request and return an SSE stream.
   */
  async delete_streaming(path, body, options) {
    const headers = this.buildHeaders(options);
    headers.set("Accept", "text/event-stream");
    const controller = new AbortController();
    const signal = options?.signal;
    if (signal) {
      signal.addEventListener("abort", () => controller.abort());
    }
    const response = await fetch(this.buildUrl(path), {
      method: "DELETE",
      headers,
      body: body !== void 0 ? JSON.stringify(body) : void 0,
      signal: controller.signal
    });
    if (!response.ok) {
      throw await this.handleErrorResponse(response);
    }
    return new Stream(response, controller);
  }
};

// src/isEmpty.ts
function numberIsEmpty(value) {
  return value === null || value === void 0 || value === 0;
}

// src/zockerParse.ts
var NUMBER_MIN = 0;
var NUMBER_MAX = 999;
function fixForSerde(value) {
  if (typeof value === "number") {
    if (!Number.isFinite(value) || value < NUMBER_MIN || value > NUMBER_MAX) {
      return Math.floor(Math.random() * (NUMBER_MAX - NUMBER_MIN + 1)) + NUMBER_MIN;
    }
    return value;
  } else if (value !== null && typeof value === "object") {
    const obj = value;
    for (const k in obj) {
      obj[k] = fixForSerde(obj[k]);
    }
    return value;
  } else {
    return value;
  }
}
function zockerParse(gen, normalize) {
  let raw;
  for (let attempt = 0; ; attempt++) {
    try {
      raw = gen.generate();
      break;
    } catch (e) {
      if (attempt >= 99) throw e;
    }
  }
  const fixed = fixForSerde(raw);
  return JSON.parse(normalize(fixed));
}

exports.AgentAgentBaseSchema = AgentAgentBaseSchema;
exports.AgentAgentSchema = AgentAgentSchema;
exports.AgentClaudeAgentSdkAgentBaseSchema = AgentClaudeAgentSdkAgentBaseSchema;
exports.AgentClaudeAgentSdkAgentSchema = AgentClaudeAgentSdkAgentSchema;
exports.AgentClaudeAgentSdkEffortSchema = AgentClaudeAgentSdkEffortSchema;
exports.AgentClaudeAgentSdkOutputModeSchema = AgentClaudeAgentSdkOutputModeSchema;
exports.AgentClaudeAgentSdkUpstreamSchema = AgentClaudeAgentSdkUpstreamSchema;
exports.AgentCompletionsMessageAssistantMessageExpressionSchema = AgentCompletionsMessageAssistantMessageExpressionSchema;
exports.AgentCompletionsMessageAssistantMessageSchema = AgentCompletionsMessageAssistantMessageSchema;
exports.AgentCompletionsMessageAssistantToolCallDeltaSchema = AgentCompletionsMessageAssistantToolCallDeltaSchema;
exports.AgentCompletionsMessageAssistantToolCallExpressionSchema = AgentCompletionsMessageAssistantToolCallExpressionSchema;
exports.AgentCompletionsMessageAssistantToolCallFunctionDeltaSchema = AgentCompletionsMessageAssistantToolCallFunctionDeltaSchema;
exports.AgentCompletionsMessageAssistantToolCallFunctionExpressionSchema = AgentCompletionsMessageAssistantToolCallFunctionExpressionSchema;
exports.AgentCompletionsMessageAssistantToolCallFunctionSchema = AgentCompletionsMessageAssistantToolCallFunctionSchema;
exports.AgentCompletionsMessageAssistantToolCallSchema = AgentCompletionsMessageAssistantToolCallSchema;
exports.AgentCompletionsMessageAssistantToolCallTypeSchema = AgentCompletionsMessageAssistantToolCallTypeSchema;
exports.AgentCompletionsMessageDeveloperMessageExpressionSchema = AgentCompletionsMessageDeveloperMessageExpressionSchema;
exports.AgentCompletionsMessageDeveloperMessageSchema = AgentCompletionsMessageDeveloperMessageSchema;
exports.AgentCompletionsMessageFileSchema = AgentCompletionsMessageFileSchema;
exports.AgentCompletionsMessageImageUrlDetailSchema = AgentCompletionsMessageImageUrlDetailSchema;
exports.AgentCompletionsMessageImageUrlSchema = AgentCompletionsMessageImageUrlSchema;
exports.AgentCompletionsMessageInputAudioSchema = AgentCompletionsMessageInputAudioSchema;
exports.AgentCompletionsMessageMessageExpressionSchema = AgentCompletionsMessageMessageExpressionSchema;
exports.AgentCompletionsMessageMessageSchema = AgentCompletionsMessageMessageSchema;
exports.AgentCompletionsMessageRichContentExpressionSchema = AgentCompletionsMessageRichContentExpressionSchema;
exports.AgentCompletionsMessageRichContentPartExpressionSchema = AgentCompletionsMessageRichContentPartExpressionSchema;
exports.AgentCompletionsMessageRichContentPartSchema = AgentCompletionsMessageRichContentPartSchema;
exports.AgentCompletionsMessageRichContentSchema = AgentCompletionsMessageRichContentSchema;
exports.AgentCompletionsMessageSimpleContentExpressionSchema = AgentCompletionsMessageSimpleContentExpressionSchema;
exports.AgentCompletionsMessageSimpleContentPartExpressionSchema = AgentCompletionsMessageSimpleContentPartExpressionSchema;
exports.AgentCompletionsMessageSimpleContentPartSchema = AgentCompletionsMessageSimpleContentPartSchema;
exports.AgentCompletionsMessageSimpleContentSchema = AgentCompletionsMessageSimpleContentSchema;
exports.AgentCompletionsMessageSystemMessageExpressionSchema = AgentCompletionsMessageSystemMessageExpressionSchema;
exports.AgentCompletionsMessageSystemMessageSchema = AgentCompletionsMessageSystemMessageSchema;
exports.AgentCompletionsMessageToolMessageExpressionSchema = AgentCompletionsMessageToolMessageExpressionSchema;
exports.AgentCompletionsMessageToolMessageSchema = AgentCompletionsMessageToolMessageSchema;
exports.AgentCompletionsMessageUserMessageExpressionSchema = AgentCompletionsMessageUserMessageExpressionSchema;
exports.AgentCompletionsMessageUserMessageSchema = AgentCompletionsMessageUserMessageSchema;
exports.AgentCompletionsMessageVideoUrlSchema = AgentCompletionsMessageVideoUrlSchema;
exports.AgentCompletionsRequestAgentCompletionCreateParamsSchema = AgentCompletionsRequestAgentCompletionCreateParamsSchema;
exports.AgentCompletionsRequestAgentCompletionCreateParamsStreamingSchema = AgentCompletionsRequestAgentCompletionCreateParamsStreamingSchema;
exports.AgentCompletionsRequestAgentCompletionCreateParamsUnarySchema = AgentCompletionsRequestAgentCompletionCreateParamsUnarySchema;
exports.AgentCompletionsRequestAgentSchema = AgentCompletionsRequestAgentSchema;
exports.AgentCompletionsRequestProviderDataCollectionSchema = AgentCompletionsRequestProviderDataCollectionSchema;
exports.AgentCompletionsRequestProviderMaxPriceSchema = AgentCompletionsRequestProviderMaxPriceSchema;
exports.AgentCompletionsRequestProviderSchema = AgentCompletionsRequestProviderSchema;
exports.AgentCompletionsRequestProviderSortSchema = AgentCompletionsRequestProviderSortSchema;
exports.AgentCompletionsRequestResponseFormatParamSchema = AgentCompletionsRequestResponseFormatParamSchema;
exports.AgentCompletionsRequestResponseFormatSchema = AgentCompletionsRequestResponseFormatSchema;
exports.AgentCompletionsResponseAssistantRoleSchema = AgentCompletionsResponseAssistantRoleSchema;
exports.AgentCompletionsResponseCompletionTokensDetailsSchema = AgentCompletionsResponseCompletionTokensDetailsSchema;
exports.AgentCompletionsResponseCostDetailsSchema = AgentCompletionsResponseCostDetailsSchema;
exports.AgentCompletionsResponseFinishReasonSchema = AgentCompletionsResponseFinishReasonSchema;
exports.AgentCompletionsResponseLogprobSchema = AgentCompletionsResponseLogprobSchema;
exports.AgentCompletionsResponseLogprobsSchema = AgentCompletionsResponseLogprobsSchema;
exports.AgentCompletionsResponsePromptTokensDetailsSchema = AgentCompletionsResponsePromptTokensDetailsSchema;
exports.AgentCompletionsResponseStreamingAgentCompletionChunkSchema = AgentCompletionsResponseStreamingAgentCompletionChunkSchema;
exports.AgentCompletionsResponseStreamingAssistantResponseChunkSchema = AgentCompletionsResponseStreamingAssistantResponseChunkSchema;
exports.AgentCompletionsResponseStreamingMessageChunkSchema = AgentCompletionsResponseStreamingMessageChunkSchema;
exports.AgentCompletionsResponseStreamingObjectSchema = AgentCompletionsResponseStreamingObjectSchema;
exports.AgentCompletionsResponseToolResponseSchema = AgentCompletionsResponseToolResponseSchema;
exports.AgentCompletionsResponseToolRoleSchema = AgentCompletionsResponseToolRoleSchema;
exports.AgentCompletionsResponseTopLogprobSchema = AgentCompletionsResponseTopLogprobSchema;
exports.AgentCompletionsResponseUnaryAgentCompletionSchema = AgentCompletionsResponseUnaryAgentCompletionSchema;
exports.AgentCompletionsResponseUnaryAssistantResponseSchema = AgentCompletionsResponseUnaryAssistantResponseSchema;
exports.AgentCompletionsResponseUnaryMessageSchema = AgentCompletionsResponseUnaryMessageSchema;
exports.AgentCompletionsResponseUnaryObjectSchema = AgentCompletionsResponseUnaryObjectSchema;
exports.AgentCompletionsResponseUpstreamUsageSchema = AgentCompletionsResponseUpstreamUsageSchema;
exports.AgentCompletionsResponseUsageSchema = AgentCompletionsResponseUsageSchema;
exports.AgentGetAgentSchema = AgentGetAgentSchema;
exports.AgentListAgentItemSchema = AgentListAgentItemSchema;
exports.AgentListAgentSchema = AgentListAgentSchema;
exports.AgentMcpServerSchema = AgentMcpServerSchema;
exports.AgentMockAgentBaseSchema = AgentMockAgentBaseSchema;
exports.AgentMockAgentSchema = AgentMockAgentSchema;
exports.AgentMockOutputModeSchema = AgentMockOutputModeSchema;
exports.AgentMockUpstreamSchema = AgentMockUpstreamSchema;
exports.AgentOpenrouterAgentBaseSchema = AgentOpenrouterAgentBaseSchema;
exports.AgentOpenrouterAgentSchema = AgentOpenrouterAgentSchema;
exports.AgentOpenrouterOutputModeSchema = AgentOpenrouterOutputModeSchema;
exports.AgentOpenrouterProviderQuantizationSchema = AgentOpenrouterProviderQuantizationSchema;
exports.AgentOpenrouterProviderSchema = AgentOpenrouterProviderSchema;
exports.AgentOpenrouterReasoningEffortSchema = AgentOpenrouterReasoningEffortSchema;
exports.AgentOpenrouterReasoningSchema = AgentOpenrouterReasoningSchema;
exports.AgentOpenrouterReasoningSummaryVerbositySchema = AgentOpenrouterReasoningSummaryVerbositySchema;
exports.AgentOpenrouterStopSchema = AgentOpenrouterStopSchema;
exports.AgentOpenrouterUpstreamSchema = AgentOpenrouterUpstreamSchema;
exports.AgentOpenrouterVerbositySchema = AgentOpenrouterVerbositySchema;
exports.AgentOutputModeSchema = AgentOutputModeSchema;
exports.AgentUpstreamSchema = AgentUpstreamSchema;
exports.AgentUsageAgentSchema = AgentUsageAgentSchema;
exports.AgentWithFallbacksAndCountAgentAgentBaseSchema = AgentWithFallbacksAndCountAgentAgentBaseSchema;
exports.AgentWithFallbacksAndCountAgentAgentSchema = AgentWithFallbacksAndCountAgentAgentSchema;
exports.AuthApiKeyWithMetadataSchema = AuthApiKeyWithMetadataSchema;
exports.AuthCreateApiKeyRequestSchema = AuthCreateApiKeyRequestSchema;
exports.AuthCreateOpenRouterByokApiKeyRequestSchema = AuthCreateOpenRouterByokApiKeyRequestSchema;
exports.AuthDisableApiKeyRequestSchema = AuthDisableApiKeyRequestSchema;
exports.AuthGetCreditsResponseSchema = AuthGetCreditsResponseSchema;
exports.AuthGetOpenRouterByokApiKeyResponseSchema = AuthGetOpenRouterByokApiKeyResponseSchema;
exports.AuthListApiKeyItemSchema = AuthListApiKeyItemSchema;
exports.AuthListApiKeyResponseSchema = AuthListApiKeyResponseSchema;
exports.EnsembleEnsembleBaseSchema = EnsembleEnsembleBaseSchema;
exports.EnsembleEnsembleSchema = EnsembleEnsembleSchema;
exports.EnsembleGetEnsembleSchema = EnsembleGetEnsembleSchema;
exports.EnsembleListEnsembleItemSchema = EnsembleListEnsembleItemSchema;
exports.EnsembleListEnsembleSchema = EnsembleListEnsembleSchema;
exports.EnsembleUsageEnsembleSchema = EnsembleUsageEnsembleSchema;
exports.FunctionsAlphaInlineFunctionSchema = FunctionsAlphaInlineFunctionSchema;
exports.FunctionsAlphaRemoteFunctionSchema = FunctionsAlphaRemoteFunctionSchema;
exports.FunctionsAlphaScalarBranchTaskExpressionSchema = FunctionsAlphaScalarBranchTaskExpressionSchema;
exports.FunctionsAlphaScalarInlineFunctionSchema = FunctionsAlphaScalarInlineFunctionSchema;
exports.FunctionsAlphaScalarLeafTaskExpressionSchema = FunctionsAlphaScalarLeafTaskExpressionSchema;
exports.FunctionsAlphaScalarPartialPlaceholderBranchTaskExpressionSchema = FunctionsAlphaScalarPartialPlaceholderBranchTaskExpressionSchema;
exports.FunctionsAlphaScalarPartialPlaceholderScalarFunctionTaskExpressionSchema = FunctionsAlphaScalarPartialPlaceholderScalarFunctionTaskExpressionSchema;
exports.FunctionsAlphaScalarPlaceholderScalarFunctionTaskExpressionSchema = FunctionsAlphaScalarPlaceholderScalarFunctionTaskExpressionSchema;
exports.FunctionsAlphaScalarRemoteFunctionSchema = FunctionsAlphaScalarRemoteFunctionSchema;
exports.FunctionsAlphaScalarScalarFunctionTaskExpressionSchema = FunctionsAlphaScalarScalarFunctionTaskExpressionSchema;
exports.FunctionsAlphaScalarVectorCompletionTaskExpressionSchema = FunctionsAlphaScalarVectorCompletionTaskExpressionSchema;
exports.FunctionsAlphaVectorBranchTaskExpressionSchema = FunctionsAlphaVectorBranchTaskExpressionSchema;
exports.FunctionsAlphaVectorExpressionVectorFunctionInputSchemaSchema = FunctionsAlphaVectorExpressionVectorFunctionInputSchemaSchema;
exports.FunctionsAlphaVectorExpressionVectorFunctionInputValueExpressionSchema = FunctionsAlphaVectorExpressionVectorFunctionInputValueExpressionSchema;
exports.FunctionsAlphaVectorExpressionVectorFunctionInputValueSchema = FunctionsAlphaVectorExpressionVectorFunctionInputValueSchema;
exports.FunctionsAlphaVectorInlineFunctionSchema = FunctionsAlphaVectorInlineFunctionSchema;
exports.FunctionsAlphaVectorLeafTaskExpressionSchema = FunctionsAlphaVectorLeafTaskExpressionSchema;
exports.FunctionsAlphaVectorPartialPlaceholderBranchTaskExpressionSchema = FunctionsAlphaVectorPartialPlaceholderBranchTaskExpressionSchema;
exports.FunctionsAlphaVectorPartialPlaceholderScalarFunctionTaskExpressionSchema = FunctionsAlphaVectorPartialPlaceholderScalarFunctionTaskExpressionSchema;
exports.FunctionsAlphaVectorPartialPlaceholderVectorFunctionTaskExpressionSchema = FunctionsAlphaVectorPartialPlaceholderVectorFunctionTaskExpressionSchema;
exports.FunctionsAlphaVectorPlaceholderScalarFunctionTaskExpressionSchema = FunctionsAlphaVectorPlaceholderScalarFunctionTaskExpressionSchema;
exports.FunctionsAlphaVectorPlaceholderVectorFunctionTaskExpressionSchema = FunctionsAlphaVectorPlaceholderVectorFunctionTaskExpressionSchema;
exports.FunctionsAlphaVectorRemoteFunctionSchema = FunctionsAlphaVectorRemoteFunctionSchema;
exports.FunctionsAlphaVectorScalarFunctionTaskExpressionSchema = FunctionsAlphaVectorScalarFunctionTaskExpressionSchema;
exports.FunctionsAlphaVectorVectorCompletionTaskExpressionSchema = FunctionsAlphaVectorVectorCompletionTaskExpressionSchema;
exports.FunctionsAlphaVectorVectorFunctionTaskExpressionSchema = FunctionsAlphaVectorVectorFunctionTaskExpressionSchema;
exports.FunctionsCheckScalarFieldsValidationSchema = FunctionsCheckScalarFieldsValidationSchema;
exports.FunctionsCheckVectorFieldsValidationSchema = FunctionsCheckVectorFieldsValidationSchema;
exports.FunctionsCompiledTaskSchema = FunctionsCompiledTaskSchema;
exports.FunctionsExecutionsRequestFunctionExecutionCreateParamsSchema = FunctionsExecutionsRequestFunctionExecutionCreateParamsSchema;
exports.FunctionsExecutionsRequestFunctionInlineProfileInlineRequestBodySchema = FunctionsExecutionsRequestFunctionInlineProfileInlineRequestBodySchema;
exports.FunctionsExecutionsRequestFunctionInlineProfileRemoteRequestBodySchema = FunctionsExecutionsRequestFunctionInlineProfileRemoteRequestBodySchema;
exports.FunctionsExecutionsRequestFunctionInlineProfileRemoteRequestPathSchema = FunctionsExecutionsRequestFunctionInlineProfileRemoteRequestPathSchema;
exports.FunctionsExecutionsRequestFunctionRemoteProfileInlineRequestBodySchema = FunctionsExecutionsRequestFunctionRemoteProfileInlineRequestBodySchema;
exports.FunctionsExecutionsRequestFunctionRemoteProfileInlineRequestPathSchema = FunctionsExecutionsRequestFunctionRemoteProfileInlineRequestPathSchema;
exports.FunctionsExecutionsRequestFunctionRemoteProfileRemoteRequestBodySchema = FunctionsExecutionsRequestFunctionRemoteProfileRemoteRequestBodySchema;
exports.FunctionsExecutionsRequestFunctionRemoteProfileRemoteRequestPathSchema = FunctionsExecutionsRequestFunctionRemoteProfileRemoteRequestPathSchema;
exports.FunctionsExecutionsRequestReasoningSchema = FunctionsExecutionsRequestReasoningSchema;
exports.FunctionsExecutionsRequestRequestSchema = FunctionsExecutionsRequestRequestSchema;
exports.FunctionsExecutionsRequestStrategySchema = FunctionsExecutionsRequestStrategySchema;
exports.FunctionsExecutionsResponseStreamingFunctionExecutionChunkSchema = FunctionsExecutionsResponseStreamingFunctionExecutionChunkSchema;
exports.FunctionsExecutionsResponseStreamingFunctionExecutionTaskChunkSchema = FunctionsExecutionsResponseStreamingFunctionExecutionTaskChunkSchema;
exports.FunctionsExecutionsResponseStreamingObjectSchema = FunctionsExecutionsResponseStreamingObjectSchema;
exports.FunctionsExecutionsResponseStreamingReasoningSummaryChunkSchema = FunctionsExecutionsResponseStreamingReasoningSummaryChunkSchema;
exports.FunctionsExecutionsResponseStreamingTaskChunkSchema = FunctionsExecutionsResponseStreamingTaskChunkSchema;
exports.FunctionsExecutionsResponseStreamingVectorCompletionTaskChunkSchema = FunctionsExecutionsResponseStreamingVectorCompletionTaskChunkSchema;
exports.FunctionsExecutionsResponseUnaryFunctionExecutionSchema = FunctionsExecutionsResponseUnaryFunctionExecutionSchema;
exports.FunctionsExecutionsResponseUnaryFunctionExecutionTaskSchema = FunctionsExecutionsResponseUnaryFunctionExecutionTaskSchema;
exports.FunctionsExecutionsResponseUnaryObjectSchema = FunctionsExecutionsResponseUnaryObjectSchema;
exports.FunctionsExecutionsResponseUnaryReasoningSummarySchema = FunctionsExecutionsResponseUnaryReasoningSummarySchema;
exports.FunctionsExecutionsResponseUnaryTaskSchema = FunctionsExecutionsResponseUnaryTaskSchema;
exports.FunctionsExecutionsResponseUnaryVectorCompletionTaskSchema = FunctionsExecutionsResponseUnaryVectorCompletionTaskSchema;
exports.FunctionsExecutionsRetryTokenSchema = FunctionsExecutionsRetryTokenSchema;
exports.FunctionsExpressionAnyOfInputSchemaSchema = FunctionsExpressionAnyOfInputSchemaSchema;
exports.FunctionsExpressionArrayInputSchemaSchema = FunctionsExpressionArrayInputSchemaSchema;
exports.FunctionsExpressionArrayInputSchemaTypeSchema = FunctionsExpressionArrayInputSchemaTypeSchema;
exports.FunctionsExpressionAudioInputSchemaSchema = FunctionsExpressionAudioInputSchemaSchema;
exports.FunctionsExpressionAudioInputSchemaTypeSchema = FunctionsExpressionAudioInputSchemaTypeSchema;
exports.FunctionsExpressionBooleanInputSchemaSchema = FunctionsExpressionBooleanInputSchemaSchema;
exports.FunctionsExpressionBooleanInputSchemaTypeSchema = FunctionsExpressionBooleanInputSchemaTypeSchema;
exports.FunctionsExpressionExpressionSchema = FunctionsExpressionExpressionSchema;
exports.FunctionsExpressionFileInputSchemaSchema = FunctionsExpressionFileInputSchemaSchema;
exports.FunctionsExpressionFileInputSchemaTypeSchema = FunctionsExpressionFileInputSchemaTypeSchema;
exports.FunctionsExpressionImageInputSchemaSchema = FunctionsExpressionImageInputSchemaSchema;
exports.FunctionsExpressionImageInputSchemaTypeSchema = FunctionsExpressionImageInputSchemaTypeSchema;
exports.FunctionsExpressionInputSchemaSchema = FunctionsExpressionInputSchemaSchema;
exports.FunctionsExpressionInputValueExpressionSchema = FunctionsExpressionInputValueExpressionSchema;
exports.FunctionsExpressionInputValueSchema = FunctionsExpressionInputValueSchema;
exports.FunctionsExpressionIntegerInputSchemaSchema = FunctionsExpressionIntegerInputSchemaSchema;
exports.FunctionsExpressionIntegerInputSchemaTypeSchema = FunctionsExpressionIntegerInputSchemaTypeSchema;
exports.FunctionsExpressionNumberInputSchemaSchema = FunctionsExpressionNumberInputSchemaSchema;
exports.FunctionsExpressionNumberInputSchemaTypeSchema = FunctionsExpressionNumberInputSchemaTypeSchema;
exports.FunctionsExpressionObjectInputSchemaSchema = FunctionsExpressionObjectInputSchemaSchema;
exports.FunctionsExpressionObjectInputSchemaTypeSchema = FunctionsExpressionObjectInputSchemaTypeSchema;
exports.FunctionsExpressionOneOrManyStringSchema = FunctionsExpressionOneOrManyStringSchema;
exports.FunctionsExpressionParamsOwnedSchema = FunctionsExpressionParamsOwnedSchema;
exports.FunctionsExpressionParamsRefSchema = FunctionsExpressionParamsRefSchema;
exports.FunctionsExpressionParamsSchema = FunctionsExpressionParamsSchema;
exports.FunctionsExpressionSpecialSchema = FunctionsExpressionSpecialSchema;
exports.FunctionsExpressionStringInputSchemaSchema = FunctionsExpressionStringInputSchemaSchema;
exports.FunctionsExpressionStringInputSchemaTypeSchema = FunctionsExpressionStringInputSchemaTypeSchema;
exports.FunctionsExpressionTaskOutputOwnedSchema = FunctionsExpressionTaskOutputOwnedSchema;
exports.FunctionsExpressionTaskOutputRefSchema = FunctionsExpressionTaskOutputRefSchema;
exports.FunctionsExpressionTaskOutputSchema = FunctionsExpressionTaskOutputSchema;
exports.FunctionsExpressionVideoInputSchemaSchema = FunctionsExpressionVideoInputSchemaSchema;
exports.FunctionsExpressionVideoInputSchemaTypeSchema = FunctionsExpressionVideoInputSchemaTypeSchema;
exports.FunctionsExpressionWithExpressionAgentCompletionsMessageAssistantToolCallExpressionSchema = FunctionsExpressionWithExpressionAgentCompletionsMessageAssistantToolCallExpressionSchema;
exports.FunctionsExpressionWithExpressionAgentCompletionsMessageAssistantToolCallFunctionExpressionSchema = FunctionsExpressionWithExpressionAgentCompletionsMessageAssistantToolCallFunctionExpressionSchema;
exports.FunctionsExpressionWithExpressionAgentCompletionsMessageFileSchema = FunctionsExpressionWithExpressionAgentCompletionsMessageFileSchema;
exports.FunctionsExpressionWithExpressionAgentCompletionsMessageImageUrlSchema = FunctionsExpressionWithExpressionAgentCompletionsMessageImageUrlSchema;
exports.FunctionsExpressionWithExpressionAgentCompletionsMessageInputAudioSchema = FunctionsExpressionWithExpressionAgentCompletionsMessageInputAudioSchema;
exports.FunctionsExpressionWithExpressionAgentCompletionsMessageMessageExpressionSchema = FunctionsExpressionWithExpressionAgentCompletionsMessageMessageExpressionSchema;
exports.FunctionsExpressionWithExpressionAgentCompletionsMessageRichContentExpressionSchema = FunctionsExpressionWithExpressionAgentCompletionsMessageRichContentExpressionSchema;
exports.FunctionsExpressionWithExpressionAgentCompletionsMessageRichContentPartExpressionSchema = FunctionsExpressionWithExpressionAgentCompletionsMessageRichContentPartExpressionSchema;
exports.FunctionsExpressionWithExpressionAgentCompletionsMessageSimpleContentExpressionSchema = FunctionsExpressionWithExpressionAgentCompletionsMessageSimpleContentExpressionSchema;
exports.FunctionsExpressionWithExpressionAgentCompletionsMessageSimpleContentPartExpressionSchema = FunctionsExpressionWithExpressionAgentCompletionsMessageSimpleContentPartExpressionSchema;
exports.FunctionsExpressionWithExpressionAgentCompletionsMessageVideoUrlSchema = FunctionsExpressionWithExpressionAgentCompletionsMessageVideoUrlSchema;
exports.FunctionsExpressionWithExpressionArrayOfFunctionsExpressionWithExpressionAgentCompletionsMessageMessageExpressionSchema = FunctionsExpressionWithExpressionArrayOfFunctionsExpressionWithExpressionAgentCompletionsMessageMessageExpressionSchema;
exports.FunctionsExpressionWithExpressionArrayOfFunctionsExpressionWithExpressionAgentCompletionsMessageRichContentExpressionSchema = FunctionsExpressionWithExpressionArrayOfFunctionsExpressionWithExpressionAgentCompletionsMessageRichContentExpressionSchema;
exports.FunctionsExpressionWithExpressionFunctionsExpressionInputValueExpressionSchema = FunctionsExpressionWithExpressionFunctionsExpressionInputValueExpressionSchema;
exports.FunctionsExpressionWithExpressionNullableAgentCompletionsMessageRichContentExpressionSchema = FunctionsExpressionWithExpressionNullableAgentCompletionsMessageRichContentExpressionSchema;
exports.FunctionsExpressionWithExpressionNullableArrayOfFunctionsExpressionWithExpressionAgentCompletionsMessageAssistantToolCallExpressionSchema = FunctionsExpressionWithExpressionNullableArrayOfFunctionsExpressionWithExpressionAgentCompletionsMessageAssistantToolCallExpressionSchema;
exports.FunctionsExpressionWithExpressionNullableStringSchema = FunctionsExpressionWithExpressionNullableStringSchema;
exports.FunctionsExpressionWithExpressionStringSchema = FunctionsExpressionWithExpressionStringSchema;
exports.FunctionsFullInlineFunctionSchema = FunctionsFullInlineFunctionSchema;
exports.FunctionsFullRemoteFunctionSchema = FunctionsFullRemoteFunctionSchema;
exports.FunctionsFunctionSchema = FunctionsFunctionSchema;
exports.FunctionsFunctionTypeSchema = FunctionsFunctionTypeSchema;
exports.FunctionsGetFunctionProfilePairSchema = FunctionsGetFunctionProfilePairSchema;
exports.FunctionsGetFunctionSchema = FunctionsGetFunctionSchema;
exports.FunctionsInlineAutoProfileSchema = FunctionsInlineAutoProfileSchema;
exports.FunctionsInlineFunctionSchema = FunctionsInlineFunctionSchema;
exports.FunctionsInlineProfileSchema = FunctionsInlineProfileSchema;
exports.FunctionsInlineTasksProfileSchema = FunctionsInlineTasksProfileSchema;
exports.FunctionsInventionsDescriptionObjectSchema = FunctionsInventionsDescriptionObjectSchema;
exports.FunctionsInventionsEssayObjectSchema = FunctionsInventionsEssayObjectSchema;
exports.FunctionsInventionsEssayTasksObjectSchema = FunctionsInventionsEssayTasksObjectSchema;
exports.FunctionsInventionsIndexObjectSchema = FunctionsInventionsIndexObjectSchema;
exports.FunctionsInventionsRecursiveRequestFunctionInventionRecursiveCreateParamsSchema = FunctionsInventionsRecursiveRequestFunctionInventionRecursiveCreateParamsSchema;
exports.FunctionsInventionsRecursiveRequestFunctionInventionRecursiveCreateParamsStreamingSchema = FunctionsInventionsRecursiveRequestFunctionInventionRecursiveCreateParamsStreamingSchema;
exports.FunctionsInventionsRecursiveRequestFunctionInventionRecursiveCreateParamsUnarySchema = FunctionsInventionsRecursiveRequestFunctionInventionRecursiveCreateParamsUnarySchema;
exports.FunctionsInventionsRecursiveResponseStreamingFunctionInventionChunkSchema = FunctionsInventionsRecursiveResponseStreamingFunctionInventionChunkSchema;
exports.FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunkSchema = FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunkSchema;
exports.FunctionsInventionsRecursiveResponseStreamingObjectSchema = FunctionsInventionsRecursiveResponseStreamingObjectSchema;
exports.FunctionsInventionsRecursiveResponseUnaryFunctionInventionRecursiveSchema = FunctionsInventionsRecursiveResponseUnaryFunctionInventionRecursiveSchema;
exports.FunctionsInventionsRecursiveResponseUnaryFunctionInventionSchema = FunctionsInventionsRecursiveResponseUnaryFunctionInventionSchema;
exports.FunctionsInventionsRecursiveResponseUnaryObjectSchema = FunctionsInventionsRecursiveResponseUnaryObjectSchema;
exports.FunctionsInventionsRequestFunctionInventionCreateParamsSchema = FunctionsInventionsRequestFunctionInventionCreateParamsSchema;
exports.FunctionsInventionsRequestFunctionInventionCreateParamsStreamingSchema = FunctionsInventionsRequestFunctionInventionCreateParamsStreamingSchema;
exports.FunctionsInventionsRequestFunctionInventionCreateParamsUnarySchema = FunctionsInventionsRequestFunctionInventionCreateParamsUnarySchema;
exports.FunctionsInventionsResponseStreamingAgentCompletionChunkSchema = FunctionsInventionsResponseStreamingAgentCompletionChunkSchema;
exports.FunctionsInventionsResponseStreamingFunctionInventionChunkSchema = FunctionsInventionsResponseStreamingFunctionInventionChunkSchema;
exports.FunctionsInventionsResponseStreamingObjectSchema = FunctionsInventionsResponseStreamingObjectSchema;
exports.FunctionsInventionsResponseUnaryAgentCompletionSchema = FunctionsInventionsResponseUnaryAgentCompletionSchema;
exports.FunctionsInventionsResponseUnaryFunctionInventionSchema = FunctionsInventionsResponseUnaryFunctionInventionSchema;
exports.FunctionsInventionsResponseUnaryObjectSchema = FunctionsInventionsResponseUnaryObjectSchema;
exports.FunctionsInventionsStateAlphaScalarBranchStateSchema = FunctionsInventionsStateAlphaScalarBranchStateSchema;
exports.FunctionsInventionsStateAlphaScalarLeafStateSchema = FunctionsInventionsStateAlphaScalarLeafStateSchema;
exports.FunctionsInventionsStateAlphaScalarStateSchema = FunctionsInventionsStateAlphaScalarStateSchema;
exports.FunctionsInventionsStateAlphaVectorBranchStateSchema = FunctionsInventionsStateAlphaVectorBranchStateSchema;
exports.FunctionsInventionsStateAlphaVectorLeafStateSchema = FunctionsInventionsStateAlphaVectorLeafStateSchema;
exports.FunctionsInventionsStateAlphaVectorStateSchema = FunctionsInventionsStateAlphaVectorStateSchema;
exports.FunctionsInventionsStateParamsSchema = FunctionsInventionsStateParamsSchema;
exports.FunctionsInventionsStateParamsStateSchema = FunctionsInventionsStateParamsStateSchema;
exports.FunctionsInventionsStateStateSchema = FunctionsInventionsStateStateSchema;
exports.FunctionsInventionsTasksLengthObjectSchema = FunctionsInventionsTasksLengthObjectSchema;
exports.FunctionsListFunctionItemSchema = FunctionsListFunctionItemSchema;
exports.FunctionsListFunctionProfilePairItemSchema = FunctionsListFunctionProfilePairItemSchema;
exports.FunctionsListFunctionProfilePairSchema = FunctionsListFunctionProfilePairSchema;
exports.FunctionsListFunctionProfilePairsQueryParametersSchema = FunctionsListFunctionProfilePairsQueryParametersSchema;
exports.FunctionsListFunctionProfilePairsSourceSchema = FunctionsListFunctionProfilePairsSourceSchema;
exports.FunctionsListFunctionSchema = FunctionsListFunctionSchema;
exports.FunctionsListFunctionsQueryParametersSchema = FunctionsListFunctionsQueryParametersSchema;
exports.FunctionsListFunctionsSourceSchema = FunctionsListFunctionsSourceSchema;
exports.FunctionsPlaceholderScalarFunctionTaskExpressionSchema = FunctionsPlaceholderScalarFunctionTaskExpressionSchema;
exports.FunctionsPlaceholderScalarFunctionTaskSchema = FunctionsPlaceholderScalarFunctionTaskSchema;
exports.FunctionsPlaceholderVectorFunctionTaskExpressionSchema = FunctionsPlaceholderVectorFunctionTaskExpressionSchema;
exports.FunctionsPlaceholderVectorFunctionTaskSchema = FunctionsPlaceholderVectorFunctionTaskSchema;
exports.FunctionsProfileSchema = FunctionsProfileSchema;
exports.FunctionsProfilesComputationsRequestDatasetItemSchema = FunctionsProfilesComputationsRequestDatasetItemSchema;
exports.FunctionsProfilesComputationsRequestFunctionInlineRequestBodySchema = FunctionsProfilesComputationsRequestFunctionInlineRequestBodySchema;
exports.FunctionsProfilesComputationsRequestFunctionProfileComputationCreateParamsSchema = FunctionsProfilesComputationsRequestFunctionProfileComputationCreateParamsSchema;
exports.FunctionsProfilesComputationsRequestFunctionRemoteRequestBodySchema = FunctionsProfilesComputationsRequestFunctionRemoteRequestBodySchema;
exports.FunctionsProfilesComputationsRequestFunctionRemoteRequestPathSchema = FunctionsProfilesComputationsRequestFunctionRemoteRequestPathSchema;
exports.FunctionsProfilesComputationsRequestRequestSchema = FunctionsProfilesComputationsRequestRequestSchema;
exports.FunctionsProfilesComputationsRequestTargetSchema = FunctionsProfilesComputationsRequestTargetSchema;
exports.FunctionsProfilesComputationsResponseFittingStatsSchema = FunctionsProfilesComputationsResponseFittingStatsSchema;
exports.FunctionsProfilesComputationsResponseStreamingFunctionExecutionChunkSchema = FunctionsProfilesComputationsResponseStreamingFunctionExecutionChunkSchema;
exports.FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunkSchema = FunctionsProfilesComputationsResponseStreamingFunctionProfileComputationChunkSchema;
exports.FunctionsProfilesComputationsResponseStreamingObjectSchema = FunctionsProfilesComputationsResponseStreamingObjectSchema;
exports.FunctionsProfilesComputationsResponseUnaryFunctionExecutionSchema = FunctionsProfilesComputationsResponseUnaryFunctionExecutionSchema;
exports.FunctionsProfilesComputationsResponseUnaryFunctionProfileComputationSchema = FunctionsProfilesComputationsResponseUnaryFunctionProfileComputationSchema;
exports.FunctionsProfilesComputationsResponseUnaryObjectSchema = FunctionsProfilesComputationsResponseUnaryObjectSchema;
exports.FunctionsProfilesComputationsRetryTokenSchema = FunctionsProfilesComputationsRetryTokenSchema;
exports.FunctionsProfilesGetProfileSchema = FunctionsProfilesGetProfileSchema;
exports.FunctionsProfilesListProfileItemSchema = FunctionsProfilesListProfileItemSchema;
exports.FunctionsProfilesListProfileSchema = FunctionsProfilesListProfileSchema;
exports.FunctionsProfilesListProfilesQueryParametersSchema = FunctionsProfilesListProfilesQueryParametersSchema;
exports.FunctionsProfilesListProfilesSourceSchema = FunctionsProfilesListProfilesSourceSchema;
exports.FunctionsProfilesUsageProfileSchema = FunctionsProfilesUsageProfileSchema;
exports.FunctionsRemoteAutoProfileSchema = FunctionsRemoteAutoProfileSchema;
exports.FunctionsRemoteFunctionPathSchema = FunctionsRemoteFunctionPathSchema;
exports.FunctionsRemoteFunctionSchema = FunctionsRemoteFunctionSchema;
exports.FunctionsRemoteProfileSchema = FunctionsRemoteProfileSchema;
exports.FunctionsRemoteSchema = FunctionsRemoteSchema;
exports.FunctionsRemoteTasksProfileSchema = FunctionsRemoteTasksProfileSchema;
exports.FunctionsScalarFunctionTaskExpressionSchema = FunctionsScalarFunctionTaskExpressionSchema;
exports.FunctionsScalarFunctionTaskSchema = FunctionsScalarFunctionTaskSchema;
exports.FunctionsTaskExpressionSchema = FunctionsTaskExpressionSchema;
exports.FunctionsTaskProfileSchema = FunctionsTaskProfileSchema;
exports.FunctionsTaskSchema = FunctionsTaskSchema;
exports.FunctionsUsageFunctionProfilePairSchema = FunctionsUsageFunctionProfilePairSchema;
exports.FunctionsUsageFunctionSchema = FunctionsUsageFunctionSchema;
exports.FunctionsVectorCompletionTaskExpressionSchema = FunctionsVectorCompletionTaskExpressionSchema;
exports.FunctionsVectorCompletionTaskSchema = FunctionsVectorCompletionTaskSchema;
exports.FunctionsVectorFunctionTaskExpressionSchema = FunctionsVectorFunctionTaskExpressionSchema;
exports.FunctionsVectorFunctionTaskSchema = FunctionsVectorFunctionTaskSchema;
exports.ObjectiveAI = ObjectiveAI;
exports.ObjectiveAIFetchError = ObjectiveAIFetchError;
exports.ObjectiveAIOptionsSchema = ObjectiveAIOptionsSchema;
exports.PrefixedUuidSchema = PrefixedUuidSchema;
exports.RequestOptionsSchema = RequestOptionsSchema;
exports.ResponseErrorSchema = ResponseErrorSchema;
exports.Stream = Stream;
exports.VectorCompletionsCacheCacheVoteRequestOwnedSchema = VectorCompletionsCacheCacheVoteRequestOwnedSchema;
exports.VectorCompletionsCacheCacheVoteRequestRefSchema = VectorCompletionsCacheCacheVoteRequestRefSchema;
exports.VectorCompletionsCacheCacheVoteRequestSchema = VectorCompletionsCacheCacheVoteRequestSchema;
exports.VectorCompletionsCacheCacheVoteSchema = VectorCompletionsCacheCacheVoteSchema;
exports.VectorCompletionsCacheCompletionVotesSchema = VectorCompletionsCacheCompletionVotesSchema;
exports.VectorCompletionsRequestEnsembleSchema = VectorCompletionsRequestEnsembleSchema;
exports.VectorCompletionsRequestProfileEntrySchema = VectorCompletionsRequestProfileEntrySchema;
exports.VectorCompletionsRequestProfileSchema = VectorCompletionsRequestProfileSchema;
exports.VectorCompletionsRequestVectorCompletionCreateParamsSchema = VectorCompletionsRequestVectorCompletionCreateParamsSchema;
exports.VectorCompletionsRequestVectorCompletionCreateParamsStreamingSchema = VectorCompletionsRequestVectorCompletionCreateParamsStreamingSchema;
exports.VectorCompletionsRequestVectorCompletionCreateParamsUnarySchema = VectorCompletionsRequestVectorCompletionCreateParamsUnarySchema;
exports.VectorCompletionsResponseStreamingAgentCompletionChunkSchema = VectorCompletionsResponseStreamingAgentCompletionChunkSchema;
exports.VectorCompletionsResponseStreamingObjectSchema = VectorCompletionsResponseStreamingObjectSchema;
exports.VectorCompletionsResponseStreamingVectorCompletionChunkSchema = VectorCompletionsResponseStreamingVectorCompletionChunkSchema;
exports.VectorCompletionsResponseUnaryAgentCompletionSchema = VectorCompletionsResponseUnaryAgentCompletionSchema;
exports.VectorCompletionsResponseUnaryObjectSchema = VectorCompletionsResponseUnaryObjectSchema;
exports.VectorCompletionsResponseUnaryVectorCompletionSchema = VectorCompletionsResponseUnaryVectorCompletionSchema;
exports.VectorCompletionsResponseVoteSchema = VectorCompletionsResponseVoteSchema;
exports.VectorCompletionsVectorResponsesSchema = VectorCompletionsVectorResponsesSchema;
exports.agentCompletionsCreateAgentCompletion = agentCompletionsCreateAgentCompletion;
exports.agentCompletionsMessageAssistantToolCallDeltaMerged = agentCompletionsMessageAssistantToolCallDeltaMerged;
exports.agentCompletionsMessageAssistantToolCallDeltaMergedList = agentCompletionsMessageAssistantToolCallDeltaMergedList;
exports.agentCompletionsMessageAssistantToolCallFunctionDeltaMerged = agentCompletionsMessageAssistantToolCallFunctionDeltaMerged;
exports.agentCompletionsMessageRichContentMerged = agentCompletionsMessageRichContentMerged;
exports.agentCompletionsResponseCompletionTokensDetailsMerged = agentCompletionsResponseCompletionTokensDetailsMerged;
exports.agentCompletionsResponseCostDetailsMerged = agentCompletionsResponseCostDetailsMerged;
exports.agentCompletionsResponseLogprobsMerged = agentCompletionsResponseLogprobsMerged;
exports.agentCompletionsResponsePromptTokensDetailsMerged = agentCompletionsResponsePromptTokensDetailsMerged;
exports.agentCompletionsResponseStreamingAgentCompletionChunkMerged = agentCompletionsResponseStreamingAgentCompletionChunkMerged;
exports.agentCompletionsResponseStreamingAssistantResponseChunkMerged = agentCompletionsResponseStreamingAssistantResponseChunkMerged;
exports.agentCompletionsResponseStreamingMessageChunkMerged = agentCompletionsResponseStreamingMessageChunkMerged;
exports.agentCompletionsResponseStreamingMessageChunkMergedList = agentCompletionsResponseStreamingMessageChunkMergedList;
exports.agentCompletionsResponseUpstreamUsageMerged = agentCompletionsResponseUpstreamUsageMerged;
exports.agentCompletionsResponseUsageMerged = agentCompletionsResponseUsageMerged;
exports.agentGetAgent = agentGetAgent;
exports.agentGetAgentUsage = agentGetAgentUsage;
exports.agentListAgents = agentListAgents;
exports.authCreateApiKey = authCreateApiKey;
exports.authCreateOpenrouterByokApiKey = authCreateOpenrouterByokApiKey;
exports.authDeleteOpenrouterByokApiKey = authDeleteOpenrouterByokApiKey;
exports.authDisableApiKey = authDisableApiKey;
exports.authGetCredits = authGetCredits;
exports.authGetOpenrouterByokApiKey = authGetOpenrouterByokApiKey;
exports.authListApiKeys = authListApiKeys;
exports.ensembleGetEnsemble = ensembleGetEnsemble;
exports.ensembleGetEnsembleUsage = ensembleGetEnsembleUsage;
exports.ensembleListEnsembles = ensembleListEnsembles;
exports.functionsExecutionsCreateFunctionExecution = functionsExecutionsCreateFunctionExecution;
exports.functionsExecutionsResponseStreamingFunctionExecutionChunkMerged = functionsExecutionsResponseStreamingFunctionExecutionChunkMerged;
exports.functionsExecutionsResponseStreamingReasoningSummaryChunkMerged = functionsExecutionsResponseStreamingReasoningSummaryChunkMerged;
exports.functionsExecutionsResponseStreamingTaskChunkMerged = functionsExecutionsResponseStreamingTaskChunkMerged;
exports.functionsExecutionsResponseStreamingTaskChunkMergedList = functionsExecutionsResponseStreamingTaskChunkMergedList;
exports.functionsExecutionsResponseStreamingVectorCompletionTaskChunkMerged = functionsExecutionsResponseStreamingVectorCompletionTaskChunkMerged;
exports.functionsGetFunction = functionsGetFunction;
exports.functionsGetFunctionProfilePairUsage = functionsGetFunctionProfilePairUsage;
exports.functionsGetFunctionUsage = functionsGetFunctionUsage;
exports.functionsInventionsCreateFunctionInvention = functionsInventionsCreateFunctionInvention;
exports.functionsInventionsRecursiveCreateFunctionInventionRecursive = functionsInventionsRecursiveCreateFunctionInventionRecursive;
exports.functionsInventionsRecursiveResponseStreamingFunctionInventionChunkMerged = functionsInventionsRecursiveResponseStreamingFunctionInventionChunkMerged;
exports.functionsInventionsRecursiveResponseStreamingFunctionInventionChunkMergedList = functionsInventionsRecursiveResponseStreamingFunctionInventionChunkMergedList;
exports.functionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunkMerged = functionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunkMerged;
exports.functionsInventionsResponseStreamingAgentCompletionChunkMerged = functionsInventionsResponseStreamingAgentCompletionChunkMerged;
exports.functionsInventionsResponseStreamingAgentCompletionChunkMergedList = functionsInventionsResponseStreamingAgentCompletionChunkMergedList;
exports.functionsInventionsResponseStreamingFunctionInventionChunkMerged = functionsInventionsResponseStreamingFunctionInventionChunkMerged;
exports.functionsListFunctionProfilePairs = functionsListFunctionProfilePairs;
exports.functionsListFunctions = functionsListFunctions;
exports.functionsProfilesComputationsComputeProfile = functionsProfilesComputationsComputeProfile;
exports.functionsProfilesComputationsResponseStreamingFunctionExecutionChunkMerged = functionsProfilesComputationsResponseStreamingFunctionExecutionChunkMerged;
exports.functionsProfilesComputationsResponseStreamingFunctionExecutionChunkMergedList = functionsProfilesComputationsResponseStreamingFunctionExecutionChunkMergedList;
exports.functionsProfilesComputationsResponseStreamingFunctionProfileComputationChunkMerged = functionsProfilesComputationsResponseStreamingFunctionProfileComputationChunkMerged;
exports.functionsProfilesGetProfile = functionsProfilesGetProfile;
exports.functionsProfilesGetProfileUsage = functionsProfilesGetProfileUsage;
exports.functionsProfilesListProfiles = functionsProfilesListProfiles;
exports.isResponseError = isResponseError;
exports.merge = merge;
exports.mergedNumberArray = mergedNumberArray;
exports.mergedString = mergedString;
exports.numberIsEmpty = numberIsEmpty;
exports.vectorCompletionsCacheGetCacheVote = vectorCompletionsCacheGetCacheVote;
exports.vectorCompletionsCacheGetCompletionVotes = vectorCompletionsCacheGetCompletionVotes;
exports.vectorCompletionsCreateVectorCompletion = vectorCompletionsCreateVectorCompletion;
exports.vectorCompletionsResponseStreamingAgentCompletionChunkMerged = vectorCompletionsResponseStreamingAgentCompletionChunkMerged;
exports.vectorCompletionsResponseStreamingAgentCompletionChunkMergedList = vectorCompletionsResponseStreamingAgentCompletionChunkMergedList;
exports.vectorCompletionsResponseStreamingVectorCompletionChunkMerged = vectorCompletionsResponseStreamingVectorCompletionChunkMerged;
exports.vectorCompletionsResponseVoteMergedList = vectorCompletionsResponseVoteMergedList;
exports.zockerParse = zockerParse;
