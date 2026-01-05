import OpenAI from "openai";
import { Stream } from "openai/streaming";

export namespace Chat {
  export namespace Completions {
    export namespace Request {
      export interface ChatCompletionCreateParamsBase {
        messages: Message[];
        model: string;
        frequency_penalty?: number | null;
        logit_bias?: Record<string, number> | null;
        logprobs?: boolean | null;
        max_completion_tokens?: number | null;
        modalities?: string[] | null;
        n?: number | null;
        parallel_tool_calls?: boolean | null;
        prediction?: Prediction | null;
        presence_penalty?: number | null;
        reasoning_effort?: ReasoningEffort | null;
        response_format?: ResponseFormat | null;
        seed?: number | null;
        service_tier?: ServiceTier | null;
        stop?: Stop | null;
        stream_options?: StreamOptions | null;
        temperature?: number | null;
        tool_choice?: ToolChoice | null;
        tools?: Tool[] | null;
        top_logprobs?: number | null;
        top_p?: number | null;
        web_search_options?: WebSearchOptions | null;
        max_tokens?: number | null;
        min_p?: number | null;
        plugins?: Plugin[] | null;
        provider?: ProviderPreferences | null;
        reasoning?: Reasoning | null;
        repetition_penalty?: number | null;
        top_a?: number | null;
        top_k?: number | null;
        usage?: Usage | null;
        verbosity?: Verbosity | null;
        models?: string[] | null;
      }

      export interface ChatCompletionCreateParamsStreaming
        extends ChatCompletionCreateParamsBase {
        stream: true;
      }

      export interface ChatCompletionCreateParamsNonStreaming
        extends ChatCompletionCreateParamsBase {
        stream?: false | null;
      }

      export type ChatCompletionCreateParams =
        | ChatCompletionCreateParamsStreaming
        | ChatCompletionCreateParamsNonStreaming;

      export type ServiceTier = "auto" | "default" | "flex";

      export type Stop = string | string[];

      export interface Prediction {
        content: Prediction.Content;
        type: "content";
      }

      export namespace Prediction {
        export type Content = string | Content.Part[];

        export namespace Content {
          export interface Part {
            text: string;
            type: "text";
          }
        }
      }

      export type ReasoningEffort = "minimal" | "low" | "medium" | "high";

      export type ResponseFormat =
        | ResponseFormat.Text
        | ResponseFormat.JsonObject
        | ResponseFormat.JsonSchema;

      export namespace ResponseFormat {
        export interface Text {
          type: "text";
        }

        export interface JsonObject {
          type: "json_object";
        }

        export interface JsonSchema {
          type: "json_schema";
          json_schema: JsonSchema.JsonSchema;
        }

        export namespace JsonSchema {
          export interface JsonSchema {
            name: string;
            description?: string | null;
            schema?: JsonValue;
            strict?: boolean | null;
          }
        }
      }

      export interface StreamOptions {
        include_usage?: boolean;
      }

      export type ToolChoice =
        | "none"
        | "auto"
        | "required"
        | ToolChoice.Function;

      export namespace ToolChoice {
        export interface Function {
          type: "function";
          function: Function.Function;
        }

        export namespace Function {
          export interface Function {
            name: string;
          }
        }
      }

      export interface WebSearchOptions {
        search_context_size?: WebSearchOptions.SearchContextSize | null;
        user_location?: WebSearchOptions.UserLocation | null;
      }

      export namespace WebSearchOptions {
        export type SearchContextSize = "low" | "medium" | "high";

        export interface UserLocation {
          approximate: UserLocation.Approximate;
          type: "approximate";
        }

        export namespace UserLocation {
          export interface Approximate {
            city?: string | null;
            country?: string | null;
            region?: string | null;
            // IANA format
            timezone?: string | null;
          }
        }
      }

      export type Tool = Tool.Function;

      export namespace Tool {
        export interface Function {
          function: Function.Definition;
          type: "function";
        }

        export namespace Function {
          export interface Definition {
            name: string;
            description?: string | null;
            parameters?: JsonValue;
            strict?: boolean | null;
          }
        }
      }

      export type Message =
        | Message.Developer
        | Message.System
        | Message.User
        | Message.Assistant
        | Message.Tool
        | Message.ChatCompletion
        | Message.ScoreCompletion
        | Message.MultichatCompletion;

      export namespace Message {
        export type SimpleContent = string | SimpleContentPart[];

        export interface SimpleContentPart {
          text: string;
          type: "text";
        }

        export type RichContent = string | RichContentPart[];

        export type RichContentPart =
          | RichContentPart.Text
          | RichContentPart.ImageUrl
          | RichContentPart.InputAudio
          | RichContentPart.InputVideo
          | RichContentPart.File;

        export namespace RichContentPart {
          export interface Text {
            text: string;
            type: "text";
          }

          export interface ImageUrl {
            image_url: ImageUrl.Definition;
            type: "image_url";
          }

          export namespace ImageUrl {
            export interface Definition {
              url: string;
              detail?: ImageUrl.Detail | null;
            }

            export type Detail = "auto" | "low" | "high";
          }

          export interface InputAudio {
            input_audio: InputAudio.Definition;
            type: "input_audio";
          }

          export namespace InputAudio {
            export interface Definition {
              data: string;
              format: Format;
            }
            export type Format = "wav" | "mp3";
          }

          export interface InputVideo {
            video_url: InputVideo.Definition;
            type: "video_url";
          }

          export namespace InputVideo {
            export interface Definition {
              url: string;
            }
          }

          export interface File {
            file: File.Definition;
            type: "file";
          }

          export namespace File {
            export interface Definition {
              file_data?: string | null;
              file_id?: string | null;
              filename?: string | null;
            }
          }
        }

        export interface Developer {
          role: "developer";
          content: SimpleContent;
          name?: string;
        }

        export interface System {
          role: "system";
          content: SimpleContent;
          name?: string;
        }

        export interface User {
          role: "user";
          content: RichContent;
          name?: string;
        }

        export interface Assistant {
          role: "assistant";
          content?: RichContent | null;
          name?: string | null;
          refusal?: string | null;
          tool_calls?: Assistant.ToolCall[] | null;
        }

        export namespace Assistant {
          export type ToolCall = ToolCall.Function;

          export namespace ToolCall {
            export interface Function {
              id: string;
              function: Function.Definition;
              type: "function";
            }

            export namespace Function {
              export interface Definition {
                name: string;
                arguments: string;
              }
            }
          }
        }

        export interface Tool {
          role: "tool";
          content: RichContent;
          tool_call_id: string;
        }

        export interface ChatCompletion {
          role: "chat_completion";
          id: string;
          choice_index: number;
          name?: string | null;
        }

        export interface ScoreCompletion {
          role: "score_completion";
          id: string;
          choice_index: number;
          name?: string | null;
        }

        export interface MultichatCompletion {
          role: "multichat_completion";
          id: string;
          choice_index: number;
          name?: string | null;
        }
      }

      export interface ProviderPreferences {
        order?: string[] | null;
        allow_fallbacks?: boolean | null;
        require_parameters?: boolean | null;
        data_collection?: ProviderPreferences.DataCollection | null;
        only?: string[] | null;
        ignore?: string[] | null;
        quantizations?: string[] | null;
        sort?: string | null;
      }

      export namespace ProviderPreferences {
        export type DataCollection = "allow" | "deny";
      }

      export interface Plugin {
        id: string;
        [key: string]: JsonValue;
      }

      export interface Reasoning {
        max_tokens?: number | null;
        effort?: ReasoningEffort | null;
        enabled?: boolean | null;
      }

      export interface Usage {
        include: boolean;
      }

      export type Verbosity = "low" | "medium" | "high";
    }

    export namespace Response {
      export namespace Streaming {
        export interface ChatCompletionChunk {
          id: string;
          choices: Choice[];
          created: number;
          model: string;
          object: "chat.completion.chunk";
          service_tier?: ServiceTier;
          system_fingerprint?: string;
          usage?: Usage;
          provider?: string;
        }

        export namespace ChatCompletionChunk {
          export function merged(
            a: ChatCompletionChunk,
            b: ChatCompletionChunk
          ): [ChatCompletionChunk, boolean] {
            const id = a.id;
            const [choices, choicesChanged] = Choice.mergedList(
              a.choices,
              b.choices
            );
            const created = a.created;
            const model = a.model;
            const object = a.object;
            const [service_tier, service_tierChanged] = merge(
              a.service_tier,
              b.service_tier
            );
            const [system_fingerprint, system_fingerprintChanged] = merge(
              a.system_fingerprint,
              b.system_fingerprint
            );
            const [usage, usageChanged] = merge(a.usage, b.usage, Usage.merged);
            const [provider, providerChanged] = merge(a.provider, b.provider);
            if (
              choicesChanged ||
              service_tierChanged ||
              system_fingerprintChanged ||
              usageChanged ||
              providerChanged
            ) {
              return [
                {
                  id,
                  choices,
                  created,
                  model,
                  object,
                  ...(service_tier !== undefined ? { service_tier } : {}),
                  ...(system_fingerprint !== undefined
                    ? { system_fingerprint }
                    : {}),
                  ...(usage !== undefined ? { usage } : {}),
                  ...(provider !== undefined ? { provider } : {}),
                },
                true,
              ];
            } else {
              return [a, false];
            }
          }
        }

        export interface Choice {
          delta: Delta;
          finish_reason: FinishReason | null;
          index: number;
          logprobs?: Logprobs;
        }

        export namespace Choice {
          export function merged(a: Choice, b: Choice): [Choice, boolean] {
            const [delta, deltaChanged] = merge(a.delta, b.delta, Delta.merged);
            const [finish_reason, finish_reasonChanged] = merge(
              a.finish_reason,
              b.finish_reason
            );
            const index = a.index;
            const [logprobs, logprobsChanged] = merge(
              a.logprobs,
              b.logprobs,
              Logprobs.merged
            );
            if (deltaChanged || finish_reasonChanged || logprobsChanged) {
              return [
                {
                  delta,
                  finish_reason,
                  index,
                  ...(logprobs !== undefined ? { logprobs } : {}),
                },
                true,
              ];
            } else {
              return [a, false];
            }
          }
          export function mergedList(
            a: Choice[],
            b: Choice[]
          ): [Choice[], boolean] {
            let merged: Choice[] | undefined = undefined;
            for (const choice of b) {
              const existingIndex = a.findIndex(
                ({ index }) => index === choice.index
              );
              if (existingIndex === -1) {
                if (merged === undefined) {
                  merged = [...a, choice];
                } else {
                  merged.push(choice);
                }
              } else {
                const [mergedChoice, choiceChanged] = Choice.merged(
                  a[existingIndex],
                  choice
                );
                if (choiceChanged) {
                  if (merged === undefined) {
                    merged = [...a];
                  }
                  merged[existingIndex] = mergedChoice;
                }
              }
            }
            return merged ? [merged, true] : [a, false];
          }
        }

