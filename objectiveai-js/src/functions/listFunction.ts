import { z } from "zod";
import { FunctionsListFunctionItemSchema } from "./listFunctionItem";

export const FunctionsListFunctionSchema = z.object({
  data: z.array(FunctionsListFunctionItemSchema).describe("List of available functions."),
}).describe("Response from listing functions.").meta({ title: "functions.ListFunction" });
export type FunctionsListFunction = z.infer<typeof FunctionsListFunctionSchema>;
