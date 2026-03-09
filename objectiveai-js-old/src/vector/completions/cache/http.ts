import { ObjectiveAI, RequestOptions } from "../../../client";
import { CacheVoteRequest } from "./request";
import { CacheVote } from "./response";

export function retrieve(
  client: ObjectiveAI,
  request: CacheVoteRequest,
  options?: RequestOptions,
): Promise<CacheVote> {
  // Using POST to match backend implementation
  return client.post_unary<CacheVote>(
    "/vector/completions/cache",
    request,
    options,
  );
}
