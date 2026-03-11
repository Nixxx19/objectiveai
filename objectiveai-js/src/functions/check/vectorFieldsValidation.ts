import { z } from "zod";
import { FunctionsExpressionExpressionSchema } from "../expression/expression";
import { FunctionsExpressionInputSchemaSchema } from "../expression/inputSchema";

export const FunctionsCheckVectorFieldsValidationSchema = z.object({
  input_schema: FunctionsExpressionInputSchemaSchema,
  output_length: FunctionsExpressionExpressionSchema,
  input_split: FunctionsExpressionExpressionSchema,
  input_merge: FunctionsExpressionExpressionSchema,
}).describe("The 4 fields needed to validate a vector function's split/merge behavior.").meta({ title: "functions.check.VectorFieldsValidation" });
export type FunctionsCheckVectorFieldsValidation = z.infer<typeof FunctionsCheckVectorFieldsValidationSchema>;
