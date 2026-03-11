import { z } from "zod";
import { FunctionsExpressionWithExpressionStringSchema } from "../../../functions/expression/withExpression";

export const AgentCompletionsMessageAssistantToolCallFunctionExpressionSchema = z.object({
  name: z.lazy(() => FunctionsExpressionWithExpressionStringSchema).describe("The function name expression."),
  arguments: z.lazy(() => FunctionsExpressionWithExpressionStringSchema).describe("The arguments expression."),
}).describe("Expression variant of [`AssistantToolCallFunction`] for dynamic content.").meta({ title: "agent.completions.message.AssistantToolCallFunctionExpression" });
export type AgentCompletionsMessageAssistantToolCallFunctionExpression = z.infer<typeof AgentCompletionsMessageAssistantToolCallFunctionExpressionSchema>;
