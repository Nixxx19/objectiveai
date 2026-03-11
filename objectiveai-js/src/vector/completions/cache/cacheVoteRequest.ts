import { z } from "zod";
import { VectorCompletionsCacheCacheVoteRequestOwnedSchema } from "./cacheVoteRequestOwned";
import { VectorCompletionsCacheCacheVoteRequestRefSchema } from "./cacheVoteRequestRef";

export const VectorCompletionsCacheCacheVoteRequestSchema = z.union([z.object({
  Ref: VectorCompletionsCacheCacheVoteRequestRefSchema,
}).strict(), z.object({
  Owned: VectorCompletionsCacheCacheVoteRequestOwnedSchema,
}).strict()]).meta({ title: "vector.completions.cache.CacheVoteRequest" });
export type VectorCompletionsCacheCacheVoteRequest = z.infer<typeof VectorCompletionsCacheCacheVoteRequestSchema>;
