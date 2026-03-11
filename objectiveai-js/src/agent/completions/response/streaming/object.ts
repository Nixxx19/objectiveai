import { z } from "zod";

export const AgentCompletionsResponseStreamingObjectSchema = z.union([z.literal("agent.completion.chunk").describe("A agent completion chunk object.")]).describe("The object type for streaming agent completion chunks.").meta({ title: "agent.completions.response.streaming.Object" });
export type AgentCompletionsResponseStreamingObject = z.infer<typeof AgentCompletionsResponseStreamingObjectSchema>;
