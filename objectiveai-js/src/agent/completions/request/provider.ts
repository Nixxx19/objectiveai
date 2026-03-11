import { z } from "zod";
import { AgentCompletionsRequestProviderDataCollectionSchema } from "./providerDataCollection";
import { AgentCompletionsRequestProviderMaxPriceSchema } from "./providerMaxPrice";
import { AgentCompletionsRequestProviderSortSchema } from "./providerSort";

export const AgentCompletionsRequestProviderSchema = z.object({
  data_collection: AgentCompletionsRequestProviderDataCollectionSchema.nullable().describe("Whether to allow providers to collect data.").optional(),
  zdr: z.boolean().nullable().describe("Whether to use zero data retention providers only.").optional(),
  sort: AgentCompletionsRequestProviderSortSchema.nullable().describe("How to sort/prioritize providers.").optional(),
  max_price: AgentCompletionsRequestProviderMaxPriceSchema.nullable().describe("Maximum price constraints.").optional(),
  preferred_min_throughput: z.number().meta({ format: "double" }).nullable().describe("Preferred minimum throughput (tokens/second).").optional(),
  preferred_max_latency: z.number().meta({ format: "double" }).nullable().describe("Preferred maximum latency (seconds).").optional(),
  min_throughput: z.number().meta({ format: "double" }).nullable().describe("Hard minimum throughput requirement (tokens/second).").optional(),
  max_latency: z.number().meta({ format: "double" }).nullable().describe("Hard maximum latency requirement (seconds).").optional(),
}).describe("Provider routing and selection preferences.").meta({ title: "agent.completions.request.Provider" });
export type AgentCompletionsRequestProvider = z.infer<typeof AgentCompletionsRequestProviderSchema>;
