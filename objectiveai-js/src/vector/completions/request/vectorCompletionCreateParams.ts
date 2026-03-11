import { z } from "zod";
import { AgentCompletionsMessageMessageSchema } from "../../../agent/completions/message/message";
import { AgentCompletionsMessageRichContentSchema } from "../../../agent/completions/message/richContent";
import { AgentCompletionsRequestProviderSchema } from "../../../agent/completions/request/provider";
import { VectorCompletionsRequestEnsembleSchema } from "./ensemble";
import { VectorCompletionsRequestProfileSchema } from "./profile";

export const VectorCompletionsRequestVectorCompletionCreateParamsSchema = z.object({
  retry: z.string().nullable().describe("If present, reuses votes from a previous request with this ID.").optional(),
  from_cache: z.boolean().nullable().describe("If true, uses cached votes when available.").optional(),
  messages: z.array(AgentCompletionsMessageMessageSchema).describe("The conversation messages (the prompt)."),
  provider: AgentCompletionsRequestProviderSchema.nullable().describe("Provider routing preferences.").optional(),
  ensemble: VectorCompletionsRequestEnsembleSchema.describe("The Ensemble of agents to use."),
  profile: VectorCompletionsRequestProfileSchema.describe("The profile weights for each agent in the ensemble.\n\nMust have the same length as the total agent count in the ensemble.\nCan be either:\n- A vector of decimals (legacy representation), or\n- A vector of objects with `weight` and optional `invert` fields."),
  seed: z.number().int().meta({ format: "int64" }).nullable().describe("Random seed for deterministic results.").optional(),
  stream: z.boolean().nullable().describe("Whether to stream the response.").optional(),
  responses: z.array(AgentCompletionsMessageRichContentSchema).describe("The possible responses the LLMs can vote for."),
  mcp_server_authorization: z.record(z.string(), z.string()).nullable().describe("Map from MCP server URL to authorization header value.").optional(),
}).describe("Parameters for creating a vector completion.\n\nVector completions run multiple agent completions (one per LLM in the\nensemble), force each to vote for one of the predefined responses, and\ncombine votes using the provided profile weights to produce final scores.").meta({ title: "vector.completions.request.VectorCompletionCreateParams" });
export type VectorCompletionsRequestVectorCompletionCreateParams = z.infer<typeof VectorCompletionsRequestVectorCompletionCreateParamsSchema>;
