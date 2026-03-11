import { z } from "zod";
import { FunctionsExpressionExpressionSchema } from "../expression/expression";
import { FunctionsExpressionObjectInputSchemaSchema } from "../expression/objectInputSchema";

export const FunctionsAlphaScalarPartialPlaceholderScalarFunctionTaskExpressionSchema = z.object({
  spec: z.string(),
  input_schema: FunctionsExpressionObjectInputSchemaSchema,
  skip: FunctionsExpressionExpressionSchema.nullable().optional(),
  input: FunctionsExpressionExpressionSchema,
}).meta({ title: "functions.alpha_scalar.PartialPlaceholderScalarFunctionTaskExpression" });
export type FunctionsAlphaScalarPartialPlaceholderScalarFunctionTaskExpression = z.infer<typeof FunctionsAlphaScalarPartialPlaceholderScalarFunctionTaskExpressionSchema>;
