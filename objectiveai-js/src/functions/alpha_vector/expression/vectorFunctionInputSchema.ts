import { z } from "zod";
import { FunctionsExpressionInputSchemaSchema } from "../../expression/inputSchema";
import { FunctionsExpressionObjectInputSchemaSchema } from "../../expression/objectInputSchema";

export const FunctionsAlphaVectorExpressionVectorFunctionInputSchemaSchema = z.object({
  context: FunctionsExpressionObjectInputSchemaSchema.nullable().optional(),
  items: FunctionsExpressionInputSchemaSchema,
}).meta({ title: "functions.alpha_vector.expression.VectorFunctionInputSchema" });
export type FunctionsAlphaVectorExpressionVectorFunctionInputSchema = z.infer<typeof FunctionsAlphaVectorExpressionVectorFunctionInputSchemaSchema>;
