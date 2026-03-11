import { z } from "zod";

export const FunctionsRemoteSchema = z.union([z.literal("github").describe("GitHub repository."), z.literal("filesystem").describe("Local filesystem."), z.literal("mock").describe("Mock (for testing).")]).describe("The remote source where a function or profile is hosted.").meta({ title: "functions.Remote" });
export type FunctionsRemote = z.infer<typeof FunctionsRemoteSchema>;
