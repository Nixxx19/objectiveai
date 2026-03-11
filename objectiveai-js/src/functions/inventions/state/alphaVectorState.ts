import { z } from "zod";
import { FunctionsAlphaVectorExpressionVectorFunctionInputSchemaSchema } from "../../alpha_vector/expression/vectorFunctionInputSchema";

export const FunctionsInventionsStateAlphaVectorStateSchema = z.object({
  depth: z.number().int().min(0).meta({ format: "uint64" }),
  min_branch_width: z.number().int().min(0).meta({ format: "uint64" }),
  max_branch_width: z.number().int().min(0).meta({ format: "uint64" }),
  min_leaf_width: z.number().int().min(0).meta({ format: "uint64" }),
  max_leaf_width: z.number().int().min(0).meta({ format: "uint64" }),
  name: z.string(),
  spec: z.string(),
  input_schema: FunctionsAlphaVectorExpressionVectorFunctionInputSchemaSchema.nullable().optional(),
}).meta({ title: "functions.inventions.state.AlphaVectorState" });
export type FunctionsInventionsStateAlphaVectorState = z.infer<typeof FunctionsInventionsStateAlphaVectorStateSchema>;
