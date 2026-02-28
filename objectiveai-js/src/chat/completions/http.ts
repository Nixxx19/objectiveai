import { ObjectiveAI, RequestOptions } from "../../client";
import { Stream } from "../../stream";
import { ChatCompletion } from "./response/unary/chatCompletion";
import {
  ChatCompletionCreateParams,
  ChatCompletionCreateParamsStreaming,
  ChatCompletionCreateParamsNonStreaming,
} from "./request/chatCompletionCreateParams";
import { ChatCompletionChunk } from "./response/streaming/chatCompletionChunk";

export function create(
  client: ObjectiveAI,
  body: ChatCompletionCreateParamsStreaming,
  options?: RequestOptions,
): Promise<Stream<ChatCompletionChunk>>;
export function create(
  client: ObjectiveAI,
  body: ChatCompletionCreateParamsNonStreaming,
  options?: RequestOptions,
): Promise<ChatCompletion>;
export function create(
  client: ObjectiveAI,
  body: ChatCompletionCreateParams,
  options?: RequestOptions,
): Promise<Stream<ChatCompletionChunk> | ChatCompletion> {
  if (body.stream) {
    return client.post_streaming<ChatCompletionChunk>(
      "/chat/completions",
      body,
      options,
    );
  }
  return client.post_unary<ChatCompletion>("/chat/completions", body, options);
}
