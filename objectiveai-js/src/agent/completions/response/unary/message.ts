import { z } from "zod";
import { AgentCompletionsResponseToolResponseSchema } from "../toolResponse";
import { AgentCompletionsResponseUnaryAssistantResponseSchema } from "./assistantResponse";

export const AgentCompletionsResponseUnaryMessageSchema = z.union([AgentCompletionsResponseUnaryAssistantResponseSchema, AgentCompletionsResponseToolResponseSchema]).meta({ title: "agent.completions.response.unary.Message" });
export type AgentCompletionsResponseUnaryMessage = z.infer<typeof AgentCompletionsResponseUnaryMessageSchema>;
