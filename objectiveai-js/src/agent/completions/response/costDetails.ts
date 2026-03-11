import { z } from "zod";

export const AgentCompletionsResponseCostDetailsSchema = z.object({
  upstream_inference_cost: z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()]).describe("Cost charged by the immediate upstream (e.g., OpenRouter)."),
  upstream_upstream_inference_cost: z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()]).describe("Cost charged by the upstream's upstream (e.g., the actual model provider)."),
}).describe("Detailed cost breakdown.").meta({ title: "agent.completions.response.CostDetails" });
export type AgentCompletionsResponseCostDetails = z.infer<typeof AgentCompletionsResponseCostDetailsSchema>;
