import z from "zod";
import { ExpressionSchema } from "../expression/expression.js";
import { ObjectInputSchemaSchema } from "../expression/input.js";
import { RemoteSchema } from "../remote.js";
import {
  AlphaVectorFunctionInputExpressionSchema,
  AlphaVectorFunctionInputSchemaSchema,
} from "./expression/input.js";
import { convert, type JSONSchema } from "../../json_schema.js";

// Branch Tasks

export const AlphaVectorScalarFunctionTaskExpressionSchema = z
  .object({
    type: z.literal("alpha.scalar.function"),
    remote: RemoteSchema,
    owner: z
      .string()
      .describe("The owner of the repository containing the function."),
    repository: z
      .string()
      .describe("The name of the repository containing the function."),
    commit: z
      .string()
      .describe(
        "The commit SHA of the repository containing the function.",
      ),
    skip: ExpressionSchema.optional().describe(
      "An expression which evaluates to a boolean indicating whether to skip this task. Receives: `input`.",
    ),
    input: ExpressionSchema.describe(
      "An expression which evaluates to the input for the child scalar function. Receives: `input`.",
    ),
  })
  .describe("An alpha scalar function task expression within a vector function.")
  .meta({ title: "AlphaVectorScalarFunctionTaskExpression" });
export type AlphaVectorScalarFunctionTaskExpression = z.infer<typeof AlphaVectorScalarFunctionTaskExpressionSchema>;
export const AlphaVectorScalarFunctionTaskExpressionJsonSchema: JSONSchema = convert(AlphaVectorScalarFunctionTaskExpressionSchema);

export const AlphaVectorFunctionTaskExpressionSchema = z
  .object({
    type: z.literal("alpha.vector.function"),
    remote: RemoteSchema,
    owner: z
      .string()
      .describe("The owner of the repository containing the function."),
    repository: z
      .string()
      .describe("The name of the repository containing the function."),
    commit: z
      .string()
      .describe(
        "The commit SHA of the repository containing the function.",
      ),
    skip: ExpressionSchema.optional().describe(
      "An expression which evaluates to a boolean indicating whether to skip this task. Receives: `input`.",
    ),
    input: AlphaVectorFunctionInputExpressionSchema.describe(
      "The input expression for the child vector function.",
    ),
  })
  .describe("An alpha vector function task expression.")
  .meta({ title: "AlphaVectorFunctionTaskExpression" });
export type AlphaVectorFunctionTaskExpression = z.infer<typeof AlphaVectorFunctionTaskExpressionSchema>;
export const AlphaVectorFunctionTaskExpressionJsonSchema: JSONSchema = convert(AlphaVectorFunctionTaskExpressionSchema);

export const PlaceholderAlphaVectorScalarFunctionTaskExpressionSchema = z
  .object({
    type: z.literal("placeholder.alpha.scalar.function"),
    name: z.string().describe("The name of the placeholder function."),
    spec: z.string().describe("The specification of the placeholder function."),
    input_schema: ObjectInputSchemaSchema.describe(
      "The input schema for the placeholder scalar function.",
    ),
    skip: ExpressionSchema.optional().describe(
      "An expression which evaluates to a boolean indicating whether to skip this task. Receives: `input`.",
    ),
    input: ExpressionSchema.describe(
      "An expression which evaluates to the input for the placeholder scalar function. Receives: `input`.",
    ),
  })
  .describe(
    "A placeholder alpha scalar function task expression within a vector function. Always outputs 0.5.",
  )
  .meta({ title: "PlaceholderAlphaVectorScalarFunctionTaskExpression" });
export type PlaceholderAlphaVectorScalarFunctionTaskExpression = z.infer<typeof PlaceholderAlphaVectorScalarFunctionTaskExpressionSchema>;
export const PlaceholderAlphaVectorScalarFunctionTaskExpressionJsonSchema: JSONSchema = convert(PlaceholderAlphaVectorScalarFunctionTaskExpressionSchema);

