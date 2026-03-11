import { z } from "zod";
import { AgentCompletionsMessageMessageSchema } from "../../../agent/completions/message/message";
import { AgentCompletionsMessageRichContentSchema } from "../../../agent/completions/message/richContent";
import { AgentCompletionsRequestAgentSchema } from "../../../agent/completions/request/agent";

export const VectorCompletionsCacheCacheVoteRequestOwnedSchema = z.object({
  agent: AgentCompletionsRequestAgentSchema,
  agents: z.array(AgentCompletionsRequestAgentSchema).nullable().optional(),
  messages: z.array(AgentCompletionsMessageMessageSchema),
  responses: z.array(AgentCompletionsMessageRichContentSchema),
}).meta({ title: "vector.completions.cache.CacheVoteRequestOwned" });
export type VectorCompletionsCacheCacheVoteRequestOwned = z.infer<typeof VectorCompletionsCacheCacheVoteRequestOwnedSchema>;
