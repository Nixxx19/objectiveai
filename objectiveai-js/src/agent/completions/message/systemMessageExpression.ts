import { z } from "zod";
import { FunctionsExpressionWithExpressionAgentCompletionsMessageSimpleContentExpressionSchema, FunctionsExpressionWithExpressionNullableStringSchema } from "../../../functions/expression/withExpression";

export const AgentCompletionsMessageSystemMessageExpressionSchema = z.object({
  content: z.lazy(() => FunctionsExpressionWithExpressionAgentCompletionsMessageSimpleContentExpressionSchema).describe("The message content expression."),
  name: z.lazy(() => FunctionsExpressionWithExpressionNullableStringSchema).nullable().describe("Optional name expression.").optional(),
}).describe("Expression variant of [`SystemMessage`] for dynamic content.").meta({ title: "agent.completions.message.SystemMessageExpression" });
export type AgentCompletionsMessageSystemMessageExpression = z.infer<typeof AgentCompletionsMessageSystemMessageExpressionSchema>;
