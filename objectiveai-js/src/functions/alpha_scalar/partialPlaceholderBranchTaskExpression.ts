import { z } from "zod";
import { FunctionsAlphaScalarPartialPlaceholderScalarFunctionTaskExpressionSchema } from "./partialPlaceholderScalarFunctionTaskExpression";

export const FunctionsAlphaScalarPartialPlaceholderBranchTaskExpressionSchema = z.union([FunctionsAlphaScalarPartialPlaceholderScalarFunctionTaskExpressionSchema.extend({
  type: z.literal("placeholder.alpha.scalar.function"),
})]).meta({ title: "functions.alpha_scalar.PartialPlaceholderBranchTaskExpression" });
export type FunctionsAlphaScalarPartialPlaceholderBranchTaskExpression = z.infer<typeof FunctionsAlphaScalarPartialPlaceholderBranchTaskExpressionSchema>;
