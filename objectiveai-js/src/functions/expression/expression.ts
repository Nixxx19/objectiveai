import z from "zod";
import { convert, type JsonSchema } from "../../json_schema";

export const JMESPathExpressionSchema = z
  .object({
    $jmespath: z.string().describe("A JMESPath expression."),
  })
  .strict()
  .describe("A JMESPath expression which evaluates to a value.")
  .meta({ title: "JMESPathExpression" });
export type JMESPathExpression = z.infer<typeof JMESPathExpressionSchema>;
export const JMESPathExpressionJsonSchema: JsonSchema = convert(JMESPathExpressionSchema);

export const StarlarkExpressionSchema = z
  .object({
    $starlark: z.string().describe("A Starlark expression."),
  })
  .strict()
  .describe("A Starlark expression which evaluates to a value.")
  .meta({ title: "StarlarkExpression" });
export type StarlarkExpression = z.infer<typeof StarlarkExpressionSchema>;
export const StarlarkExpressionJsonSchema: JsonSchema = convert(StarlarkExpressionSchema);

export const SpecialExpressionSchema = z
  .object({
    $special: z
      .enum([
        "input",
        "output",
        "l1_normalized_function_output",
        "input_items_output_length",
        "input_items_optional_context_split",
        "input_items_optional_context_merge",
        "vector_completion_scores",
        "vector_completion_scores_weighted_sum",
      ])
      .describe("A predefined special expression variant."),
  })
  .strict()
  .describe("A special predefined expression.")
  .meta({ title: "SpecialExpression" });
export type SpecialExpression = z.infer<typeof SpecialExpressionSchema>;
export const SpecialExpressionJsonSchema: JsonSchema = convert(SpecialExpressionSchema);

export const ExpressionSchema = z
  .union([JMESPathExpressionSchema, StarlarkExpressionSchema, SpecialExpressionSchema])
  .describe("An expression (JMESPath, Starlark, or Special) which evaluates to a value.")
  .meta({ title: "Expression" });
export type Expression = z.infer<typeof ExpressionSchema>;
export const ExpressionJsonSchema: JsonSchema = convert(ExpressionSchema);
