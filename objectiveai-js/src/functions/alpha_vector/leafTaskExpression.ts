import { z } from "zod";
import { FunctionsAlphaVectorVectorCompletionTaskExpressionSchema } from "./vectorCompletionTaskExpression";

export const FunctionsAlphaVectorLeafTaskExpressionSchema = z.union([FunctionsAlphaVectorVectorCompletionTaskExpressionSchema.extend({
  type: z.literal("vector.completion"),
})]).meta({ title: "functions.alpha_vector.LeafTaskExpression" });
export type FunctionsAlphaVectorLeafTaskExpression = z.infer<typeof FunctionsAlphaVectorLeafTaskExpressionSchema>;
