import { z } from "zod";

export const AgentCompletionsMessageVideoUrlSchema = z.object({
  url: z.string().describe("The URL of the video."),
}).describe("A video URL for multimodal input.").meta({ title: "agent.completions.message.VideoUrl" });
export type AgentCompletionsMessageVideoUrl = z.infer<typeof AgentCompletionsMessageVideoUrlSchema>;
