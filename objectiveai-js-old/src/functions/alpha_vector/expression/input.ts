import z from "zod";
import { ObjectInputSchemaSchema } from "../../expression/input.js";
import { InputSchemaSchema } from "../../expression/input.js";
import { ExpressionSchema } from "../../expression/expression.js";
import { convert, type JsonSchema } from "../../../jsonSchema.js";

export const AlphaVectorFunctionInputSchemaSchema = z
  .object({
    context: ObjectInputSchemaSchema.optional().describe(
      "The optional context input schema shared across all items.",
    ),
    items: InputSchemaSchema.describe("The input schema for each item."),
  })
  .describe("The input schema for an alpha vector function.")
  .meta({ title: "AlphaVectorFunctionInputSchema" });
export type AlphaVectorFunctionInputSchema = z.infer<typeof AlphaVectorFunctionInputSchemaSchema>;
export const AlphaVectorFunctionInputSchemaJsonSchema: JsonSchema = convert(AlphaVectorFunctionInputSchemaSchema);

export const AlphaVectorFunctionInputExpressionSchema = z
  .object({
    context: ExpressionSchema.optional().describe(
      "An expression which evaluates to the context input. Receives: `input`.",
    ),
    items: ExpressionSchema.describe(
      "An expression which evaluates to the items input. Receives: `input`.",
    ),
  })
  .describe("The input expression for an alpha vector function.")
  .meta({ title: "AlphaVectorFunctionInputExpression" });
export type AlphaVectorFunctionInputExpression = z.infer<typeof AlphaVectorFunctionInputExpressionSchema>;
export const AlphaVectorFunctionInputExpressionJsonSchema: JsonSchema = convert(AlphaVectorFunctionInputExpressionSchema);
