import z from "zod";
import { InputSchemaSchema } from "./expression/input";
import { TaskExpressionsSchema } from "./task";
import { ExpressionSchema } from "./expression/expression";
import { convert, type JsonSchema } from "../json_schema";

// Inline Function

export const InlineScalarFunctionSchema = z
  .object({
    type: z.literal("scalar.function"),
    tasks: TaskExpressionsSchema,
  })
  .describe(
    "A scalar function defined inline. Each task's output expression must return a number in [0,1]. The function's output is the weighted average of all task outputs using profile weights. If there is only one task, its output becomes the function's output directly.",
  )
  .meta({ title: "InlineScalarFunction" });
export type InlineScalarFunction = z.infer<typeof InlineScalarFunctionSchema>;
export const InlineScalarFunctionJsonSchema: JsonSchema = convert(
  InlineScalarFunctionSchema,
);

export const InlineVectorFunctionSchema = z
  .object({
    type: z.literal("vector.function"),
    tasks: TaskExpressionsSchema,
    input_split: ExpressionSchema.optional()
      .nullable()
      .describe(
        "Splits the function input into an array of sub-inputs, one per output element. " +
          "The array length must equal `output_length`. Each sub-input, when executed independently, must produce `output_length = 1`. " +
          "Used by execution strategies (e.g., swiss_system) that process subsets of the split inputs in parallel pools. " +
          "Only required when using such a strategy. " +
          "Receives: `input`.",
      ),
    input_merge: ExpressionSchema.optional()
      .nullable()
      .describe(
        "Recombines a variable-size, arbitrarily-ordered subset of sub-inputs (produced by `input_split`) into a single input. " +
          "The merged input is then executed as a normal function call. " +
          "Used by execution strategies (e.g., swiss_system) that group sub-inputs into pools for parallel evaluation. " +
          "Only required when using such a strategy. " +
          "Receives: `input` (an array of sub-inputs).",
      ),
  })
  .describe(
    "A vector function defined inline. Each task's output expression must return an array of numbers summing to ~1. The function's output is the weighted average of all task outputs using profile weights. If there is only one task, its output becomes the function's output directly.",
  )
  .meta({ title: "InlineVectorFunction" });
export type InlineVectorFunction = z.infer<typeof InlineVectorFunctionSchema>;
export const InlineVectorFunctionJsonSchema: JsonSchema = convert(
  InlineVectorFunctionSchema,
);

export const InlineFunctionSchema = z
  .discriminatedUnion("type", [
    InlineScalarFunctionSchema,
    InlineVectorFunctionSchema,
  ])
  .describe("A function defined inline.")
  .meta({ title: "InlineFunction" });
export type InlineFunction = z.infer<typeof InlineFunctionSchema>;
export const InlineFunctionJsonSchema: JsonSchema =
  convert(InlineFunctionSchema);

// Remote Function

export const RemoteScalarFunctionSchema = InlineScalarFunctionSchema.extend({
  description: z.string().describe("The description of the scalar function."),
  input_schema: InputSchemaSchema,
})
  .describe('A remote scalar function. "function.json"')
  .meta({ title: "RemoteScalarFunction" });
export type RemoteScalarFunction = z.infer<typeof RemoteScalarFunctionSchema>;
export const RemoteScalarFunctionJsonSchema: JsonSchema = convert(
  RemoteScalarFunctionSchema,
);

export const RemoteVectorFunctionSchema = InlineVectorFunctionSchema.extend({
  description: z.string().describe("The description of the vector function."),
  input_schema: InputSchemaSchema,
  output_length: ExpressionSchema.describe(
    "An expression which evaluates to the length of the output vector. The output length must be determinable from the input alone. Receives: `input`.",
  ),
  input_split: ExpressionSchema.describe(
    "Splits the function input into an array of sub-inputs, one per output element. " +
      "The array length must equal `output_length`. Each sub-input, when executed independently, must produce `output_length = 1`. " +
      "Used by execution strategies (e.g., swiss_system) that process subsets of the split inputs in parallel pools. " +
      "Receives: `input`.",
  ),
  input_merge: ExpressionSchema.describe(
    "Recombines a variable-size, arbitrarily-ordered subset of sub-inputs (produced by `input_split`) into a single input. " +
      "The merged input is then executed as a normal function call. " +
      "Used by execution strategies (e.g., swiss_system) that group sub-inputs into pools for parallel evaluation. " +
      "Receives: `input` (an array of sub-inputs).",
  ),
})
  .describe('A remote vector function. "function.json"')
  .meta({ title: "RemoteVectorFunction" });
export type RemoteVectorFunction = z.infer<typeof RemoteVectorFunctionSchema>;
export const RemoteVectorFunctionJsonSchema: JsonSchema = convert(
  RemoteVectorFunctionSchema,
);

export const RemoteFunctionSchema = z
  .discriminatedUnion("type", [
    RemoteScalarFunctionSchema,
    RemoteVectorFunctionSchema,
  ])
  .describe('A remote function. "function.json"')
  .meta({ title: "RemoteFunction" });
export type RemoteFunction = z.infer<typeof RemoteFunctionSchema>;
export const RemoteFunctionJsonSchema: JsonSchema =
  convert(RemoteFunctionSchema);

// Function

export const FunctionSchema = z
  .union([InlineFunctionSchema, RemoteFunctionSchema])
  .describe("A function.")
  .meta({ title: "Function" });
export type Function = z.infer<typeof FunctionSchema>;
export const FunctionJsonSchema: JsonSchema = convert(FunctionSchema);
