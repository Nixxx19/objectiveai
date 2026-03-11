import { z } from "zod";
import { FunctionsAlphaScalarBranchTaskExpressionSchema } from "../../alpha_scalar/branchTaskExpression";
import { FunctionsExpressionObjectInputSchemaSchema } from "../../expression/objectInputSchema";

export const FunctionsInventionsStateAlphaScalarBranchStateSchema = z.object({
  depth: z.number().int().min(0).meta({ format: "uint64" }),
  min_branch_width: z.number().int().min(0).meta({ format: "uint64" }),
  max_branch_width: z.number().int().min(0).meta({ format: "uint64" }),
  min_leaf_width: z.number().int().min(0).meta({ format: "uint64" }),
  max_leaf_width: z.number().int().min(0).meta({ format: "uint64" }),
  name: z.string(),
  spec: z.string(),
  essay: z.string().nullable().optional(),
  input_schema: FunctionsExpressionObjectInputSchemaSchema.nullable().optional(),
  essay_tasks: z.string().nullable().optional(),
  tasks: z.array(FunctionsAlphaScalarBranchTaskExpressionSchema).nullable().optional(),
  tasks_length: z.number().int().min(0).meta({ format: "uint64" }).nullable().optional(),
  description: z.string().nullable().optional(),
  readme: z.string().nullable().optional(),
}).meta({ title: "functions.inventions.state.AlphaScalarBranchState" });
export type FunctionsInventionsStateAlphaScalarBranchState = z.infer<typeof FunctionsInventionsStateAlphaScalarBranchStateSchema>;
