import { z } from "zod";
import { FunctionsExpressionExpressionSchema } from "../expression/expression";
import { FunctionsRemoteSchema } from "../remote";

export const FunctionsAlphaScalarScalarFunctionTaskExpressionSchema = z.object({
  remote: FunctionsRemoteSchema,
  owner: z.string(),
  repository: z.string(),
  commit: z.string(),
  skip: FunctionsExpressionExpressionSchema.nullable().optional(),
  input: FunctionsExpressionExpressionSchema,
}).meta({ title: "functions.alpha_scalar.ScalarFunctionTaskExpression" });
export type FunctionsAlphaScalarScalarFunctionTaskExpression = z.infer<typeof FunctionsAlphaScalarScalarFunctionTaskExpressionSchema>;
