import { z } from "zod";
import { FunctionsAlphaVectorExpressionVectorFunctionInputExpressionSchema } from "./expression/vectorFunctionInputExpression";
import { FunctionsAlphaVectorExpressionVectorFunctionInputSchemaSchema } from "./expression/vectorFunctionInputSchema";
import { FunctionsExpressionExpressionSchema } from "../expression/expression";

export const FunctionsAlphaVectorPartialPlaceholderVectorFunctionTaskExpressionSchema = z.object({
  spec: z.string(),
  input_schema: FunctionsAlphaVectorExpressionVectorFunctionInputSchemaSchema,
  skip: FunctionsExpressionExpressionSchema.nullable().optional(),
  input: FunctionsAlphaVectorExpressionVectorFunctionInputExpressionSchema,
}).meta({ title: "functions.alpha_vector.PartialPlaceholderVectorFunctionTaskExpression" });
export type FunctionsAlphaVectorPartialPlaceholderVectorFunctionTaskExpression = z.infer<typeof FunctionsAlphaVectorPartialPlaceholderVectorFunctionTaskExpressionSchema>;
