import { z } from "zod";
import { FunctionsAlphaScalarBranchTaskExpressionSchema } from "./branchTaskExpression";
import { FunctionsAlphaScalarLeafTaskExpressionSchema } from "./leafTaskExpression";
import { FunctionsExpressionObjectInputSchemaSchema } from "../expression/objectInputSchema";

export const FunctionsAlphaScalarRemoteFunctionSchema = z.union([z.object({
  description: z.string(),
  input_schema: FunctionsExpressionObjectInputSchemaSchema,
  tasks: z.array(FunctionsAlphaScalarBranchTaskExpressionSchema),
  type: z.literal("alpha.scalar.branch.function"),
}), z.object({
  description: z.string(),
  input_schema: FunctionsExpressionObjectInputSchemaSchema,
  tasks: z.array(FunctionsAlphaScalarLeafTaskExpressionSchema),
  type: z.literal("alpha.scalar.leaf.function"),
})]).meta({ title: "functions.alpha_scalar.RemoteFunction" });
export type FunctionsAlphaScalarRemoteFunction = z.infer<typeof FunctionsAlphaScalarRemoteFunctionSchema>;
