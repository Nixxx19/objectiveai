import { z } from "zod";
import { FunctionsAlphaVectorBranchTaskExpressionSchema } from "../../alpha_vector/branchTaskExpression";
import { FunctionsAlphaVectorExpressionVectorFunctionInputSchemaSchema } from "../../alpha_vector/expression/vectorFunctionInputSchema";

export const FunctionsInventionsStateAlphaVectorBranchStateSchema = z.object({
  depth: z.number().int().min(0).meta({ format: "uint64" }),
  min_branch_width: z.number().int().min(0).meta({ format: "uint64" }),
  max_branch_width: z.number().int().min(0).meta({ format: "uint64" }),
  min_leaf_width: z.number().int().min(0).meta({ format: "uint64" }),
  max_leaf_width: z.number().int().min(0).meta({ format: "uint64" }),
  name: z.string(),
  spec: z.string(),
  essay: z.string().nullable().optional(),
  input_schema: FunctionsAlphaVectorExpressionVectorFunctionInputSchemaSchema.nullable().optional(),
  essay_tasks: z.string().nullable().optional(),
  tasks: z.array(FunctionsAlphaVectorBranchTaskExpressionSchema).nullable().optional(),
  tasks_length: z.number().int().min(0).meta({ format: "uint64" }).nullable().optional(),
  description: z.string().nullable().optional(),
  readme: z.string().nullable().optional(),
}).meta({ title: "functions.inventions.state.AlphaVectorBranchState" });
export type FunctionsInventionsStateAlphaVectorBranchState = z.infer<typeof FunctionsInventionsStateAlphaVectorBranchStateSchema>;
