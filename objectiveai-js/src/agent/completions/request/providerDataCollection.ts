import { z } from "zod";

export const AgentCompletionsRequestProviderDataCollectionSchema = z.union([z.literal("deny").describe("Do not allow data collection."), z.literal("allow").describe("Allow data collection.")]).describe("Data collection policy for providers.").meta({ title: "agent.completions.request.ProviderDataCollection" });
export type AgentCompletionsRequestProviderDataCollection = z.infer<typeof AgentCompletionsRequestProviderDataCollectionSchema>;
