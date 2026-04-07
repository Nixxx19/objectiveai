import { ObjectiveAI, type RequestOptions } from "../../../client";
import type { VectorCompletionsCacheCompletionVotes } from "./completionVotes";
import type { VectorCompletionsCacheCacheVote } from "./cacheVote";
import type { VectorCompletionsCacheCacheVoteRequest } from "./cacheVoteRequest";

export function vectorCompletionsCacheGetCompletionVotes(
  client: ObjectiveAI,
  id: string,
  options?: RequestOptions,
): Promise<VectorCompletionsCacheCompletionVotes> {
  return client.get_unary<VectorCompletionsCacheCompletionVotes>(
    "vector/completions/votes",
    { id },
    options,
  );
}

export function vectorCompletionsCacheGetCacheVote(
  client: ObjectiveAI,
  body: VectorCompletionsCacheCacheVoteRequest,
  options?: RequestOptions,
): Promise<VectorCompletionsCacheCacheVote> {
  return client.get_unary<VectorCompletionsCacheCacheVote>(
    "vector/completions/cache",
    body,
    options,
  );
}
