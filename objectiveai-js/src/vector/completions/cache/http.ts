import { ObjectiveAI, type RequestOptions } from "../../../client";
import type { VectorCompletionsCacheCompletionVotes } from "./completionVotes";
import type { VectorCompletionsCacheCacheVote } from "./cacheVote";
import type { VectorCompletionsCacheCacheVoteRequestOwned } from "./cacheVoteRequestOwned";

export function vectorCompletionsCacheGetCompletionVotes(
  client: ObjectiveAI,
  id: string,
  options?: RequestOptions,
): Promise<VectorCompletionsCacheCompletionVotes> {
  return client.get_unary<VectorCompletionsCacheCompletionVotes>(
    `/vector/completions/${id}`,
    undefined,
    options,
  );
}

export function vectorCompletionsCacheGetCacheVote(
  client: ObjectiveAI,
  body: VectorCompletionsCacheCacheVoteRequestOwned,
  options?: RequestOptions,
): Promise<VectorCompletionsCacheCacheVote> {
  return client.post_unary<VectorCompletionsCacheCacheVote>(
    "/vector/completions/cache",
    body,
    options,
  );
}
