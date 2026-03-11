import { z } from "zod";
import { FunctionsAlphaVectorExpressionVectorFunctionInputExpressionSchema } from "./expression/vectorFunctionInputExpression";
import { FunctionsExpressionExpressionSchema } from "../expression/expression";
import { FunctionsRemoteSchema } from "../remote";

export const FunctionsAlphaVectorVectorFunctionTaskExpressionSchema = z.object({
  remote: FunctionsRemoteSchema,
  owner: z.string(),
  repository: z.string(),
  commit: z.string(),
  skip: FunctionsExpressionExpressionSchema.nullable().optional(),
  input: FunctionsAlphaVectorExpressionVectorFunctionInputExpressionSchema,
}).meta({ title: "functions.alpha_vector.VectorFunctionTaskExpression" });
export type FunctionsAlphaVectorVectorFunctionTaskExpression = z.infer<typeof FunctionsAlphaVectorVectorFunctionTaskExpressionSchema>;