        export interface Delta {
          content?: string;
          refusal?: string;
          role?: Role;
          tool_calls?: ToolCall[];
          reasoning?: string;
          images?: Image[];
        }

        export namespace Delta {
          export function merged(a: Delta, b: Delta): [Delta, boolean] {
            const [content, contentChanged] = merge(
              a.content,
              b.content,
              mergedString
            );
            const [refusal, refusalChanged] = merge(
              a.refusal,
              b.refusal,
              mergedString
            );
            const [role, roleChanged] = merge(a.role, b.role);
            const [tool_calls, tool_callsChanged] = merge(
              a.tool_calls,
              b.tool_calls,
              ToolCall.mergedList
            );
            const [reasoning, reasoningChanged] = merge(
              a.reasoning,
              b.reasoning,
              mergedString
            );
            const [images, imagesChanged] = merge(
              a.images,
              b.images,
              Image.mergedList
            );
            if (
              contentChanged ||
              reasoningChanged ||
              refusalChanged ||
              roleChanged ||
              tool_callsChanged ||
              imagesChanged
            ) {
              return [
                {
                  ...(content !== undefined ? { content } : {}),
                  ...(reasoning !== undefined ? { reasoning } : {}),
                  ...(refusal !== undefined ? { refusal } : {}),
                  ...(role !== undefined ? { role } : {}),
                  ...(tool_calls !== undefined ? { tool_calls } : {}),
                  ...(images !== undefined ? { images } : {}),
                },
                true,
              ];
            } else {
              return [a, false];
            }
          }
        }

        export type ToolCall = ToolCall.Function;

        export namespace ToolCall {
          export function merged(
            a: ToolCall,
            b: ToolCall
          ): [ToolCall, boolean] {
            return Function.merged(a, b);
          }
          export function mergedList(
            a: ToolCall[],
            b: ToolCall[]
          ): [ToolCall[], boolean] {
            let merged: ToolCall[] | undefined = undefined;
            for (const toolCall of b) {
              const existingIndex = a.findIndex(
                ({ index }) => index === toolCall.index
              );
              if (existingIndex === -1) {
                if (merged === undefined) {
                  merged = [...a, toolCall];
                } else {
                  merged.push(toolCall);
                }
              } else {
                const [mergedToolCall, toolCallChanged] = ToolCall.merged(
                  a[existingIndex],
                  toolCall
                );
                if (toolCallChanged) {
                  if (merged === undefined) {
                    merged = [...a];
                  }
                  merged[existingIndex] = mergedToolCall;
                }
              }
            }
            return merged ? [merged, true] : [a, false];
          }
          export interface Function {
            index: number;
            id?: string;
            function?: Function.Definition;
            type?: "function";
          }

          export namespace Function {
            export function merged(
              a: Function,
              b: Function
            ): [Function, boolean] {
              const index = a.index;
              const [id, idChanged] = merge(a.id, b.id);
              const [function_, functionChanged] = merge(
                a.function,
                b.function,
                Definition.merged
              );
              const [type, typeChanged] = merge(a.type, b.type);
              if (idChanged || functionChanged || typeChanged) {
                return [
                  {
                    index,
                    ...(id !== undefined ? { id } : {}),
                    ...(function_ !== undefined ? { function: function_ } : {}),
                    ...(type !== undefined ? { type } : {}),
                  },
                  true,
                ];
              } else {
                return [a, false];
              }
            }
            export interface Definition {
              name?: string;
              arguments?: string;
            }

            export namespace Definition {
              export function merged(
                a: Definition,
                b: Definition
              ): [Definition, boolean] {
                const [name, nameChanged] = merge(a.name, b.name);
                const [arguments_, argumentsChanged] = merge(
                  a.arguments,
                  b.arguments,
                  mergedString
                );
                if (nameChanged || argumentsChanged) {
                  return [
                    {
                      ...(name !== undefined ? { name } : {}),
                      ...(arguments_ !== undefined
                        ? { arguments: arguments_ }
                        : {}),
                    },
                    true,
                  ];
                } else {
                  return [a, false];
                }
              }
            }
          }
        }
      }

      export namespace Unary {
        export interface ChatCompletion {
          id: string;
          choices: Choice[];
          created: number;
          model: string;
          object: "chat.completion";
          service_tier?: ServiceTier;
          system_fingerprint?: string;
          usage?: Usage;
          provider?: string;
        }

        export interface Choice {
          message: Message;
          finish_reason: FinishReason;
          index: number;
          logprobs: Logprobs | null;
        }

        export interface Message {
          content: string | null;
          refusal: string | null;
          role: Role;
          annotations?: Annotation[];
          audio?: Audio;
          tool_calls?: ToolCall[];
          reasoning?: string;
          images?: Image[];
        }

        export type Annotation = Annotation.UrlCitation;

        export namespace Annotation {
          export interface UrlCitation {
            url_citation: UrlCitation.Definition;
          }

          export namespace UrlCitation {
            export interface Definition {
              end_index: number;
              start_index: number;
              title: string;
              url: string;
            }
          }
        }

        export interface Audio {
          id: string;
          data: string;
          expires_at: number;
          transcript: string;
        }

        export type ToolCall = ToolCall.Function;

        export namespace ToolCall {
          export interface Function {
            id: string;
            function: Function.Definition;
            type: "function";
          }

          export namespace Function {
            export interface Definition {
              name: string;
              arguments: string;
            }
          }
        }
      }

      export type ServiceTier = "auto" | "default" | "flex";

      export type FinishReason =
        | "stop"
        | "length"
        | "tool_calls"
        | "content_filter"
        | "error";

      export interface Usage {
        completion_tokens: number;
        prompt_tokens: number;
        total_tokens: number;
        completion_tokens_details?: Usage.CompletionTokensDetails;
        prompt_tokens_details?: Usage.PromptTokensDetails;
        cost?: number;
        cost_details?: Usage.CostDetails;
      }

      export namespace Usage {
        export function merged(a: Usage, b: Usage): [Usage, boolean] {
          const [completion_tokens, completion_tokensChanged] = merge(
            a.completion_tokens,
            b.completion_tokens,
            mergedNumber
          );
          const [prompt_tokens, prompt_tokensChanged] = merge(
            a.prompt_tokens,
            b.prompt_tokens,
            mergedNumber
          );
          const [total_tokens, total_tokensChanged] = merge(
            a.total_tokens,
            b.total_tokens,
            mergedNumber
          );
          const [completion_tokens_details, completion_tokens_detailsChanged] =
            merge(
              a.completion_tokens_details,
              b.completion_tokens_details,
              CompletionTokensDetails.merged
            );
          const [prompt_tokens_details, prompt_tokens_detailsChanged] = merge(
            a.prompt_tokens_details,
            b.prompt_tokens_details,
            PromptTokensDetails.merged
          );
          const [cost, costChanged] = merge(a.cost, b.cost, mergedNumber);
          const [cost_details, cost_detailsChanged] = merge(
            a.cost_details,
            b.cost_details,
            CostDetails.merged
          );
          if (
            completion_tokensChanged ||
            prompt_tokensChanged ||
            total_tokensChanged ||
            completion_tokens_detailsChanged ||
            prompt_tokens_detailsChanged ||
            costChanged ||
            cost_detailsChanged
          ) {
            return [
              {
                completion_tokens,
                prompt_tokens,
                total_tokens,
                ...(completion_tokens_details !== undefined
                  ? { completion_tokens_details }
                  : {}),
                ...(prompt_tokens_details !== undefined
                  ? { prompt_tokens_details }
                  : {}),
                ...(cost !== undefined ? { cost } : {}),
                ...(cost_details !== undefined ? { cost_details } : {}),
              },
              true,
            ];
          } else {
            return [a, false];
          }
        }
        export interface CompletionTokensDetails {
          accepted_prediction_tokens?: number;
          audio_tokens?: number;
          reasoning_tokens?: number;
          rejected_prediction_tokens?: number;
        }

        export namespace CompletionTokensDetails {
          export function merged(
            a: CompletionTokensDetails,
            b: CompletionTokensDetails
          ): [CompletionTokensDetails, boolean] {
            const [
              accepted_prediction_tokens,
              accepted_prediction_tokensChanged,
            ] = merge(
              a.accepted_prediction_tokens,
              b.accepted_prediction_tokens,
              mergedNumber
            );
            const [audio_tokens, audio_tokensChanged] = merge(
              a.audio_tokens,
              b.audio_tokens,
              mergedNumber
            );
            const [reasoning_tokens, reasoning_tokensChanged] = merge(
              a.reasoning_tokens,
              b.reasoning_tokens,
              mergedNumber
            );
            const [
              rejected_prediction_tokens,
              rejected_prediction_tokensChanged,
            ] = merge(
              a.rejected_prediction_tokens,
              b.rejected_prediction_tokens,
              mergedNumber
            );
            if (
              accepted_prediction_tokensChanged ||
              audio_tokensChanged ||
              reasoning_tokensChanged ||
              rejected_prediction_tokensChanged
            ) {
              return [
                {
                  ...(accepted_prediction_tokens !== undefined
                    ? { accepted_prediction_tokens }
                    : {}),
                  ...(audio_tokens !== undefined ? { audio_tokens } : {}),
                  ...(reasoning_tokens !== undefined
                    ? { reasoning_tokens }
                    : {}),
                  ...(rejected_prediction_tokens !== undefined
                    ? { rejected_prediction_tokens }
                    : {}),
                },
                true,
              ];
            } else {
              return [a, false];
            }
          }
        }

        export interface PromptTokensDetails {
          audio_tokens?: number;
          cached_tokens?: number;
        }

        export namespace PromptTokensDetails {
          export function merged(
            a: PromptTokensDetails,
            b: PromptTokensDetails
          ): [PromptTokensDetails, boolean] {
            const [audio_tokens, audio_tokensChanged] = merge(
              a.audio_tokens,
              b.audio_tokens,
              mergedNumber
            );
            const [cached_tokens, cached_tokensChanged] = merge(
              a.cached_tokens,
              b.cached_tokens,
              mergedNumber
            );
            if (audio_tokensChanged || cached_tokensChanged) {
              return [
                {
                  ...(audio_tokens !== undefined ? { audio_tokens } : {}),
                  ...(cached_tokens !== undefined ? { cached_tokens } : {}),
                },
                true,
              ];
            } else {
              return [a, false];
            }
          }
        }

        export interface CostDetails {
          upstream_inference_cost?: number;
          upstream_upstream_inference_cost?: number;
        }

