import z from "zod";
import { VoteSchema } from "../response/vote";
import { convert, type JsonSchema } from "../../../jsonSchema";

export const CacheVoteSchema = z
  .object({
    vote: VoteSchema.optional().nullable(),
  })
  .describe("Response containing a cached vote if one was found.");
export type CacheVote = z.infer<typeof CacheVoteSchema>;
export const CacheVoteJsonSchema: JsonSchema = convert(CacheVoteSchema);
