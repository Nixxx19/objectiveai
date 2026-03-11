import { z } from "zod";
import { FunctionsAlphaScalarVectorCompletionTaskExpressionSchema } from "./vectorCompletionTaskExpression";

export const FunctionsAlphaScalarLeafTaskExpressionSchema = z.union([FunctionsAlphaScalarVectorCompletionTaskExpressionSchema.extend({
  type: z.literal("vector.completion"),
})]).meta({ title: "functions.alpha_scalar.LeafTaskExpression" });
export type FunctionsAlphaScalarLeafTaskExpression = z.infer<typeof FunctionsAlphaScalarLeafTaskExpressionSchema>;
