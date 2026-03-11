import { z } from "zod";
import { AgentCompletionsMessageRichContentSchema } from "../../agent/completions/message/richContent";
import { FunctionsExpressionExpressionSchema } from "../expression/expression";

export const FunctionsAlphaScalarVectorCompletionTaskExpressionSchema = z.object({
  skip: FunctionsExpressionExpressionSchema.nullable().optional(),
  messages: FunctionsExpressionExpressionSchema,
  responses: z.array(AgentCompletionsMessageRichContentSchema),
}).meta({ title: "functions.alpha_scalar.VectorCompletionTaskExpression" });
export type FunctionsAlphaScalarVectorCompletionTaskExpression = z.infer<typeof FunctionsAlphaScalarVectorCompletionTaskExpressionSchema>;
