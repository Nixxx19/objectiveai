import { z } from "zod";
import { AgentCompletionsResponseUsageSchema } from "../../../../../agent/completions/response/usage";
import { FunctionsInventionsRecursiveResponseUnaryFunctionInventionSchema } from "./functionInvention";
import { FunctionsInventionsRecursiveResponseUnaryObjectSchema } from "./object";

export const FunctionsInventionsRecursiveResponseUnaryFunctionInventionRecursiveSchema = z.object({
  id: z.string(),
  inventions: z.array(FunctionsInventionsRecursiveResponseUnaryFunctionInventionSchema),
  inventions_errors: z.boolean(),
  created: z.number().int().min(0).meta({ format: "uint64" }),
  object: FunctionsInventionsRecursiveResponseUnaryObjectSchema,
  usage: AgentCompletionsResponseUsageSchema,
}).meta({ title: "functions.inventions.recursive.response.unary.FunctionInventionRecursive" });
export type FunctionsInventionsRecursiveResponseUnaryFunctionInventionRecursive = z.infer<typeof FunctionsInventionsRecursiveResponseUnaryFunctionInventionRecursiveSchema>;
