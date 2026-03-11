import { z } from "zod";
import { FunctionsExpressionInputSchemaSchema } from "../expression/inputSchema";

export const FunctionsCheckScalarFieldsValidationSchema = z.object({
  input_schema: FunctionsExpressionInputSchemaSchema,
}).describe("The fields needed to validate a scalar function's input behavior.").meta({ title: "functions.check.ScalarFieldsValidation" });
export type FunctionsCheckScalarFieldsValidation = z.infer<typeof FunctionsCheckScalarFieldsValidationSchema>;
