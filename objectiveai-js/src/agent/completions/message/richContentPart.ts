import { z } from "zod";
import { AgentCompletionsMessageFileSchema } from "./file";
import { AgentCompletionsMessageImageUrlSchema } from "./imageUrl";
import { AgentCompletionsMessageInputAudioSchema } from "./inputAudio";
import { AgentCompletionsMessageVideoUrlSchema } from "./videoUrl";

export const AgentCompletionsMessageRichContentPartSchema = z.union([z.object({
  text: z.string(),
  type: z.literal("text"),
}).describe("Text content."), z.object({
  image_url: AgentCompletionsMessageImageUrlSchema,
  type: z.literal("image_url"),
}).describe("An image URL."), z.object({
  input_audio: AgentCompletionsMessageInputAudioSchema,
  type: z.literal("input_audio"),
}).describe("Audio input."), z.object({
  video_url: AgentCompletionsMessageVideoUrlSchema,
  type: z.literal("input_video"),
}).describe("Video input."), z.object({
  video_url: AgentCompletionsMessageVideoUrlSchema,
  type: z.literal("video_url"),
}).describe("A video URL."), z.object({
  file: AgentCompletionsMessageFileSchema,
  type: z.literal("file"),
}).describe("A file.")]).describe("A part of rich content.").meta({ title: "agent.completions.message.RichContentPart" });
export type AgentCompletionsMessageRichContentPart = z.infer<typeof AgentCompletionsMessageRichContentPartSchema>;
