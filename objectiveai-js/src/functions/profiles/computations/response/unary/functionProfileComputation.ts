import { z } from "zod";
import { AgentCompletionsResponseUsageSchema } from "../../../../../agent/completions/response/usage";
import { FunctionsInlineTasksProfileSchema } from "../../../../inlineTasksProfile";
import { FunctionsProfilesComputationsResponseFittingStatsSchema } from "../fittingStats";
import { FunctionsProfilesComputationsResponseUnaryFunctionExecutionSchema } from "./functionExecution";
import { FunctionsProfilesComputationsResponseUnaryObjectSchema } from "./object";

export const FunctionsProfilesComputationsResponseUnaryFunctionProfileComputationSchema = z.object({
  id: z.string(),
  executions: z.array(FunctionsProfilesComputationsResponseUnaryFunctionExecutionSchema),
  executions_errors: z.boolean(),
  profile: FunctionsInlineTasksProfileSchema,
  fitting_stats: FunctionsProfilesComputationsResponseFittingStatsSchema,
  retry_token: z.string().nullable().optional(),
  created: z.number().int().min(0).meta({ format: "uint64" }),
  function: z.string().nullable().optional(),
  object: FunctionsProfilesComputationsResponseUnaryObjectSchema,
  usage: AgentCompletionsResponseUsageSchema,
}).meta({ title: "functions.profiles.computations.response.unary.FunctionProfileComputation" });
export type FunctionsProfilesComputationsResponseUnaryFunctionProfileComputation = z.infer<typeof FunctionsProfilesComputationsResponseUnaryFunctionProfileComputationSchema>;
