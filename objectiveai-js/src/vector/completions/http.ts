import z from "zod";
import { convert, type JsonSchema } from "../../jsonSchema";
import { ObjectiveAI, RequestOptions } from "../../client";
import { Stream } from "../../stream";
import { VectorCompletion } from "./response/unary/vectorCompletion";
import {
  VectorCompletionCreateParams,
  VectorCompletionCreateParamsStreaming,
  VectorCompletionCreateParamsNonStreaming,
} from "./request/vectorCompletionCreateParams";
import { VectorCompletionChunk } from "./response/streaming/vectorCompletionChunk";
import { VotesSchema } from "./response/vote";

export function create(
  client: ObjectiveAI,
  body: VectorCompletionCreateParamsStreaming,
  options?: RequestOptions,
): Promise<Stream<VectorCompletionChunk>>;
export function create(
  client: ObjectiveAI,
  body: VectorCompletionCreateParamsNonStreaming,
  options?: RequestOptions,
): Promise<VectorCompletion>;
export function create(
  client: ObjectiveAI,
  body: VectorCompletionCreateParams,
  options?: RequestOptions,
): Promise<Stream<VectorCompletionChunk> | VectorCompletion> {
  if (body.stream) {
    return client.post_streaming<VectorCompletionChunk>(
      "/vector/completions",
      body,
      options,
    );
  }
  return client.post_unary<VectorCompletion>(
    "/vector/completions",
    body,
    options,
  );
}

export const RetrieveSchema = z
  .object({
    data: VotesSchema.optional().nullable(),
  })
  .describe("Response containing votes from a historical vector completion.");
export type Retrieve = z.infer<typeof RetrieveSchema>;
export const RetrieveJsonSchema: JsonSchema = convert(RetrieveSchema);

export function retrieve(
  client: ObjectiveAI,
  id: string,
  options?: RequestOptions,
): Promise<Retrieve> {
  return client.post_unary<Retrieve>(
    `/vector/completions/${id}`,
    {},
    options,
  );
}
