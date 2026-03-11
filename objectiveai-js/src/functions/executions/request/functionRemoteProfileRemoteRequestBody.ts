import { z } from "zod";
import { AgentCompletionsRequestProviderSchema } from "../../../agent/completions/request/provider";
import { FunctionsExecutionsRequestReasoningSchema } from "./reasoning";
import { FunctionsExecutionsRequestStrategySchema } from "./strategy";
import { FunctionsExpressionInputSchema } from "../../expression/input";

export const FunctionsExecutionsRequestFunctionRemoteProfileRemoteRequestBodySchema = z.object({
  retry_token: z.string().nullable().describe("If present, reuses votes from a previous execution with this token.").optional(),
  from_cache: z.boolean().nullable().describe("If true, uses cached votes when available.").optional(),
  reasoning: FunctionsExecutionsRequestReasoningSchema.nullable().describe("Reasoning summary configuration.").optional(),
  strategy: FunctionsExecutionsRequestStrategySchema.nullable().describe("Execution strategy.\nDefaults to `Default` strategy if not specified.").optional(),
  input: FunctionsExpressionInputSchema.describe("The input data to pass to the Function."),
  provider: AgentCompletionsRequestProviderSchema.nullable().describe("Provider routing preferences.").optional(),
  seed: z.number().int().meta({ format: "int64" }).nullable().describe("Random seed for deterministic results.").optional(),
  stream: z.boolean().nullable().describe("Whether to stream the response.").optional(),
  mcp_server_authorization: z.record(z.string(), z.string()).nullable().describe("Map from MCP server URL to authorization header value.").optional(),
}).describe("Base request body with common execution parameters.\n\nUsed directly for remote Function + remote Profile, or flattened into\nother request body types.").meta({ title: "functions.executions.request.FunctionRemoteProfileRemoteRequestBody" });
export type FunctionsExecutionsRequestFunctionRemoteProfileRemoteRequestBody = z.infer<typeof FunctionsExecutionsRequestFunctionRemoteProfileRemoteRequestBodySchema>;
