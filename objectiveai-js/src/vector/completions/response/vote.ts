import { z } from "zod";

export const VectorCompletionsResponseVoteSchema = z.object({
  agent: z.string().describe("The agent that produced this vote (content-addressed ID)."),
  ensemble_index: z.number().int().min(0).meta({ format: "uint64" }).describe("Index of the agent configuration within the ensemble."),
  flat_ensemble_index: z.number().int().min(0).meta({ format: "uint64" }).describe("Flattened index accounting for agent counts in the ensemble."),
  prompt_id: z.string().describe("Content hash of the request messages (for caching/deduplication)."),
  responses_ids: z.array(z.string()).describe("Content hashes of each response option in the request."),
  vote: z.array(z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()])).describe("The vote distribution. Each index corresponds to a response from the\nrequest. Typically one element is 1.0 (selected) and the rest are 0.0."),
  weight: z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()]).describe("The weight applied to this vote when computing final scores."),
  retry: z.boolean().nullable().describe("If true, this vote was reused from a previous request via the `retry`\nparameter. All fields reflect the original request's values.").optional(),
  from_cache: z.boolean().nullable().describe("If true, this vote was retrieved from cache rather than generated fresh.").optional(),
}).describe("A single LLM's vote in a vector completion.\n\nEach LLM in the ensemble produces a vote indicating which response(s) it\nselected. Votes are weighted according to the profile and combined to\nproduce the final scores.\n\n# Vote Format\n\nThe `vote` field is a vector of decimals corresponding to the responses\nin the request. Typically one element is 1.0 and the rest are 0.0 (discrete\nselection), but when `top_logprobs` is used, votes may be probability\ndistributions.").meta({ title: "vector.completions.response.Vote" });
export type VectorCompletionsResponseVote = z.infer<typeof VectorCompletionsResponseVoteSchema>;
