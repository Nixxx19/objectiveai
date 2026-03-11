import { z } from "zod";
import { FunctionsExpressionObjectInputSchemaSchema } from "../../expression/objectInputSchema";

export const FunctionsInventionsStateAlphaScalarStateSchema = z.object({
  depth: z.number().int().min(0).meta({ format: "uint64" }),
  min_branch_width: z.number().int().min(0).meta({ format: "uint64" }),
  max_branch_width: z.number().int().min(0).meta({ format: "uint64" }),
  min_leaf_width: z.number().int().min(0).meta({ format: "uint64" }),
  max_leaf_width: z.number().int().min(0).meta({ format: "uint64" }),
  name: z.string(),
  spec: z.string(),
  input_schema: FunctionsExpressionObjectInputSchemaSchema.nullable().optional(),
}).meta({ title: "functions.inventions.state.AlphaScalarState" });
export type FunctionsInventionsStateAlphaScalarState = z.infer<typeof FunctionsInventionsStateAlphaScalarStateSchema>;
