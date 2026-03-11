import { z } from "zod";
import { FunctionsExpressionExpressionSchema } from "../expression/expression";
import { FunctionsExpressionObjectInputSchemaSchema } from "../expression/objectInputSchema";

export const FunctionsAlphaVectorPartialPlaceholderScalarFunctionTaskExpressionSchema = z.object({
  spec: z.string(),
  input_schema: FunctionsExpressionObjectInputSchemaSchema,
  skip: FunctionsExpressionExpressionSchema.nullable().optional(),
  input: FunctionsExpressionExpressionSchema,
}).meta({ title: "functions.alpha_vector.PartialPlaceholderScalarFunctionTaskExpression" });
export type FunctionsAlphaVectorPartialPlaceholderScalarFunctionTaskExpression = z.infer<typeof FunctionsAlphaVectorPartialPlaceholderScalarFunctionTaskExpressionSchema>;
