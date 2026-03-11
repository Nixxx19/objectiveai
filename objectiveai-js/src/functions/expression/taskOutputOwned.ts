import { z } from "zod";

export const FunctionsExpressionTaskOutputOwnedSchema = z.union([z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()]).describe("A single scalar score."), z.array(z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()])).describe("A vector of scores."), z.array(z.array(z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()]))).describe("Multiple vectors of scores (from mapped tasks)."), z.unknown().describe("An error occurred during execution.")]).describe("Owned task output variants.").meta({ title: "functions.expression.TaskOutputOwned" });
export type FunctionsExpressionTaskOutputOwned = z.infer<typeof FunctionsExpressionTaskOutputOwnedSchema>;
