import { z } from "zod";
import { FunctionsExpressionWithExpressionAgentCompletionsMessageRichContentExpressionSchema, FunctionsExpressionWithExpressionNullableStringSchema } from "../../../functions/expression/withExpression";

export const AgentCompletionsMessageUserMessageExpressionSchema = z.object({
  content: z.lazy(() => FunctionsExpressionWithExpressionAgentCompletionsMessageRichContentExpressionSchema).describe("The message content expression."),
  name: z.lazy(() => FunctionsExpressionWithExpressionNullableStringSchema).nullable().describe("Optional name expression.").optional(),
}).describe("Expression variant of [`UserMessage`] for dynamic content.").meta({ title: "agent.completions.message.UserMessageExpression" });
export type AgentCompletionsMessageUserMessageExpression = z.infer<typeof AgentCompletionsMessageUserMessageExpressionSchema>;