        export namespace CostDetails {
          export function merged(
            a: CostDetails,
            b: CostDetails
          ): [CostDetails, boolean] {
            const [upstream_inference_cost, upstream_inference_costChanged] =
              merge(
                a.upstream_inference_cost,
                b.upstream_inference_cost,
                mergedNumber
              );
            const [
              upstream_upstream_inference_cost,
              upstream_upstream_inference_costChanged,
            ] = merge(
              a.upstream_upstream_inference_cost,
              b.upstream_upstream_inference_cost,
              mergedNumber
            );
            if (
              upstream_inference_costChanged ||
              upstream_upstream_inference_costChanged
            ) {
              return [
                {
                  ...(upstream_inference_cost !== undefined
                    ? { upstream_inference_cost }
                    : {}),
                  ...(upstream_upstream_inference_cost !== undefined
                    ? { upstream_upstream_inference_cost }
                    : {}),
                },
                true,
              ];
            } else {
              return [a, false];
            }
          }
        }
      }

      export interface Logprobs {
        content: Logprobs.Logprob[];
        refusal: Logprobs.Logprob[];
      }

      export namespace Logprobs {
        export function merged(a: Logprobs, b: Logprobs): [Logprobs, boolean] {
          const [content, contentChanged] = merge(
            a.content,
            b.content,
            Logprob.mergedList
          );
          const [refusal, refusalChanged] = merge(
            a.refusal,
            b.refusal,
            Logprob.mergedList
          );
          if (contentChanged || refusalChanged) {
            return [{ content, refusal }, true];
          } else {
            return [a, false];
          }
        }

        export interface Logprob {
          token: string;
          bytes: number[];
          logprob: number;
          top_logprobs: Logprob.TopLogprob[];
        }

        export namespace Logprob {
          export function mergedList(
            a: Logprob[],
            b: Logprob[]
          ): [Logprob[], boolean] {
            if (b.length === 0) {
              return [a, false];
            } else if (a.length === 0) {
              return [b, true];
            } else {
              return [[...a, ...b], true];
            }
          }

          export interface TopLogprob {
            token: string;
            bytes: number[] | null;
            logprob: number | null;
          }
        }
      }

      export type Role = "assistant";

      export type Image = Image.ImageUrl;

      export namespace Image {
        export function mergedList(a: Image[], b: Image[]): [Image[], boolean] {
          if (b.length === 0) {
            return [a, false];
          } else if (a.length === 0) {
            return [b, true];
          } else {
            return [[...a, ...b], true];
          }
        }
        export interface ImageUrl {
          type: "image_url";
          image_url: { url: string };
        }
      }
    }

    export interface ListOptions {
      count?: number | null;
      offset?: number | null;
    }

    export interface ListItem {
      id: string;
      preview: string | null;
      created: string; // RFC 3339 timestamp
    }

    export async function list(
      openai: OpenAI,
      listOptions?: ListOptions,
      options?: OpenAI.RequestOptions
    ): Promise<{ data: ListItem[] }> {
      const response = await openai.chat.completions.list({
        query: listOptions,
        ...options,
      });
      return response as unknown as { data: ListItem[] };
    }

    export async function publish(
      openai: OpenAI,
      id: string,
      options?: OpenAI.RequestOptions
    ): Promise<void> {
      await openai.post(`/chat/completions/${id}/publish`, options);
    }

    export async function retrieve(
      openai: OpenAI,
      id: string,
      options?: OpenAI.RequestOptions
    ): Promise<{
      request: Request.ChatCompletionCreateParams;
      response: Response.Unary.ChatCompletion;
    }> {
      const response = await openai.chat.completions.retrieve(id, options);
      return response as unknown as {
        request: Request.ChatCompletionCreateParams;
        response: Response.Unary.ChatCompletion;
      };
    }

    export async function create(
      openai: OpenAI,
      body: Request.ChatCompletionCreateParamsStreaming,
      options?: OpenAI.RequestOptions
    ): Promise<
      Stream<Response.Streaming.ChatCompletionChunk | ObjectiveAIError>
    >;
    export async function create(
      openai: OpenAI,
      body: Request.ChatCompletionCreateParamsNonStreaming,
      options?: OpenAI.RequestOptions
    ): Promise<Response.Unary.ChatCompletion>;
    export async function create(
      openai: OpenAI,
      body: Request.ChatCompletionCreateParams,
      options?: OpenAI.RequestOptions
    ): Promise<
      | Stream<Response.Streaming.ChatCompletionChunk | ObjectiveAIError>
      | Response.Unary.ChatCompletion
    > {
      const response = await openai.post("/chat/completions", {
        body,
        stream: body.stream ?? false,
        ...options,
      });
      return response as unknown as
        | Stream<Response.Streaming.ChatCompletionChunk | ObjectiveAIError>
        | Response.Unary.ChatCompletion;
    }
  }
}

export namespace Score {
  export namespace Completions {
    export namespace Request {
      export interface ChatCompletionCreateParamsBase {
        messages: Chat.Completions.Request.Message[];
        model: Model;
        seed?: number | null;
        service_tier?: Chat.Completions.Request.ServiceTier | null;
        stream_options?: Chat.Completions.Request.StreamOptions | null;
        tools?: Chat.Completions.Request.Tool[] | null;
        usage?: Chat.Completions.Request.Usage | null;
        choices: Choice[];
      }

      export interface ChatCompletionCreateParamsStreaming
        extends ChatCompletionCreateParamsBase {
        stream: true;
      }

      export interface ChatCompletionCreateParamsNonStreaming
        extends ChatCompletionCreateParamsBase {
        stream?: false | null;
      }

      export type ChatCompletionCreateParams =
        | ChatCompletionCreateParamsStreaming
        | ChatCompletionCreateParamsNonStreaming;

      export type Model = string | ScoreModelBase;

      export type Choice =
        | string
        | Chat.Completions.Response.Unary.Message
        | Choice.ChatCompletion
        | Choice.ScoreCompletion
        | Choice.MultichatCompletion;

      export namespace Choice {
        export interface ChatCompletion {
          type: "chat_completion";
          id: string;
          choice_index: number;
        }

        export interface ScoreCompletion {
          type: "score_completion";
          id: string;
          choice_index: number;
        }

        export interface MultichatCompletion {
          type: "multichat_completion";
          id: string;
          choice_index: number;
        }
      }
    }

    export namespace Response {
      export namespace Streaming {
        export interface ChatCompletionChunk {
          id: string;
          choices: Choice[];
          created: number;
          model: string;
          object: "chat.completion.chunk";
          usage?: Chat.Completions.Response.Usage;
          weight_data?: WeightData;
        }

        export namespace ChatCompletionChunk {
          export function merged(
            a: ChatCompletionChunk,
            b: ChatCompletionChunk
          ): [ChatCompletionChunk, boolean] {
            const id = a.id;
            const [choices, choicesChanged] = Choice.mergedList(
              a.choices,
              b.choices
            );
            const created = a.created;
            const model = a.model;
            const object = a.object;
            const [usage, usageChanged] = merge(
              a.usage,
              b.usage,
              Chat.Completions.Response.Usage.merged
            );
            const [weight_data, weight_dataChanged] = merge(
              a.weight_data,
              b.weight_data
            );
            if (choicesChanged || usageChanged || weight_dataChanged) {
              return [
                {
                  id,
                  choices,
                  created,
                  model,
                  object,
                  ...(usage !== undefined ? { usage } : {}),
                  ...(weight_data !== undefined ? { weight_data } : {}),
                },
                true,
              ];
            } else {
              return [a, false];
            }
          }
        }

        export interface Choice {
          delta: Delta;
          finish_reason: Chat.Completions.Response.FinishReason | null;
          index: number;
          logprobs?: Chat.Completions.Response.Logprobs;
          weight?: number;
          confidence?: number;
          error?: ObjectiveAIError;
          model?: string;
          model_index?: number | null;
          completion_metadata?: CompletionMetadata;
        }

        export namespace Choice {
          export function merged(a: Choice, b: Choice): [Choice, boolean] {
            const [delta, deltaChanged] = merge(a.delta, b.delta, Delta.merged);
            const [finish_reason, finish_reasonChanged] = merge(
              a.finish_reason,
              b.finish_reason
            );
            const index = a.index;
            const [logprobs, logprobsChanged] = merge(
              a.logprobs,
              b.logprobs,
              Chat.Completions.Response.Logprobs.merged
            );
            const [weight, weightChanged] = merge(a.weight, b.weight);
            const [confidence, confidenceChanged] = merge(
              a.confidence,
              b.confidence
            );
            const [error, errorChanged] = merge(a.error, b.error);
            const [model, modelChanged] = merge(a.model, b.model);
            const [model_index, model_indexChanged] = merge(
              a.model_index,
              b.model_index
            );
            const [completion_metadata, completion_metadataChanged] = merge(
              a.completion_metadata,
              b.completion_metadata,
              CompletionMetadata.merged
            );
            if (
              deltaChanged ||
              finish_reasonChanged ||
              logprobsChanged ||
              weightChanged ||
              confidenceChanged ||
              errorChanged ||
              modelChanged ||
              model_indexChanged ||
              completion_metadataChanged
            ) {
              return [
                {
                  delta,
                  finish_reason,
                  index,
                  ...(logprobs !== undefined ? { logprobs } : {}),
                  ...(weight !== undefined ? { weight } : {}),
                  ...(confidence !== undefined ? { confidence } : {}),
                  ...(error !== undefined ? { error } : {}),
                  ...(model !== undefined ? { model } : {}),
                  ...(model_index !== undefined ? { model_index } : {}),
                  ...(completion_metadata !== undefined
                    ? { completion_metadata }
                    : {}),
                },
                true,
              ];
            } else {
              return [a, false];
            }
          }
          export function mergedList(
            a: Choice[],
            b: Choice[]
          ): [Choice[], boolean] {
            let merged: Choice[] | undefined = undefined;
            for (const choice of b) {
              const existingIndex = a.findIndex(
                ({ index }) => index === choice.index
              );
              if (existingIndex === -1) {
                if (merged === undefined) {
                  merged = [...a, choice];
                } else {
                  merged.push(choice);
                }
              } else {
                const [mergedChoice, choiceChanged] = Choice.merged(
                  a[existingIndex],
                  choice
                );
                if (choiceChanged) {
                  if (merged === undefined) {
                    merged = [...a];
                  }
                  merged[existingIndex] = mergedChoice;
                }
              }
            }
            return merged ? [merged, true] : [a, false];
          }
        }

        export interface Delta
          extends Chat.Completions.Response.Streaming.Delta {
          vote?: number[];
        }

