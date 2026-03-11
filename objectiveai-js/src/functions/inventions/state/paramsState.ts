import { z } from "zod";
import { FunctionsInventionsStateAlphaScalarBranchStateSchema } from "./alphaScalarBranchState";
import { FunctionsInventionsStateAlphaScalarLeafStateSchema } from "./alphaScalarLeafState";
import { FunctionsInventionsStateAlphaScalarStateSchema } from "./alphaScalarState";
import { FunctionsInventionsStateAlphaVectorBranchStateSchema } from "./alphaVectorBranchState";
import { FunctionsInventionsStateAlphaVectorLeafStateSchema } from "./alphaVectorLeafState";
import { FunctionsInventionsStateAlphaVectorStateSchema } from "./alphaVectorState";

export const FunctionsInventionsStateParamsStateSchema = z.union([FunctionsInventionsStateAlphaScalarBranchStateSchema.extend({
  type: z.literal("alpha.scalar.branch.function"),
}), FunctionsInventionsStateAlphaScalarLeafStateSchema.extend({
  type: z.literal("alpha.scalar.leaf.function"),
}), FunctionsInventionsStateAlphaVectorBranchStateSchema.extend({
  type: z.literal("alpha.vector.branch.function"),
}), FunctionsInventionsStateAlphaVectorLeafStateSchema.extend({
  type: z.literal("alpha.vector.leaf.function"),
}), FunctionsInventionsStateAlphaScalarStateSchema.extend({
  type: z.literal("alpha.scalar.function"),
}), FunctionsInventionsStateAlphaVectorStateSchema.extend({
  type: z.literal("alpha.vector.function"),
})]).meta({ title: "functions.inventions.state.ParamsState" });
export type FunctionsInventionsStateParamsState = z.infer<typeof FunctionsInventionsStateParamsStateSchema>;