export const PlaceholderAlphaVectorFunctionTaskExpressionSchema = z
  .object({
    type: z.literal("placeholder.alpha.vector.function"),
    name: z.string().describe("The name of the placeholder function."),
    spec: z.string().describe("The specification of the placeholder function."),
    input_schema: AlphaVectorFunctionInputSchemaSchema.describe(
      "The input schema for the placeholder vector function.",
    ),
    skip: ExpressionSchema.optional().describe(
      "An expression which evaluates to a boolean indicating whether to skip this task. Receives: `input`.",
    ),
    input: AlphaVectorFunctionInputExpressionSchema.describe(
      "The input expression for the placeholder vector function.",
    ),
  })
  .describe(
    "A placeholder alpha vector function task expression. Always outputs an equalized vector.",
  )
  .meta({ title: "PlaceholderAlphaVectorFunctionTaskExpression" });
export type PlaceholderAlphaVectorFunctionTaskExpression = z.infer<typeof PlaceholderAlphaVectorFunctionTaskExpressionSchema>;
export const PlaceholderAlphaVectorFunctionTaskExpressionJsonSchema: JSONSchema = convert(PlaceholderAlphaVectorFunctionTaskExpressionSchema);

export const AlphaVectorBranchTaskExpressionSchema = z
  .discriminatedUnion("type", [
    AlphaVectorScalarFunctionTaskExpressionSchema,
    AlphaVectorFunctionTaskExpressionSchema,
    PlaceholderAlphaVectorScalarFunctionTaskExpressionSchema,
    PlaceholderAlphaVectorFunctionTaskExpressionSchema,
  ])
  .describe("A branch task expression for an alpha vector function.")
  .meta({ title: "AlphaVectorBranchTaskExpression" });
export type AlphaVectorBranchTaskExpression = z.infer<typeof AlphaVectorBranchTaskExpressionSchema>;
export const AlphaVectorBranchTaskExpressionJsonSchema: JSONSchema = convert(AlphaVectorBranchTaskExpressionSchema);

export const AlphaVectorBranchTaskExpressionsSchema = z
  .array(AlphaVectorBranchTaskExpressionSchema)
  .describe(
    "The list of branch tasks to be executed as part of the alpha vector function.",
  )
  .meta({ title: "AlphaVectorBranchTaskExpressions" });
export type AlphaVectorBranchTaskExpressions = z.infer<typeof AlphaVectorBranchTaskExpressionsSchema>;
export const AlphaVectorBranchTaskExpressionsJsonSchema: JSONSchema = convert(AlphaVectorBranchTaskExpressionsSchema);

// Leaf Tasks

export const AlphaVectorVectorCompletionTaskExpressionSchema = z
  .object({
    type: z.literal("vector.completion"),
    skip: ExpressionSchema.optional().describe(
      "An expression which evaluates to a boolean indicating whether to skip this task. Receives: `input`.",
    ),
    messages: ExpressionSchema.describe(
      "An expression which evaluates to the chat messages. Receives: `input`.",
    ),
    responses: ExpressionSchema.describe(
      "An expression which evaluates to the response options for the vector completion. Receives: `input`.",
    ),
  })
  .describe("A vector completion task expression for an alpha vector leaf function.")
  .meta({ title: "AlphaVectorVectorCompletionTaskExpression" });
export type AlphaVectorVectorCompletionTaskExpression = z.infer<typeof AlphaVectorVectorCompletionTaskExpressionSchema>;
export const AlphaVectorVectorCompletionTaskExpressionJsonSchema: JSONSchema = convert(AlphaVectorVectorCompletionTaskExpressionSchema);

export const AlphaVectorLeafTaskExpressionSchema = z
  .discriminatedUnion("type", [
    AlphaVectorVectorCompletionTaskExpressionSchema,
  ])
  .describe("A leaf task expression for an alpha vector function.")
  .meta({ title: "AlphaVectorLeafTaskExpression" });
export type AlphaVectorLeafTaskExpression = z.infer<typeof AlphaVectorLeafTaskExpressionSchema>;
export const AlphaVectorLeafTaskExpressionJsonSchema: JSONSchema = convert(AlphaVectorLeafTaskExpressionSchema);

export const AlphaVectorLeafTaskExpressionsSchema = z
  .array(AlphaVectorLeafTaskExpressionSchema)
  .describe(
    "The list of leaf tasks to be executed as part of the alpha vector function.",
  )
  .meta({ title: "AlphaVectorLeafTaskExpressions" });
export type AlphaVectorLeafTaskExpressions = z.infer<typeof AlphaVectorLeafTaskExpressionsSchema>;
export const AlphaVectorLeafTaskExpressionsJsonSchema: JSONSchema = convert(AlphaVectorLeafTaskExpressionsSchema);
