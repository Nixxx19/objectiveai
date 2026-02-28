import z from "zod";
import { ExpressionSchema } from "../expression/expression.js";
import { ObjectInputSchemaSchema } from "../expression/input.js";
import { RichContentSchema } from "src/chat/completions/request/message";
import { RemoteSchema } from "../remote.js";
import { convert, type JsonSchema } from "../../jsonSchema.js";

// Branch Tasks

export const AlphaScalarScalarFunctionTaskExpressionSchema = z
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
  .describe("An alpha scalar function task expression.")
  .meta({ title: "AlphaScalarScalarFunctionTaskExpression" });
export type AlphaScalarScalarFunctionTaskExpression = z.infer<typeof AlphaScalarScalarFunctionTaskExpressionSchema>;
export const AlphaScalarScalarFunctionTaskExpressionJsonSchema: JsonSchema = convert(AlphaScalarScalarFunctionTaskExpressionSchema);

export const AlphaScalarPlaceholderScalarFunctionTaskExpressionSchema = z
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
    "A placeholder alpha scalar function task expression. Always outputs 0.5.",
  )
  .meta({ title: "AlphaScalarPlaceholderScalarFunctionTaskExpression" });
export type AlphaScalarPlaceholderScalarFunctionTaskExpression = z.infer<typeof AlphaScalarPlaceholderScalarFunctionTaskExpressionSchema>;
export const AlphaScalarPlaceholderScalarFunctionTaskExpressionJsonSchema: JsonSchema = convert(AlphaScalarPlaceholderScalarFunctionTaskExpressionSchema);

export const AlphaScalarBranchTaskExpressionSchema = z
  .discriminatedUnion("type", [
    AlphaScalarScalarFunctionTaskExpressionSchema,
    AlphaScalarPlaceholderScalarFunctionTaskExpressionSchema,
  ])
  .describe("A branch task expression for an alpha scalar function.")
  .meta({ title: "AlphaScalarBranchTaskExpression" });
export type AlphaScalarBranchTaskExpression = z.infer<typeof AlphaScalarBranchTaskExpressionSchema>;
export const AlphaScalarBranchTaskExpressionJsonSchema: JsonSchema = convert(AlphaScalarBranchTaskExpressionSchema);

export const AlphaScalarBranchTaskExpressionsSchema = z
  .array(AlphaScalarBranchTaskExpressionSchema)
  .describe(
    "The list of branch tasks to be executed as part of the alpha scalar function.",
  )
  .meta({ title: "AlphaScalarBranchTaskExpressions" });
export type AlphaScalarBranchTaskExpressions = z.infer<typeof AlphaScalarBranchTaskExpressionsSchema>;
export const AlphaScalarBranchTaskExpressionsJsonSchema: JsonSchema = convert(AlphaScalarBranchTaskExpressionsSchema);

// Leaf Tasks

export const AlphaScalarVectorCompletionTaskExpressionSchema = z
  .object({
    type: z.literal("vector.completion"),
    skip: ExpressionSchema.optional().describe(
      "An expression which evaluates to a boolean indicating whether to skip this task. Receives: `input`.",
    ),
    messages: ExpressionSchema.describe(
      "An expression which evaluates to the chat messages. Receives: `input`.",
    ),
    responses: z
      .array(RichContentSchema)
      .describe("The response options for the vector completion."),
  })
  .describe("A vector completion task expression for an alpha scalar leaf function.")
  .meta({ title: "AlphaScalarVectorCompletionTaskExpression" });
export type AlphaScalarVectorCompletionTaskExpression = z.infer<typeof AlphaScalarVectorCompletionTaskExpressionSchema>;
export const AlphaScalarVectorCompletionTaskExpressionJsonSchema: JsonSchema = convert(AlphaScalarVectorCompletionTaskExpressionSchema);

export const AlphaScalarLeafTaskExpressionSchema = z
  .discriminatedUnion("type", [
    AlphaScalarVectorCompletionTaskExpressionSchema,
  ])
  .describe("A leaf task expression for an alpha scalar function.")
  .meta({ title: "AlphaScalarLeafTaskExpression" });
export type AlphaScalarLeafTaskExpression = z.infer<typeof AlphaScalarLeafTaskExpressionSchema>;
export const AlphaScalarLeafTaskExpressionJsonSchema: JsonSchema = convert(AlphaScalarLeafTaskExpressionSchema);

export const AlphaScalarLeafTaskExpressionsSchema = z
  .array(AlphaScalarLeafTaskExpressionSchema)
  .describe(
    "The list of leaf tasks to be executed as part of the alpha scalar function.",
  )
  .meta({ title: "AlphaScalarLeafTaskExpressions" });
export type AlphaScalarLeafTaskExpressions = z.infer<typeof AlphaScalarLeafTaskExpressionsSchema>;
export const AlphaScalarLeafTaskExpressionsJsonSchema: JsonSchema = convert(AlphaScalarLeafTaskExpressionsSchema);
