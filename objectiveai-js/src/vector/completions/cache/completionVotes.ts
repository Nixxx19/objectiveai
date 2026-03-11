import { z } from "zod";
import { VectorCompletionsResponseVoteSchema } from "../response/vote";

export const VectorCompletionsCacheCompletionVotesSchema = z.object({
  data: z.array(VectorCompletionsResponseVoteSchema).nullable().optional(),
}).meta({ title: "vector.completions.cache.CompletionVotes" });
export type VectorCompletionsCacheCompletionVotes = z.infer<typeof VectorCompletionsCacheCompletionVotesSchema>;
