import { z } from "zod";

export const FunctionsExpressionOneOrManyStringSchema = z.union([z.string().describe("A single value."), z.array(z.string()).describe("Multiple values (from array expressions).")]).describe("Result of an expression that may produce one or many values.").meta({ title: "functions.expression.OneOrMany.string" });
export type FunctionsExpressionOneOrManyString = z.infer<typeof FunctionsExpressionOneOrManyStringSchema>;
