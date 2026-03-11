import { z } from "zod";

export const AgentCompletionsRequestProviderMaxPriceSchema = z.object({
  prompt: z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()]).nullable().describe("Maximum price per prompt token.").optional(),
  completion: z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()]).nullable().describe("Maximum price per completion token.").optional(),
  image: z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()]).nullable().describe("Maximum price per image.").optional(),
  audio: z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()]).nullable().describe("Maximum price per audio second.").optional(),
  request: z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()]).nullable().describe("Maximum price per request.").optional(),
}).describe("Maximum price constraints per token type.").meta({ title: "agent.completions.request.ProviderMaxPrice" });
export type AgentCompletionsRequestProviderMaxPrice = z.infer<typeof AgentCompletionsRequestProviderMaxPriceSchema>;
