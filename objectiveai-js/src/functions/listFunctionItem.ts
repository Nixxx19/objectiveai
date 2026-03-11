import { z } from "zod";
import { FunctionsRemoteSchema } from "./remote";

export const FunctionsListFunctionItemSchema = z.object({
  remote: FunctionsRemoteSchema.describe("The remote source where the function is hosted."),
  owner: z.string().describe("Repository owner."),
  repository: z.string().describe("Repository name."),
  commit: z.string().describe("Git commit SHA."),
}).describe("A function in a list response.").meta({ title: "functions.ListFunctionItem" });
export type FunctionsListFunctionItem = z.infer<typeof FunctionsListFunctionItemSchema>;