        export namespace Delta {
          export function merged(a: Delta, b: Delta): [Delta, boolean] {
            const [content, contentChanged] = merge(
              a.content,
              b.content,
              mergedString
            );
            const [refusal, refusalChanged] = merge(
              a.refusal,
              b.refusal,
              mergedString
            );
            const [role, roleChanged] = merge(a.role, b.role);
            const [tool_calls, tool_callsChanged] = merge(
              a.tool_calls,
              b.tool_calls,
              Chat.Completions.Response.Streaming.ToolCall.mergedList
            );
            const [reasoning, reasoningChanged] = merge(
              a.reasoning,
              b.reasoning,
              mergedString
            );
            const [images, imagesChanged] = merge(
              a.images,
              b.images,
              Chat.Completions.Response.Image.mergedList
            );
            const [vote, voteChanged] = merge(a.vote, b.vote);
            if (
              contentChanged ||
              reasoningChanged ||
              refusalChanged ||
              roleChanged ||
              tool_callsChanged ||
              imagesChanged ||
              voteChanged
            ) {
              return [
                {
                  ...(content !== undefined ? { content } : {}),
                  ...(reasoning !== undefined ? { reasoning } : {}),
                  ...(refusal !== undefined ? { refusal } : {}),
                  ...(role !== undefined ? { role } : {}),
                  ...(tool_calls !== undefined ? { tool_calls } : {}),
                  ...(images !== undefined ? { images } : {}),
                  ...(vote !== undefined ? { vote } : {}),
                },
                true,
              ];
            } else {
              return [a, false];
            }
          }
        }
      }

      export namespace Unary {
        export interface ChatCompletion {
          id: string;
          choices: Choice[];
          created: number;
          model: string;
          object: "chat.completion";
          usage?: Chat.Completions.Response.Usage;
          weight_data: WeightData | null;
        }

        export interface Choice {
          message: Message;
          finish_reason: Chat.Completions.Response.FinishReason;
          index: number;
          logprobs: Chat.Completions.Response.Logprobs | null;
          weight: number | null;
          confidence: number | null;
          error: ObjectiveAIError | null;
          model: string | null;
          model_index: number | null;
          completion_metadata: CompletionMetadata | null;
        }

        export interface Message
          extends Chat.Completions.Response.Unary.Message {
          vote: number[] | null;
        }
      }

      export type WeightData = WeightData.Static | WeightData.TrainingTable;

      export namespace WeightData {
        export interface Static {
          type: "static";
        }

        export interface TrainingTable {
          type: "training_table";
          embeddings_response: EmbeddingsResponse;
        }
      }

      export interface EmbeddingsResponse {
        data: EmbeddingsResponse.Embedding[];
        model: string;
        object: "list";
        usage?: Chat.Completions.Response.Usage;
      }

      export namespace EmbeddingsResponse {
        export interface Embedding {
          embedding: number[];
          index: number;
          object: "embedding";
        }
      }

      export interface CompletionMetadata {
        id: string;
        created: number;
        model: string;
        service_tier?: Chat.Completions.Response.ServiceTier;
        system_fingerprint?: string;
        usage?: Chat.Completions.Response.Usage;
        provider?: string;
      }

      export namespace CompletionMetadata {
        export function merged(
          a: CompletionMetadata,
          b: CompletionMetadata
        ): [CompletionMetadata, boolean] {
          const id = a.id;
          const created = a.created;
          const model = a.model;
          const [service_tier, service_tierChanged] = merge(
            a.service_tier,
            b.service_tier
          );
          const [system_fingerprint, system_fingerprintChanged] = merge(
            a.system_fingerprint,
            b.system_fingerprint
          );
          const [usage, usageChanged] = merge(
            a.usage,
            b.usage,
            Chat.Completions.Response.Usage.merged
          );
          if (
            service_tierChanged ||
            system_fingerprintChanged ||
            usageChanged
          ) {
            return [
              {
                id,
                created,
                model,
                ...(service_tier !== undefined ? { service_tier } : {}),
                ...(system_fingerprint !== undefined
                  ? { system_fingerprint }
                  : {}),
                ...(usage !== undefined ? { usage } : {}),
              },
              true,
            ];
          } else {
            return [a, false];
          }
        }
      }
    }

    export async function list(
      openai: OpenAI,
      listOptions?: Chat.Completions.ListOptions,
      options?: OpenAI.RequestOptions
    ): Promise<{ data: Chat.Completions.ListItem[] }> {
      const response = await openai.get("/score/completions", {
        query: listOptions,
        ...options,
      });
      return response as { data: Chat.Completions.ListItem[] };
    }

    export async function publish(
      openai: OpenAI,
      id: string,
      options?: OpenAI.RequestOptions
    ): Promise<void> {
      await openai.post(`/score/completions/${id}/publish`, options);
    }

    export async function retrieve(
      openai: OpenAI,
      id: string,
      options?: OpenAI.RequestOptions
    ): Promise<{
      request: Request.ChatCompletionCreateParams;
      response: Response.Unary.ChatCompletion;
      correct_vote: number[] | null;
      correct_vote_modified: string | null; // RFC 3339 timestamp
    }> {
      const response = await openai.get(`/score/completions/${id}`, options);
      return response as {
        request: Request.ChatCompletionCreateParams;
        response: Response.Unary.ChatCompletion;
        correct_vote: number[] | null;
        correct_vote_modified: string | null; // RFC 3339 timestamp
      };
    }

    export async function trainingTableAdd(
      openai: OpenAI,
      id: string,
      correctVote: number[],
      options?: OpenAI.RequestOptions
    ): Promise<void> {
      await openai.post(`/score/completions/${id}/training_table`, {
        body: { correct_vote: correctVote },
        ...options,
      });
    }

    export async function trainingTableDelete(
      openai: OpenAI,
      id: string,
      options?: OpenAI.RequestOptions
    ): Promise<void> {
      await openai.delete(`/score/completions/${id}/training_table`, options);
    }

    export async function create(
      openai: OpenAI,
      body: Request.ChatCompletionCreateParamsStreaming,
      options?: OpenAI.RequestOptions
    ): Promise<
      Stream<Response.Streaming.ChatCompletionChunk | ObjectiveAIError>
    >;
    export async function create(
      openai: OpenAI,
      body: Request.ChatCompletionCreateParamsNonStreaming,
      options?: OpenAI.RequestOptions
    ): Promise<Response.Unary.ChatCompletion>;
    export async function create(
      openai: OpenAI,
      body: Request.ChatCompletionCreateParams,
      options?: OpenAI.RequestOptions
    ): Promise<
      | Stream<Response.Streaming.ChatCompletionChunk | ObjectiveAIError>
      | Response.Unary.ChatCompletion
    > {
      const response = await openai.post("/score/completions", {
        body,
        stream: body.stream ?? false,
        ...options,
      });
      return response as unknown as
        | Stream<Response.Streaming.ChatCompletionChunk | ObjectiveAIError>
        | Response.Unary.ChatCompletion;
    }
  }
}

export namespace Multichat {
  export namespace Completions {
    export namespace Request {
      export interface ChatCompletionCreateParamsBase {
        messages: Chat.Completions.Request.Message[];
        model: Model;
        seed?: number | null;
        service_tier?: Chat.Completions.Request.ServiceTier | null;
        stream_options?: Chat.Completions.Request.StreamOptions | null;
        tools?: Chat.Completions.Request.Tool[] | null;
        usage?: Chat.Completions.Request.Usage | null;
        response_format?: Chat.Completions.Request.ResponseFormat | null;
      }

      export interface ChatCompletionCreateParamsStreaming
        extends ChatCompletionCreateParamsBase {
        stream: true;
      }

      export interface ChatCompletionCreateParamsNonStreaming
        extends ChatCompletionCreateParamsBase {
        stream?: false | null;
      }

      export type ChatCompletionCreateParams =
        | ChatCompletionCreateParamsStreaming
        | ChatCompletionCreateParamsNonStreaming;

      export type Model = string | MultichatModelBase;
    }

    export namespace Response {
      export namespace Streaming {
        export interface ChatCompletionChunk {
          id: string;
          choices: Choice[];
          created: number;
          model: string;
          object: "chat.completion.chunk";
          usage?: Chat.Completions.Response.Usage;
        }

        export namespace ChatCompletionChunk {
          export function merged(
            a: ChatCompletionChunk,
            b: ChatCompletionChunk
          ): [ChatCompletionChunk, boolean] {
            const id = a.id;
            const [choices, choicesChanged] = Choice.mergedList(
              a.choices,
              b.choices
            );
            const created = a.created;
            const model = a.model;
            const object = a.object;
            const [usage, usageChanged] = merge(
              a.usage,
              b.usage,
              Chat.Completions.Response.Usage.merged
            );
            if (choicesChanged || usageChanged) {
              return [
                {
                  id,
                  choices,
                  created,
                  model,
                  object,
                  ...(usage !== undefined ? { usage } : {}),
                },
                true,
              ];
            } else {
              return [a, false];
            }
          }
        }

        export interface Choice {
          delta: Chat.Completions.Response.Streaming.Delta;
          finish_reason: Chat.Completions.Response.FinishReason | null;
          index: number;
          logprobs?: Chat.Completions.Response.Logprobs;
          error?: ObjectiveAIError;
          model?: string;
          model_index?: number | null;
          completion_metadata?: Score.Completions.Response.CompletionMetadata;
        }

        export namespace Choice {
          export function merged(a: Choice, b: Choice): [Choice, boolean] {
            const [delta, deltaChanged] = merge(
              a.delta,
              b.delta,
              Chat.Completions.Response.Streaming.Delta.merged
            );
            const [finish_reason, finish_reasonChanged] = merge(
              a.finish_reason,
              b.finish_reason
            );
            const index = a.index;
            const [logprobs, logprobsChanged] = merge(
              a.logprobs,
              b.logprobs,
              Chat.Completions.Response.Logprobs.merged
            );
            const [error, errorChanged] = merge(a.error, b.error);
            const [model, modelChanged] = merge(a.model, b.model);
            const [model_index, model_indexChanged] = merge(
              a.model_index,
              b.model_index
            );
            const [completion_metadata, completion_metadataChanged] = merge(
              a.completion_metadata,
              b.completion_metadata,
              Score.Completions.Response.CompletionMetadata.merged
            );
            if (
              deltaChanged ||
              finish_reasonChanged ||
              logprobsChanged ||
              errorChanged ||
              modelChanged ||
              model_indexChanged ||
              completion_metadataChanged
            ) {
              return [
                {
                  delta,
                  finish_reason,
                  index,
                  ...(logprobs !== undefined ? { logprobs } : {}),
                  ...(error !== undefined ? { error } : {}),
                  ...(model !== undefined ? { model } : {}),
                  ...(model_index !== undefined ? { model_index } : {}),
                  ...(completion_metadata !== undefined
                    ? { completion_metadata }
                    : {}),
                },
                true,
              ];
            } else {
              return [a, false];
            }
          }
          export function mergedList(
            a: Choice[],
            b: Choice[]
          ): [Choice[], boolean] {
            let merged: Choice[] | undefined = undefined;
            for (const choice of b) {
              const existingIndex = a.findIndex(
                ({ index }) => index === choice.index
              );
              if (existingIndex === -1) {
                if (merged === undefined) {
                  merged = [...a, choice];
                } else {
                  merged.push(choice);
                }
              } else {
                const [mergedChoice, choiceChanged] = Choice.merged(
                  a[existingIndex],
                  choice
                );
                if (choiceChanged) {
                  if (merged === undefined) {
                    merged = [...a];
                  }
                  merged[existingIndex] = mergedChoice;
                }
              }
            }
            return merged ? [merged, true] : [a, false];
          }
        }
      }

