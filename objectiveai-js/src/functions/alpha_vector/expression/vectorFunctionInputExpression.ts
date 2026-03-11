import { z } from "zod";
import { FunctionsExpressionExpressionSchema } from "../../expression/expression";

export const FunctionsAlphaVectorExpressionVectorFunctionInputExpressionSchema = z.object({
  context: FunctionsExpressionExpressionSchema.nullable().optional(),
  items: FunctionsExpressionExpressionSchema,
}).meta({ title: "functions.alpha_vector.expression.VectorFunctionInputExpression" });
export type FunctionsAlphaVectorExpressionVectorFunctionInputExpression = z.infer<typeof FunctionsAlphaVectorExpressionVectorFunctionInputExpressionSchema>;
