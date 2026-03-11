import { z } from "zod";
import { AgentCompletionsMessageImageUrlDetailSchema } from "./imageUrlDetail";

export const AgentCompletionsMessageImageUrlSchema = z.object({
  url: z.string().describe("The URL of the image (can be a data URL or HTTP URL)."),
  detail: AgentCompletionsMessageImageUrlDetailSchema.nullable().describe("The detail level for image processing.").optional(),
}).describe("An image URL for multimodal input.").meta({ title: "agent.completions.message.ImageUrl" });
export type AgentCompletionsMessageImageUrl = z.infer<typeof AgentCompletionsMessageImageUrlSchema>;
