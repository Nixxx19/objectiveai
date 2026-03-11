import { z } from "zod";
import { FunctionsAlphaScalarPlaceholderScalarFunctionTaskExpressionSchema } from "./placeholderScalarFunctionTaskExpression";
import { FunctionsAlphaScalarScalarFunctionTaskExpressionSchema } from "./scalarFunctionTaskExpression";

export const FunctionsAlphaScalarBranchTaskExpressionSchema = z.union([FunctionsAlphaScalarScalarFunctionTaskExpressionSchema.extend({
  type: z.literal("alpha.scalar.function"),
}), FunctionsAlphaScalarPlaceholderScalarFunctionTaskExpressionSchema.extend({
  type: z.literal("placeholder.alpha.scalar.function"),
})]).meta({ title: "functions.alpha_scalar.BranchTaskExpression" });
export type FunctionsAlphaScalarBranchTaskExpression = z.infer<typeof FunctionsAlphaScalarBranchTaskExpressionSchema>;
