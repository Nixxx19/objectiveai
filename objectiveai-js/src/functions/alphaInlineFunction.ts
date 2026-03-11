import { z } from "zod";
import { FunctionsAlphaScalarInlineFunctionSchema } from "./alpha_scalar/inlineFunction";
import { FunctionsAlphaVectorInlineFunctionSchema } from "./alpha_vector/inlineFunction";

export const FunctionsAlphaInlineFunctionSchema = z.union([FunctionsAlphaScalarInlineFunctionSchema, FunctionsAlphaVectorInlineFunctionSchema]).meta({ title: "functions.AlphaInlineFunction" });
export type FunctionsAlphaInlineFunction = z.infer<typeof FunctionsAlphaInlineFunctionSchema>;
