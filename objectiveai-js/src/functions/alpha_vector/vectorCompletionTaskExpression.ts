import { z } from "zod";
import { FunctionsExpressionExpressionSchema } from "../expression/expression";

export const FunctionsAlphaVectorVectorCompletionTaskExpressionSchema = z.object({
  skip: FunctionsExpressionExpressionSchema.nullable().optional(),
  messages: FunctionsExpressionExpressionSchema,
  responses: FunctionsExpressionExpressionSchema,
}).meta({ title: "functions.alpha_vector.VectorCompletionTaskExpression" });
export type FunctionsAlphaVectorVectorCompletionTaskExpression = z.infer<typeof FunctionsAlphaVectorVectorCompletionTaskExpressionSchema>;
