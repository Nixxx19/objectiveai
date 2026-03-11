import { z } from "zod";
import { AgentCompletionsResponseUsageSchema } from "../../../../agent/completions/response/usage";
import { FunctionsFullRemoteFunctionSchema } from "../../../fullRemoteFunction";
import { FunctionsInventionsResponseUnaryAgentCompletionSchema } from "./agentCompletion";
import { FunctionsInventionsResponseUnaryObjectSchema } from "./object";
import { FunctionsInventionsStateStateSchema } from "../../state/state";
import { FunctionsRemoteFunctionPathSchema } from "../../../remoteFunctionPath";
import { ResponseErrorSchema } from "../../../../responseError";

export const FunctionsInventionsResponseUnaryFunctionInventionSchema = z.object({
  id: z.string(),
  completions: z.array(FunctionsInventionsResponseUnaryAgentCompletionSchema),
  state: FunctionsInventionsStateStateSchema,
  path: FunctionsRemoteFunctionPathSchema.nullable().optional(),
  function: FunctionsFullRemoteFunctionSchema.nullable().optional(),
  created: z.number().int().min(0).meta({ format: "uint64" }),
  object: FunctionsInventionsResponseUnaryObjectSchema,
  usage: AgentCompletionsResponseUsageSchema,
  error: ResponseErrorSchema.nullable().optional(),
}).meta({ title: "functions.inventions.response.unary.FunctionInvention" });
export type FunctionsInventionsResponseUnaryFunctionInvention = z.infer<typeof FunctionsInventionsResponseUnaryFunctionInventionSchema>;
