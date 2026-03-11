import { z } from "zod";
import { AgentCompletionsRequestAgentSchema } from "../../../agent/completions/request/agent";

export const FunctionsExecutionsRequestReasoningSchema = z.object({
  agent: AgentCompletionsRequestAgentSchema.describe("The primary agent to use for generating reasoning summaries."),
  agents: z.array(AgentCompletionsRequestAgentSchema).nullable().describe("Fallback agents tried in order if the primary is rate-limited or errors.").optional(),
}).describe("Configuration for generating reasoning summaries during execution.\n\nWhen enabled, an LLM summarizes the execution's reasoning process.").meta({ title: "functions.executions.request.Reasoning" });
export type FunctionsExecutionsRequestReasoning = z.infer<typeof FunctionsExecutionsRequestReasoningSchema>;
