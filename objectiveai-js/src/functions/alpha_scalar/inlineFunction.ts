import { z } from "zod";
import { FunctionsAlphaScalarBranchTaskExpressionSchema } from "./branchTaskExpression";
import { FunctionsAlphaScalarLeafTaskExpressionSchema } from "./leafTaskExpression";

export const FunctionsAlphaScalarInlineFunctionSchema = z.union([z.object({
  tasks: z.array(FunctionsAlphaScalarBranchTaskExpressionSchema),
  type: z.literal("alpha.scalar.branch.function"),
}), z.object({
  tasks: z.array(FunctionsAlphaScalarLeafTaskExpressionSchema),
  type: z.literal("alpha.scalar.leaf.function"),
})]).meta({ title: "functions.alpha_scalar.InlineFunction" });
export type FunctionsAlphaScalarInlineFunction = z.infer<typeof FunctionsAlphaScalarInlineFunctionSchema>;
