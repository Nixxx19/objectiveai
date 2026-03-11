import { z } from "zod";
import { AgentCompletionsResponseStreamingAssistantResponseChunkSchema } from "./assistantResponseChunk";
import { AgentCompletionsResponseToolResponseSchema } from "../toolResponse";

export const AgentCompletionsResponseStreamingMessageChunkSchema = z.union([AgentCompletionsResponseStreamingAssistantResponseChunkSchema, AgentCompletionsResponseToolResponseSchema]).meta({ title: "agent.completions.response.streaming.MessageChunk" });
export type AgentCompletionsResponseStreamingMessageChunk = z.infer<typeof AgentCompletionsResponseStreamingMessageChunkSchema>;
