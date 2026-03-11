import { z } from "zod";
import { FunctionsExpressionSpecialSchema } from "./special";

export const FunctionsExpressionExpressionSchema = z.union([z.object({
  JMESPath: z.string(),
}).strict().describe("A JMESPath expression."), z.object({
  Starlark: z.string(),
}).strict().describe("A Starlark expression."), z.object({
  Special: FunctionsExpressionSpecialSchema,
}).strict().describe("A predefined special expression variant.")]).describe("An expression that can be either JMESPath or Starlark.\n\nSerializes as `{\"$jmespath\": \"...\"}` or `{\"$starlark\": \"...\"}` in JSON.\n\n# Examples\n\nJMESPath:\n```json\n{\"$jmespath\": \"input.items[0].name\"}\n```\n\nStarlark:\n```json\n{\"$starlark\": \"input['items'][0]['name']\"}\n```").meta({ title: "functions.expression.Expression" });
export type FunctionsExpressionExpression = z.infer<typeof FunctionsExpressionExpressionSchema>;
