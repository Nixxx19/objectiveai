import { z } from "zod";
import { FunctionsInlineFunctionSchema } from "./inlineFunction";
import { FunctionsRemoteFunctionSchema } from "./remoteFunction";

export const FunctionsFunctionSchema = z.union([FunctionsRemoteFunctionSchema.describe("A remote function with metadata (description, schema, etc.)."), FunctionsInlineFunctionSchema.describe("An inline function definition without metadata.")]).describe("A Function definition, either remote or inline.\n\nFunctions are composable scoring pipelines that transform structured input\ninto scores. Each task has an `output` expression that transforms its raw result\ninto a `TaskOutputOwned`. The function's final output is the weighted average of\nall task outputs using profile weights.\n\nUse [`compile_tasks`](Self::compile_tasks) to preview how task expressions resolve\nfor given inputs.").meta({ title: "functions.Function" });
export type FunctionsFunction = z.infer<typeof FunctionsFunctionSchema>;
