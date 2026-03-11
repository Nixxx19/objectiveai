import { z } from "zod";
import { FunctionsExpressionInputSchema } from "../../expression/input";

export const FunctionsAlphaVectorExpressionVectorFunctionInputSchema = z.object({
  context: z.record(z.string(), FunctionsExpressionInputSchema).nullable().optional(),
  items: z.array(FunctionsExpressionInputSchema),
}).meta({ title: "functions.alpha_vector.expression.VectorFunctionInput" });
export type FunctionsAlphaVectorExpressionVectorFunctionInput = z.infer<typeof FunctionsAlphaVectorExpressionVectorFunctionInputSchema>;
