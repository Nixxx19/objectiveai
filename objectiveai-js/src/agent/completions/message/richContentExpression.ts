import { z } from "zod";
import { FunctionsExpressionWithExpressionAgentCompletionsMessageRichContentPartExpressionSchema } from "../../../functions/expression/withExpression";

export const AgentCompletionsMessageRichContentExpressionSchema = z.union([z.string().describe("Plain text content."), z.array(z.lazy(() => FunctionsExpressionWithExpressionAgentCompletionsMessageRichContentPartExpressionSchema)).describe("Multi-part content expressions.")]).describe("Expression variant of [`RichContent`] for dynamic content.").meta({ title: "agent.completions.message.RichContentExpression" });
export type AgentCompletionsMessageRichContentExpression = z.infer<typeof AgentCompletionsMessageRichContentExpressionSchema>;