      export namespace Unary {
        export interface ChatCompletion {
          id: string;
          choices: Choice[];
          created: number;
          model: string;
          object: "chat.completion";
          usage?: Chat.Completions.Response.Usage;
        }

        export interface Choice {
          message: Chat.Completions.Response.Unary.Message;
          finish_reason: Chat.Completions.Response.FinishReason;
          index: number;
          logprobs: Chat.Completions.Response.Logprobs | null;
          error: ObjectiveAIError | null;
          model: string | null;
          model_index: number | null;
          completion_metadata: Score.Completions.Response.CompletionMetadata | null;
        }
      }
    }

    export async function list(
      openai: OpenAI,
      listOptions?: Chat.Completions.ListOptions,
      options?: OpenAI.RequestOptions
    ): Promise<{ data: Chat.Completions.ListItem[] }> {
      const response = await openai.get("/multichat/completions", {
        query: listOptions,
        ...options,
      });
      return response as { data: Chat.Completions.ListItem[] };
    }

    export async function publish(
      openai: OpenAI,
      id: string,
      options?: OpenAI.RequestOptions
    ): Promise<void> {
      await openai.post(`/multichat/completions/${id}/publish`, options);
    }

    export async function retrieve(
      openai: OpenAI,
      id: string,
      options?: OpenAI.RequestOptions
    ): Promise<{
      request: Request.ChatCompletionCreateParams;
      response: Response.Unary.ChatCompletion;
    }> {
      const response = await openai.get(
        `/multichat/completions/${id}`,
        options
      );
      return response as {
        request: Request.ChatCompletionCreateParams;
        response: Response.Unary.ChatCompletion;
        correct_vote: number[] | null;
        correct_vote_modified: string | null; // RFC 3339 timestamp
      };
    }

    export async function create(
      openai: OpenAI,
      body: Request.ChatCompletionCreateParamsStreaming,
      options?: OpenAI.RequestOptions
    ): Promise<
      Stream<Response.Streaming.ChatCompletionChunk | ObjectiveAIError>
    >;
    export async function create(
      openai: OpenAI,
      body: Request.ChatCompletionCreateParamsNonStreaming,
      options?: OpenAI.RequestOptions
    ): Promise<Response.Unary.ChatCompletion>;
    export async function create(
      openai: OpenAI,
      body: Request.ChatCompletionCreateParams,
      options?: OpenAI.RequestOptions
    ): Promise<
      | Stream<Response.Streaming.ChatCompletionChunk | ObjectiveAIError>
      | Response.Unary.ChatCompletion
    > {
      const response = await openai.post("/multichat/completions", {
        body,
        stream: body.stream ?? false,
        ...options,
      });
      return response as unknown as
        | Stream<Response.Streaming.ChatCompletionChunk | ObjectiveAIError>
        | Response.Unary.ChatCompletion;
    }
  }
}

export namespace Conversation {
  export namespace Completions {
    export interface ListItem extends Chat.Completions.ListItem {
      type: "score" | "multichat";
    }

    export async function list(
      openai: OpenAI,
      listOptions?: Chat.Completions.ListOptions,
      options?: OpenAI.RequestOptions
    ): Promise<{ data: ListItem[] }> {
      const response = await openai.get("/conversation/completions", {
        query: listOptions,
        ...options,
      });
      return response as { data: ListItem[] };
    }
  }
}

export namespace Functions {
  export interface Expression {
    $jmespath: string;
  }

  export type WithExpression<T> = T | Expression;

  export type InputProperty =
    | InputProperty.Array
    | InputProperty.String
    | InputProperty.Number
    | InputProperty.Integer
    | InputProperty.Boolean;

  export namespace InputProperty {
    export interface Array {
      type: "array";
      description?: string | null;
      minItems?: number | null;
      items: Array.ItemsProperty | Record<string, Array.ItemsProperty>;
    }

    export namespace Array {
      export type ItemsProperty =
        | InputProperty.String
        | InputProperty.Number
        | InputProperty.Integer
        | InputProperty.Boolean;
    }

    export interface String {
      type: "string";
      description?: string | null;
      required?: boolean | null;
      enum?: string[] | null;
    }

    export interface Number {
      type: "number";
      description?: string | null;
      required?: boolean | null;
    }

    export interface Integer {
      type: "integer";
      description?: string | null;
      required?: boolean | null;
    }

    export interface Boolean {
      type: "boolean";
      description?: string | null;
      required?: boolean | null;
    }
  }

  export type OutputProperty =
    | OutputProperty.Object
    | InputProperty.Array
    | InputProperty.String
    | InputProperty.Number
    | InputProperty.Integer
    | InputProperty.Boolean;

  export namespace OutputProperty {
    export interface Object {
      type: "object";
      required?: boolean | null;
      properties: Record<string, OutputProperty>;
    }

    export interface Array {
      type: "array";
      required?: boolean | null;
      items: OutputProperty;
    }

    export interface String {
      type: "string";
      required?: boolean | null;
    }

    export interface Number {
      type: "number";
      required?: boolean | null;
    }

    export interface Integer {
      type: "integer";
      required?: boolean | null;
    }

    export interface Boolean {
      type: "boolean";
      required?: boolean | null;
    }
  }

  export type Step = Step.Score | Step.Multichat;

  export namespace Step {
    export type Message =
      | Message.Developer
      | Message.System
      | Message.User
      | Message.Assistant
      | Message.Tool
      | Message.ChatCompletion
      | Message.ScoreCompletion
      | Message.MultichatCompletion;

    export namespace Message {
      export interface Developer {
        role: Chat.Completions.Request.Message.Developer["role"];
        content: WithExpression<SimpleContent>;
        name?: WithExpression<
          Chat.Completions.Request.Message.Developer["name"]
        >;
      }

      export interface System {
        role: Chat.Completions.Request.Message.System["role"];
        content: WithExpression<SimpleContent>;
        name?: WithExpression<Chat.Completions.Request.Message.System["name"]>;
      }

      export interface User {
        role: Chat.Completions.Request.Message.User["role"];
        content: WithExpression<RichContent>;
        name?: WithExpression<Chat.Completions.Request.Message.User["name"]>;
      }

      export interface Assistant {
        role: Chat.Completions.Request.Message.Assistant["role"];
        content?: WithExpression<RichContent> | null;
        name?: WithExpression<
          Chat.Completions.Request.Message.Assistant["name"]
        >;
        refusal?: WithExpression<
          Chat.Completions.Request.Message.Assistant["refusal"]
        >;
        tool_calls?: WithExpression<
          WithExpression<Assistant.ToolCall>[]
        > | null;
      }

      export namespace Assistant {
        export type ToolCall = ToolCall.Function;

        export namespace ToolCall {
          export interface Function {
            id: WithExpression<
              Chat.Completions.Request.Message.Assistant.ToolCall.Function["id"]
            >;
            function: WithExpression<Function.Definition>;
            type: Chat.Completions.Request.Message.Assistant.ToolCall.Function["type"];
          }

          export namespace Function {
            export interface Definition {
              name: WithExpression<
                Chat.Completions.Request.Message.Assistant.ToolCall.Function.Definition["name"]
              >;
              arguments: WithExpression<
                Chat.Completions.Request.Message.Assistant.ToolCall.Function.Definition["arguments"]
              >;
            }
          }
        }
      }

      export interface Tool {
        role: Chat.Completions.Request.Message.Tool["role"];
        content: WithExpression<RichContent>;
        tool_call_id: WithExpression<
          Chat.Completions.Request.Message.Tool["tool_call_id"]
        >;
      }

      export interface ChatCompletion {
        role: Chat.Completions.Request.Message.ChatCompletion["role"];
        id: WithExpression<
          Chat.Completions.Request.Message.ChatCompletion["id"]
        >;
        choice_index: WithExpression<
          Chat.Completions.Request.Message.ChatCompletion["choice_index"]
        >;
        name?: WithExpression<
          Chat.Completions.Request.Message.ChatCompletion["name"]
        >;
      }

      export interface ScoreCompletion {
        role: Chat.Completions.Request.Message.ScoreCompletion["role"];
        id: WithExpression<
          Chat.Completions.Request.Message.ScoreCompletion["id"]
        >;
        choice_index: WithExpression<
          Chat.Completions.Request.Message.ScoreCompletion["choice_index"]
        >;
        name?: WithExpression<
          Chat.Completions.Request.Message.ScoreCompletion["name"]
        >;
      }

      export interface MultichatCompletion {
        role: Chat.Completions.Request.Message.MultichatCompletion["role"];
        id: WithExpression<
          Chat.Completions.Request.Message.MultichatCompletion["id"]
        >;
        choice_index: WithExpression<
          Chat.Completions.Request.Message.MultichatCompletion["choice_index"]
        >;
        name?: WithExpression<
          Chat.Completions.Request.Message.MultichatCompletion["name"]
        >;
      }

      export type SimpleContent =
        | string
        | WithExpression<Chat.Completions.Request.Message.SimpleContentPart>[];

      export type RichContent =
        | string
        | WithExpression<Chat.Completions.Request.Message.RichContentPart>[];
    }

    export type ResponseFormat =
      | ResponseFormat.Text
      | ResponseFormat.JsonObject
      | ResponseFormat.JsonSchema;

    export namespace ResponseFormat {
      export interface Text {
        type: "text";
      }

      export interface JsonObject {
        type: "json_object";
      }

      export interface JsonSchema {
        type: "json_schema";
        json_schema: WithExpression<JsonSchema.JsonSchema>;
      }

      export namespace JsonSchema {
        export interface JsonSchema {
          name: WithExpression<
            Chat.Completions.Request.ResponseFormat.JsonSchema.JsonSchema["name"]
          >;
          description?: WithExpression<
            Chat.Completions.Request.ResponseFormat.JsonSchema.JsonSchema["description"]
          >;
          schema?: WithExpression<
            Chat.Completions.Request.ResponseFormat.JsonSchema.JsonSchema["schema"]
          >;
          strict?: WithExpression<
            Chat.Completions.Request.ResponseFormat.JsonSchema.JsonSchema["strict"]
          >;
        }
      }
    }

    export interface Multichat {
      type: "multichat";
      messages: WithExpression<WithExpression<Message>[]>;
      model: WithExpression<
        Multichat.Completions.Request.ChatCompletionCreateParams["model"]
      >;
      response_format?: WithExpression<ResponseFormat> | null;
    }

    export interface Score {
      type: "score";
      messages: WithExpression<WithExpression<Message>[]>;
      model: WithExpression<
        Score.Completions.Request.ChatCompletionCreateParams["model"]
      >;
      choices: WithExpression<
        WithExpression<Score.Completions.Request.Choice>[]
      >;
    }
  }

