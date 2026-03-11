import { z } from "zod";
import { AgentCompletionsResponseUsageSchema } from "../../../../agent/completions/response/usage";
import { ResponseErrorSchema } from "../../../../responseError";
import { VectorCompletionsResponseUnaryAgentCompletionSchema } from "../../../../vector/completions/response/unary/agentCompletion";
import { VectorCompletionsResponseUnaryObjectSchema } from "../../../../vector/completions/response/unary/object";
import { VectorCompletionsResponseVoteSchema } from "../../../../vector/completions/response/vote";

export const FunctionsExecutionsResponseUnaryVectorCompletionTaskSchema = z.object({
  index: z.number().int().min(0).meta({ format: "uint64" }),
  task_index: z.number().int().min(0).meta({ format: "uint64" }),
  task_path: z.array(z.number().int().min(0).meta({ format: "uint64" })),
  id: z.string().describe("Unique identifier for this vector completion."),
  completions: z.array(VectorCompletionsResponseUnaryAgentCompletionSchema).describe("The underlying agent completions from each agent in the ensemble."),
  votes: z.array(VectorCompletionsResponseVoteSchema).describe("Individual votes from each agent, showing their selections."),
  scores: z.array(z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()])).describe("Final weighted scores for each response option. Sums to 1."),
  weights: z.array(z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()])).describe("Total weight allocated to each response option. Same length as `scores`.\nFor discrete votes, an LLM's full weight goes to its selected response.\nFor probabilistic votes, the weight is divided according to the distribution."),
  created: z.number().int().min(0).meta({ format: "uint64" }).describe("Unix timestamp when the completion was created."),
  ensemble: z.string().describe("ID of the ensemble used for this completion."),
  object: VectorCompletionsResponseUnaryObjectSchema.describe("Object type identifier (`\"vector.completion\"`)."),
  usage: AgentCompletionsResponseUsageSchema.describe("Aggregated token and cost usage across all completions."),
  error: ResponseErrorSchema.nullable().optional(),
}).describe("A complete vector completion response (non-streaming).\n\nContains the final scores, all votes from the ensemble, and the underlying\nagent completions that produced those votes.").meta({ title: "functions.executions.response.unary.VectorCompletionTask" });
export type FunctionsExecutionsResponseUnaryVectorCompletionTask = z.infer<typeof FunctionsExecutionsResponseUnaryVectorCompletionTaskSchema>;
