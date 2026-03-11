import { z } from "zod";
import { VectorCompletionsResponseVoteSchema } from "../response/vote";

export const VectorCompletionsCacheCacheVoteSchema = z.object({
  vote: VectorCompletionsResponseVoteSchema.nullable().optional(),
}).meta({ title: "vector.completions.cache.CacheVote" });
export type VectorCompletionsCacheCacheVote = z.infer<typeof VectorCompletionsCacheCacheVoteSchema>;