  export namespace Request {
    export interface ExecuteByIdRequestBase {
      arguments?: JsonValue;
      retry_token?: string;
      seed?: Chat.Completions.Request.ChatCompletionCreateParamsBase["seed"];
      service_tier?: Chat.Completions.Request.ChatCompletionCreateParamsBase["service_tier"];
      stream_options?: Chat.Completions.Request.ChatCompletionCreateParamsBase["stream_options"];
      usage?: Chat.Completions.Request.ChatCompletionCreateParamsBase["usage"];
    }

    export interface ExecuteByIdRequestStreaming
      extends ExecuteByIdRequestBase {
      stream: true;
    }

    export interface ExecuteByIdRequestNonStreaming
      extends ExecuteByIdRequestBase {
      stream?: false | null;
    }

    export type ExecuteByIdRequest =
      | ExecuteByIdRequestStreaming
      | ExecuteByIdRequestNonStreaming;

    export interface ExecuteByDefinitionRequestBase
      extends ExecuteByIdRequestBase {
      steps: WithExpression<WithExpression<Step>[]>;
      output: Expression | Record<string, Expression>;
      publish?: {
        id: string;
        description: string;
        input: Record<string, InputProperty>;
        output: OutputProperty;
      } | null;
    }

    export interface ExecuteByDefinitionRequestStreaming
      extends ExecuteByDefinitionRequestBase {
      stream: true;
    }

    export interface ExecuteByDefinitionRequestNonStreaming
      extends ExecuteByDefinitionRequestBase {
      stream?: false | null;
    }

    export type ExecuteByDefinitionRequest =
      | ExecuteByDefinitionRequestStreaming
      | ExecuteByDefinitionRequestNonStreaming;
  }

  export namespace Response {
    export namespace Streaming {
      export interface ExecuteResponseChunk {
        completions: CompletionChunk[];
        output?: JsonValue;
        retry_token?: string;
        error?: ObjectiveAIError;
        function_published?: boolean;
      }

      export type CompletionChunk =
        | CompletionChunk.ScoreCompletionChunk
        | CompletionChunk.MultichatCompletionChunk;

      export namespace CompletionChunk {
        export interface ScoreCompletionChunk
          extends Score.Completions.Response.Streaming.ChatCompletionChunk {
          type: "score";
          index: number;
        }

        export namespace ScoreCompletionChunk {
          export function merged(
            a: ScoreCompletionChunk,
            b: ScoreCompletionChunk
          ): [ScoreCompletionChunk, boolean] {
            const [base, baseChanged] =
              Score.Completions.Response.Streaming.ChatCompletionChunk.merged(
                a,
                b
              );
            return baseChanged
              ? [{ type: a.type, ...base, index: a.index }, true]
              : [a, false];
          }
        }

        export interface MultichatCompletionChunk
          extends Multichat.Completions.Response.Streaming.ChatCompletionChunk {
          type: "multichat";
          index: number;
        }

        export namespace MultichatCompletionChunk {
          export function merged(
            a: MultichatCompletionChunk,
            b: MultichatCompletionChunk
          ): [MultichatCompletionChunk, boolean] {
            const [base, baseChanged] =
              Multichat.Completions.Response.Streaming.ChatCompletionChunk.merged(
                a,
                b
              );
            return baseChanged
              ? [{ type: a.type, ...base, index: a.index }, true]
              : [a, false];
          }
        }

        export function mergedList(
          a: CompletionChunk[],
          b: CompletionChunk[]
        ): [CompletionChunk[], boolean] {
          let merged: CompletionChunk[] | undefined = undefined;
          for (const chunk of b) {
            const existingIndex = a.findIndex(
              (c) => c.index === chunk.index && c.type === chunk.type
            );
            if (existingIndex === -1) {
              if (merged === undefined) {
                merged = [...a, chunk];
              } else {
                merged.push(chunk);
              }
            } else if (chunk.type === "score") {
              const [mergedChunk, chunkChanged] = ScoreCompletionChunk.merged(
                a[existingIndex] as ScoreCompletionChunk,
                chunk
              );
              if (chunkChanged) {
                if (merged === undefined) {
                  merged = [...a];
                }
                merged[existingIndex] = mergedChunk;
              }
            } else if (chunk.type === "multichat") {
              const [mergedChunk, chunkChanged] =
                MultichatCompletionChunk.merged(
                  a[existingIndex] as MultichatCompletionChunk,
                  chunk
                );
              if (chunkChanged) {
                if (merged === undefined) {
                  merged = [...a];
                }
                merged[existingIndex] = mergedChunk;
              }
            }
          }
          return merged ? [merged, true] : [a, false];
        }
      }

      export function merged(
        a: ExecuteResponseChunk,
        b: ExecuteResponseChunk
      ): [ExecuteResponseChunk, boolean] {
        const [completions, completionsChanged] = CompletionChunk.mergedList(
          a.completions,
          b.completions
        );
        const [output, outputChanged] = merge(a.output, b.output);
        const [retry_token, retry_tokenChanged] = merge(
          a.retry_token,
          b.retry_token
        );
        const [error, errorChanged] = merge(a.error, b.error);
        const [function_published, function_publishedChanged] = merge(
          a.function_published,
          b.function_published
        );
        if (
          completionsChanged ||
          outputChanged ||
          retry_tokenChanged ||
          errorChanged ||
          function_publishedChanged
        ) {
          return [
            {
              completions,
              ...(output !== undefined ? { output } : {}),
              ...(retry_token !== undefined ? { retry_token } : {}),
              ...(error !== undefined ? { error } : {}),
              ...(function_published !== undefined
                ? { function_published }
                : {}),
            },
            true,
          ];
        } else {
          return [a, false];
        }
      }
    }

    export namespace Unary {
      export interface ExecuteResponse {
        completions: Completion[];
        output: JsonValue;
        retry_token: string | null;
        error: ObjectiveAIError | null;
        function_published?: boolean;
      }

      export type Completion = Completion.Score | Completion.Multichat;

      export namespace Completion {
        export interface Score
          extends Score.Completions.Response.Unary.ChatCompletion {
          type: "score";
          index: number;
        }

        export interface Multichat
          extends Multichat.Completions.Response.Unary.ChatCompletion {
          type: "multichat";
          index: number;
        }
      }
    }
  }

  export interface FunctionHeader {
    author: string;
    id: string;
    version: number;
    description: string;
  }

  export interface Function extends FunctionHeader {
    input_schema: Record<string, InputProperty>;
    output_schema: OutputProperty;
    steps: WithExpression<WithExpression<Step>[]>;
    output: Expression | Record<string, Expression>;
  }

  export interface FunctionWithMetadata extends Function {
    created: string; // RFC 3339 timestamp
    requests: number;
    completion_tokens: number;
    prompt_tokens: number;
    total_cost: number;
  }

  export async function list(
    openai: OpenAI,
    listOptions?: Models.ListOptions,
    options?: OpenAI.RequestOptions
  ): Promise<{ data: FunctionHeader[] }> {
    const response = await openai.get("/functions", {
      query: listOptions,
      ...options,
    });
    return response as unknown as { data: FunctionHeader[] };
  }

  export async function count(
    openai: OpenAI,
    options?: OpenAI.RequestOptions
  ): Promise<{ data: number }> {
    const response = await openai.get("/functions/count", options);
    return response as unknown as { data: number };
  }

  export async function retrieve(
    openai: OpenAI,
    author: string,
    id: string,
    version?: number | null,
    retrieveOptions?: Models.RetrieveOptionsWithoutMetadata,
    options?: OpenAI.RequestOptions
  ): Promise<Function>;
  export async function retrieve(
    openai: OpenAI,
    author: string,
    id: string,
    version?: number | null,
    retrieveOptions?: Models.RetrieveOptionsWithMetadata,
    options?: OpenAI.RequestOptions
  ): Promise<FunctionWithMetadata>;
  export async function retrieve(
    openai: OpenAI,
    author: string,
    id: string,
    version?: number | null,
    retrieveOptions?: Models.RetrieveOptions,
    options?: OpenAI.RequestOptions
  ): Promise<Function | FunctionWithMetadata> {
    const url =
      version !== null && version !== undefined
        ? `/functions/${author}/${id}/${version}`
        : `/functions/${author}/${id}`;
    const response = await openai.get(url, {
      query: retrieveOptions,
      ...options,
    });
    return response as unknown as Function | FunctionWithMetadata;
  }

  export async function executeById(
    openai: OpenAI,
    author: string,
    id: string,
    version: number | null | undefined,
    body: Request.ExecuteByIdRequestStreaming,
    options?: OpenAI.RequestOptions
  ): Promise<Stream<Response.Streaming.ExecuteResponseChunk>>;
  export async function executeById(
    openai: OpenAI,
    author: string,
    id: string,
    version: number | null | undefined,
    body: Request.ExecuteByIdRequestNonStreaming,
    options?: OpenAI.RequestOptions
  ): Promise<Response.Unary.ExecuteResponse>;
  export async function executeById(
    openai: OpenAI,
    author: string,
    id: string,
    version: number | null | undefined,
    body: Request.ExecuteByIdRequest,
    options?: OpenAI.RequestOptions
  ): Promise<
    | Stream<Response.Streaming.ExecuteResponseChunk>
    | Response.Unary.ExecuteResponse
  > {
    const url =
      version !== null && version !== undefined
        ? `/functions/${author}/${id}/${version}`
        : `/functions/${author}/${id}`;
    const response = await openai.post(url, {
      body,
      stream: body.stream ?? false,
      ...options,
    });
    return response as unknown as
      | Stream<Response.Streaming.ExecuteResponseChunk>
      | Response.Unary.ExecuteResponse;
  }

  export async function executeByDefinition(
    openai: OpenAI,
    body: Request.ExecuteByDefinitionRequestStreaming,
    options?: OpenAI.RequestOptions
  ): Promise<Stream<Response.Streaming.ExecuteResponseChunk>>;
  export async function executeByDefinition(
    openai: OpenAI,
    body: Request.ExecuteByDefinitionRequestNonStreaming,
    options?: OpenAI.RequestOptions
  ): Promise<Response.Unary.ExecuteResponse>;
  export async function executeByDefinition(
    openai: OpenAI,
    body: Request.ExecuteByDefinitionRequest,
    options?: OpenAI.RequestOptions
  ): Promise<
    | Stream<Response.Streaming.ExecuteResponseChunk>
    | Response.Unary.ExecuteResponse
  > {
    const response = await openai.post("/functions", {
      body,
      stream: body.stream ?? false,
      ...options,
    });
    return response as unknown as
      | Stream<Response.Streaming.ExecuteResponseChunk>
      | Response.Unary.ExecuteResponse;
  }
}

