import { z } from "zod";
import { FunctionsAlphaInlineFunctionSchema } from "./alphaInlineFunction";
import { FunctionsInlineFunctionSchema } from "./inlineFunction";

export const FunctionsFullInlineFunctionSchema = z.union([FunctionsAlphaInlineFunctionSchema, FunctionsInlineFunctionSchema]).meta({ title: "functions.FullInlineFunction" });
export type FunctionsFullInlineFunction = z.infer<typeof FunctionsFullInlineFunctionSchema>;
