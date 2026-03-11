import { z } from "zod";
import { FunctionsExpressionTaskOutputOwnedSchema } from "./taskOutputOwned";
import { FunctionsExpressionTaskOutputRefSchema } from "./taskOutputRef";

export const FunctionsExpressionTaskOutputSchema = z.union([FunctionsExpressionTaskOutputOwnedSchema.describe("Owned version."), FunctionsExpressionTaskOutputRefSchema.describe("Borrowed version.")]).describe("Output from an executed task.").meta({ title: "functions.expression.TaskOutput" });
export type FunctionsExpressionTaskOutput = z.infer<typeof FunctionsExpressionTaskOutputSchema>;