export namespace Models {
  export interface Model {
    id: string;
    name: string;
    created: number;
    description: string;
    architecture: Architecture;
    top_provider: TopProvider;
    pricing: Pricing;
    context_length?: number;
    hugging_face_id?: string;
    per_request_limits?: Record<string, unknown>;
    supported_parameters?: string[];
  }

  export interface ModelWithProviders {
    id: string;
    name: string;
    created: number;
    description: string;
    architecture: Architecture;
    endpoints: Provider[];
  }

  export interface Architecture {
    input_modalities: string[];
    output_modalities: string[];
    tokenizer: string;
    instruct_type?: string;
  }

  export interface TopProvider {
    is_moderated: boolean;
    context_length?: number;
    max_completion_tokens?: number;
  }

  export interface Pricing {
    prompt: number;
    completion: number;
    image?: number;
    request?: number;
    input_cache_read?: number;
    input_cache_write?: number;
    web_search?: number;
    internal_reasoning?: number;
    discount?: number;
  }

  export interface Provider {
    name: string;
    tag: string;
    context_length?: number;
    pricing: Pricing;
    provider_name: string;
    supported_parameters?: string[];
    quantization?: string;
    max_completion_tokens?: number;
    max_prompt_tokens?: number;
    status?: unknown;
  }

  export interface ListOptions {
    count?: number;
    offset?: number;
    me?: boolean;
    sort?:
      | "created"
      | "tokens"
      | "completion_tokens"
      | "prompt_tokens"
      | "cost"
      | "requests";
    from?: string; // RFC 3339 timestamp
    to?: string; // RFC 3339 timestamp
  }

  export interface RetrieveOptionsBase {
    me?: boolean | null;
    from?: string | null; // RFC 3339 timestamp
    to?: string | null; // RFC 3339 timestamp
  }
  export interface RetrieveOptionsWithMetadata extends RetrieveOptionsBase {
    metadata: true;
  }
  export interface RetrieveOptionsWithoutMetadata extends RetrieveOptionsBase {
    metadata?: false | null;
  }
  export type RetrieveOptions =
    | RetrieveOptionsWithMetadata
    | RetrieveOptionsWithoutMetadata;

  export async function list(
    openai: OpenAI,
    listOptions?: ListOptions,
    options?: OpenAI.RequestOptions
  ): Promise<{ data: Model[] }> {
    const response = await openai.models.list({
      query: listOptions,
      ...options,
    });
    return response as unknown as { data: Model[] };
  }

  export async function retrieve(
    openai: OpenAI,
    model: string,
    retrieveOptions?: Models.RetrieveOptions,
    options?: OpenAI.RequestOptions
  ): Promise<{ data: ModelWithProviders }> {
    const response = await openai.models.retrieve(model, {
      query: retrieveOptions,
      ...options,
    });
    return response as unknown as { data: ModelWithProviders };
  }
}

export namespace Auth {
  export interface ApiKey {
    api_key: string;
    created: string; // RFC 3339 timestamp
    expires: string | null; // RFC 3339 timestamp
    disabled: string | null; // RFC 3339 timestamp
    name: string;
    description: string | null;
  }

  export interface ApiKeyWithCost extends ApiKey {
    cost: number;
  }

  export namespace ApiKey {
    export async function list(
      openai: OpenAI,
      options?: OpenAI.RequestOptions
    ): Promise<{ data: ApiKeyWithCost[] }> {
      const response = await openai.get("/auth/keys", options);
      return response as { data: ApiKeyWithCost[] };
    }

    export async function create(
      openai: OpenAI,
      name: string,
      expires?: Date | null,
      description?: string | null,
      options?: OpenAI.RequestOptions
    ): Promise<ApiKey> {
      const response = await openai.post("/auth/keys", {
        body: {
          name,
          expires,
          description,
        },
        ...options,
      });
      return response as ApiKey;
    }

    export async function remove(
      openai: OpenAI,
      key: string,
      options?: OpenAI.RequestOptions
    ): Promise<ApiKey> {
      const response = await openai.delete("/auth/keys", {
        body: {
          api_key: key,
        },
        ...options,
      });
      return response as ApiKey;
    }
  }

  export interface OpenRouterApiKey {
    api_key: string;
  }

  export namespace OpenRouterApiKey {
    export async function retrieve(
      openai: OpenAI,
      options?: OpenAI.RequestOptions
    ): Promise<OpenRouterApiKey> {
      const response = await openai.get("/auth/keys/openrouter", options);
      return response as OpenRouterApiKey;
    }

    export async function create(
      openai: OpenAI,
      apiKey: string,
      options?: OpenAI.RequestOptions
    ): Promise<OpenRouterApiKey> {
      const response = await openai.post("/auth/keys/openrouter", {
        body: {
          api_key: apiKey,
        },
        ...options,
      });
      return response as OpenRouterApiKey;
    }

    export async function remove(
      openai: OpenAI,
      options?: OpenAI.RequestOptions
    ): Promise<OpenRouterApiKey> {
      const response = await openai.delete("/auth/keys/openrouter", options);
      return response as OpenRouterApiKey;
    }
  }

  export interface Credits {
    credits: number;
    total_credits_purchased: number;
    total_credits_used: number;
  }

  export namespace Credits {
    export async function retrieve(
      openai: OpenAI,
      options?: OpenAI.RequestOptions
    ): Promise<Credits> {
      const response = await openai.get("/auth/credits", options);
      return response as Credits;
    }
  }

  export namespace Username {
    export async function retrieve(
      openai: OpenAI,
      options?: OpenAI.RequestOptions
    ): Promise<{ username: string | null }> {
      const response = await openai.get("/auth/username", options);
      return response as { username: string | null };
    }

    export async function set(
      openai: OpenAI,
      username: string,
      options?: OpenAI.RequestOptions
    ): Promise<{ username: string }> {
      const response = await openai.post("/auth/username", {
        body: { username },
        ...options,
      });
      return response as { username: string };
    }
  }
}

export interface Metadata {
  score_requests: number;
  score_completion_tokens: number;
  score_prompt_tokens: number;
  score_total_cost: number;
  chat_requests: number;
  chat_completion_tokens: number;
  chat_prompt_tokens: number;
  chat_total_cost: number;
  multichat_requests: number;
  multichat_completion_tokens: number;
  multichat_prompt_tokens: number;
  multichat_total_cost: number;
}

export namespace Metadata {
  export async function get(
    openai: OpenAI,
    options?: OpenAI.RequestOptions
  ): Promise<Metadata> {
    const response = await openai.get("/metadata", options);
    return response as Metadata;
  }
}

export interface ScoreLlmBase {
  model: string;
  weight: ScoreLlm.Weight;
  output_mode: ScoreLlm.OutputMode;
  synthetic_reasoning?: boolean | null;
  top_logprobs?: number | null;
  prefix_messages?: Chat.Completions.Request.Message[] | null;
  suffix_messages?: Chat.Completions.Request.Message[] | null;
  frequency_penalty?: number | null;
  logit_bias?: Record<string, number> | null;
  max_completion_tokens?: number | null;
  presence_penalty?: number | null;
  stop?: Chat.Completions.Request.Stop | null;
  temperature?: number | null;
  top_p?: number | null;
  max_tokens?: number | null;
  min_p?: number | null;
  provider?: Chat.Completions.Request.ProviderPreferences | null;
  reasoning?: Chat.Completions.Request.Reasoning | null;
  repetition_penalty?: number | null;
  top_a?: number | null;
  top_k?: number | null;
  verbosity?: Chat.Completions.Request.Verbosity | null;
  models?: string[] | null;
}

export interface ScoreLlmWithoutIndices extends ScoreLlmBase {
  id: string;
  multichat_id: string;
  training_table_id?: string;
}

export interface ScoreLlmWithoutIndicesWithMetadata
  extends ScoreLlmWithoutIndices {
  created: string; // RFC 3339 timestamp
  requests: number;
  completion_tokens: number;
  prompt_tokens: number;
  total_cost: number;
  average_confidence: number;
  conformity: number;
}

export interface ScoreLlm extends ScoreLlmWithoutIndices {
  index: number;
  multichat_index: number;
  training_table_index?: number;
}

export namespace ScoreLlm {
  export type Weight = Weight.Static | Weight.TrainingTable;

  export namespace Weight {
    export interface Static {
      type: "static";
      weight: number;
    }

    export interface TrainingTable {
      type: "training_table";
      base_weight: number;
      min_weight: number;
      max_weight: number;
    }
  }

  export type OutputMode = "instruction" | "json_schema" | "tool_call";

  export async function list(
    openai: OpenAI,
    listOptions?: Models.ListOptions,
    options?: OpenAI.RequestOptions
  ): Promise<{ data: string[] }> {
    const response = await openai.get("/score/llms", {
      query: listOptions,
      ...options,
    });
    return response as { data: string[] };
  }

  export async function count(
    openai: OpenAI,
    options?: OpenAI.RequestOptions
  ): Promise<{ data: number }> {
    const response = await openai.get("/score/llms/count", options);
    return response as { data: number };
  }

  export async function retrieve(
    openai: OpenAI,
    model: string,
    retrieveOptions?: Models.RetrieveOptionsWithoutMetadata,
    options?: OpenAI.RequestOptions
  ): Promise<ScoreLlmWithoutIndices>;
  export async function retrieve(
    openai: OpenAI,
    model: string,
    retrieveOptions: Models.RetrieveOptionsWithMetadata,
    options?: OpenAI.RequestOptions
  ): Promise<ScoreLlmWithoutIndicesWithMetadata>;
  export async function retrieve(
    openai: OpenAI,
    model: string,
    retrieveOptions?: Models.RetrieveOptions,
    options?: OpenAI.RequestOptions
  ): Promise<ScoreLlmWithoutIndices | ScoreLlmWithoutIndicesWithMetadata> {
    const response = await openai.get(`/score/llms/${model}`, {
      query: retrieveOptions,
      ...options,
    });
    return response as
      | ScoreLlmWithoutIndices
      | ScoreLlmWithoutIndicesWithMetadata;
  }

  export async function retrieveValidate(
    openai: OpenAI,
    model: ScoreLlmBase,
    retrieveOptions?: Models.RetrieveOptionsWithoutMetadata,
    options?: OpenAI.RequestOptions
  ): Promise<ScoreLlmWithoutIndices>;
  export async function retrieveValidate(
    openai: OpenAI,
    model: ScoreLlmBase,
    retrieveOptions: Models.RetrieveOptionsWithMetadata,
    options?: OpenAI.RequestOptions
  ): Promise<ScoreLlmWithoutIndicesWithMetadata>;
  export async function retrieveValidate(
    openai: OpenAI,
    model: ScoreLlmBase,
    retrieveOptions?: Models.RetrieveOptions,
    options?: OpenAI.RequestOptions
  ): Promise<ScoreLlmWithoutIndices | ScoreLlmWithoutIndicesWithMetadata> {
    const response = await openai.post("/score/llms", {
      query: retrieveOptions,
      body: model,
      ...options,
    });
    return response as
      | ScoreLlmWithoutIndices
      | ScoreLlmWithoutIndicesWithMetadata;
  }
}

