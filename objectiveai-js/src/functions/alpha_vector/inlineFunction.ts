import { z } from "zod";
import { FunctionsAlphaVectorBranchTaskExpressionSchema } from "./branchTaskExpression";
import { FunctionsAlphaVectorLeafTaskExpressionSchema } from "./leafTaskExpression";

export const FunctionsAlphaVectorInlineFunctionSchema = z.union([z.object({
  tasks: z.array(FunctionsAlphaVectorBranchTaskExpressionSchema),
  type: z.literal("alpha.vector.branch.function"),
}), z.object({
  tasks: z.array(FunctionsAlphaVectorLeafTaskExpressionSchema),
  type: z.literal("alpha.vector.leaf.function"),
})]).meta({ title: "functions.alpha_vector.InlineFunction" });
export type FunctionsAlphaVectorInlineFunction = z.infer<typeof FunctionsAlphaVectorInlineFunctionSchema>;