export type MultichatLlmBase = ScoreLlmBase;

export type MultichatLlmWithoutIndices = ScoreLlmWithoutIndices;

export interface MultichatLlmWithoutIndicesWithMetadata
  extends MultichatLlmWithoutIndices {
  created: string; // RFC 3339 timestamp
  requests: number;
  completion_tokens: number;
  prompt_tokens: number;
  total_cost: number;
}

export type MultichatLlm = ScoreLlm;

export namespace MultichatLlm {
  export async function list(
    openai: OpenAI,
    listOptions?: Models.ListOptions,
    options?: OpenAI.RequestOptions
  ): Promise<{ data: string[] }> {
    const response = await openai.get("/multichat/llms", {
      query: listOptions,
      ...options,
    });
    return response as { data: string[] };
  }

  export async function count(
    openai: OpenAI,
    options?: OpenAI.RequestOptions
  ): Promise<{ data: number }> {
    const response = await openai.get("/multichat/llms/count", options);
    return response as { data: number };
  }

  export async function retrieve(
    openai: OpenAI,
    model: string,
    retrieveOptions?: Models.RetrieveOptionsWithoutMetadata,
    options?: OpenAI.RequestOptions
  ): Promise<ScoreLlmWithoutIndices>;
  export async function retrieve(
    openai: OpenAI,
    model: string,
    retrieveOptions: Models.RetrieveOptionsWithMetadata,
    options?: OpenAI.RequestOptions
  ): Promise<ScoreLlmWithoutIndicesWithMetadata>;
  export async function retrieve(
    openai: OpenAI,
    model: string,
    retrieveOptions?: Models.RetrieveOptions,
    options?: OpenAI.RequestOptions
  ): Promise<ScoreLlmWithoutIndices | ScoreLlmWithoutIndicesWithMetadata> {
    const response = await openai.get(`/multichat/llms/${model}`, {
      query: retrieveOptions,
      ...options,
    });
    return response as
      | ScoreLlmWithoutIndices
      | ScoreLlmWithoutIndicesWithMetadata;
  }

  export async function retrieveValidate(
    openai: OpenAI,
    model: ScoreLlmBase,
    retrieveOptions?: Models.RetrieveOptionsWithoutMetadata,
    options?: OpenAI.RequestOptions
  ): Promise<ScoreLlmWithoutIndices>;
  export async function retrieveValidate(
    openai: OpenAI,
    model: ScoreLlmBase,
    retrieveOptions: Models.RetrieveOptionsWithMetadata,
    options?: OpenAI.RequestOptions
  ): Promise<ScoreLlmWithoutIndicesWithMetadata>;
  export async function retrieveValidate(
    openai: OpenAI,
    model: ScoreLlmBase,
    retrieveOptions?: Models.RetrieveOptions,
    options?: OpenAI.RequestOptions
  ): Promise<ScoreLlmWithoutIndices | ScoreLlmWithoutIndicesWithMetadata> {
    const response = await openai.post("/multichat/llms", {
      query: retrieveOptions,
      body: model,
      ...options,
    });
    return response as
      | ScoreLlmWithoutIndices
      | ScoreLlmWithoutIndicesWithMetadata;
  }
}

export interface ScoreModelBase {
  llms: ScoreLlmBase[];
  weight: ScoreModel.Weight;
}

export interface ScoreModel {
  id: string;
  multichat_id: string;
  training_table_id?: string;
  llms: ScoreLlm[];
  weight: ScoreModel.Weight;
}

export interface ScoreModelWithMetadata extends ScoreModel {
  created: string; // RFC 3339 timestamp
  requests: number;
  completion_tokens: number;
  prompt_tokens: number;
  total_cost: number;
  average_winner_confidence: number;
  decisiveness: number;
}

export namespace ScoreModel {
  export type Weight = Weight.Static | Weight.TrainingTable;

  export namespace Weight {
    export interface Static {
      type: "static";
    }

    export interface TrainingTable {
      type: "training_table";
      embeddings: {
        model: string;
        max_tokens: number;
        provider?: Chat.Completions.Request.ProviderPreferences | null;
      };
      top: number;
    }
  }

  export async function list(
    openai: OpenAI,
    listOptions?: Models.ListOptions,
    options?: OpenAI.RequestOptions
  ): Promise<{ data: string[] }> {
    const response = await openai.get("/score/models", {
      query: listOptions,
      ...options,
    });
    return response as { data: string[] };
  }

  export async function count(
    openai: OpenAI,
    options?: OpenAI.RequestOptions
  ): Promise<{ count: number }> {
    const response = await openai.get("/score/models/count", options);
    return response as { count: number };
  }

  export async function retrieve(
    openai: OpenAI,
    model: string,
    retrieveOptions?: Models.RetrieveOptionsWithoutMetadata,
    options?: OpenAI.RequestOptions
  ): Promise<ScoreModel>;
  export async function retrieve(
    openai: OpenAI,
    model: string,
    retrieveOptions: Models.RetrieveOptionsWithMetadata,
    options?: OpenAI.RequestOptions
  ): Promise<ScoreModelWithMetadata>;
  export async function retrieve(
    openai: OpenAI,
    model: string,
    retrieveOptions?: Models.RetrieveOptions,
    options?: OpenAI.RequestOptions
  ): Promise<ScoreModel | ScoreModelWithMetadata> {
    const response = await openai.get(`/score/models/${model}`, {
      query: retrieveOptions,
      ...options,
    });
    return response as ScoreModel | ScoreModelWithMetadata;
  }

  export async function retrieveValidate(
    openai: OpenAI,
    model: ScoreModelBase,
    retrieveOptions?: Models.RetrieveOptionsWithoutMetadata,
    options?: OpenAI.RequestOptions
  ): Promise<ScoreModel>;
  export async function retrieveValidate(
    openai: OpenAI,
    model: ScoreModelBase,
    retrieveOptions: Models.RetrieveOptionsWithMetadata,
    options?: OpenAI.RequestOptions
  ): Promise<ScoreModelWithMetadata>;
  export async function retrieveValidate(
    openai: OpenAI,
    model: ScoreModelBase,
    retrieveOptions?: Models.RetrieveOptions,
    options?: OpenAI.RequestOptions
  ): Promise<ScoreModel | ScoreModelWithMetadata> {
    const response = await openai.post("/score/models", {
      query: retrieveOptions,
      body: model,
      ...options,
    });
    return response as ScoreModel | ScoreModelWithMetadata;
  }
}

export type MultichatModelBase = ScoreModelBase;

export type MultichatModel = ScoreModel;

export interface MultichatModelWithMetadata extends MultichatModel {
  created: string; // RFC 3339 timestamp
  requests: number;
  completion_tokens: number;
  prompt_tokens: number;
  total_cost: number;
}

export namespace MultichatModel {
  export async function list(
    openai: OpenAI,
    listOptions?: Models.ListOptions,
    options?: OpenAI.RequestOptions
  ): Promise<{ data: string[] }> {
    const response = await openai.get("/multichat/models", {
      query: listOptions,
      ...options,
    });
    return response as { data: string[] };
  }

  export async function count(
    openai: OpenAI,
    options?: OpenAI.RequestOptions
  ): Promise<{ count: number }> {
    const response = await openai.get("/multichat/models/count", options);
    return response as { count: number };
  }

  export async function retrieve(
    openai: OpenAI,
    model: string,
    retrieveOptions?: Models.RetrieveOptionsWithoutMetadata,
    options?: OpenAI.RequestOptions
  ): Promise<MultichatModel>;
  export async function retrieve(
    openai: OpenAI,
    model: string,
    retrieveOptions: Models.RetrieveOptionsWithMetadata,
    options?: OpenAI.RequestOptions
  ): Promise<MultichatModelWithMetadata>;
  export async function retrieve(
    openai: OpenAI,
    model: string,
    retrieveOptions?: Models.RetrieveOptions,
    options?: OpenAI.RequestOptions
  ): Promise<MultichatModel | MultichatModelWithMetadata> {
    const response = await openai.get(`/multichat/models/${model}`, {
      query: retrieveOptions,
      ...options,
    });
    return response as MultichatModel | MultichatModelWithMetadata;
  }

  export async function retrieveValidate(
    openai: OpenAI,
    model: MultichatModelBase,
    retrieveOptions?: Models.RetrieveOptionsWithoutMetadata,
    options?: OpenAI.RequestOptions
  ): Promise<MultichatModel>;
  export async function retrieveValidate(
    openai: OpenAI,
    model: MultichatModelBase,
    retrieveOptions: Models.RetrieveOptionsWithMetadata,
    options?: OpenAI.RequestOptions
  ): Promise<MultichatModelWithMetadata>;
  export async function retrieveValidate(
    openai: OpenAI,
    model: MultichatModelBase,
    retrieveOptions?: Models.RetrieveOptions,
    options?: OpenAI.RequestOptions
  ): Promise<MultichatModel | MultichatModelWithMetadata> {
    const response = await openai.post("/multichat/models", {
      query: retrieveOptions,
      body: model,
      ...options,
    });
    return response as MultichatModel | MultichatModelWithMetadata;
  }
}

export type JsonValue =
  | null
  | boolean
  | number
  | string
  | JsonValue[]
  | { [key: string]: JsonValue };

export interface ObjectiveAIError {
  code: number;
  message: JsonValue;
}

function merge<T extends {}>(
  a: T,
  b: T,
  combine?: (a: T, b: T) => [T, boolean]
): [T, boolean];
function merge<T extends {}>(
  a: T | null,
  b: T | null,
  combine?: (a: T, b: T) => [T, boolean]
): [T | null, boolean];
function merge<T extends {}>(
  a: T | undefined,
  b: T | undefined,
  combine?: (a: T, b: T) => [T, boolean]
): [T | undefined, boolean];
function merge<T extends {}>(
  a: T | null | undefined,
  b: T | null | undefined,
  combine?: (a: T, b: T) => [T, boolean]
): [T | null | undefined, boolean];
function merge<T extends {}>(
  a: T | null | undefined,
  b: T | null | undefined,
  combine?: (a: T, b: T) => [T, boolean]
): [T | null | undefined, boolean] {
  if (a !== null && a !== undefined && b !== null && b !== undefined) {
    return combine ? combine(a, b) : [a, false];
  } else if (a !== null && a !== undefined) {
    return [a, false];
  } else if (b !== null && b !== undefined) {
    return [b, true];
  } else if (a === null || b === null) {
    return [null, false];
  } else {
    return [undefined, false];
  }
}

function mergedString(a: string, b: string): [string, boolean] {
  return b === "" ? [a, false] : [a + b, true];
}
function mergedNumber(a: number, b: number): [number, boolean] {
  return b === 0 ? [a, false] : [a + b, true];
}
