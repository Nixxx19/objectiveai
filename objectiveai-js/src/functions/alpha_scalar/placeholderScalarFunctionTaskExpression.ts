import { z } from "zod";
import { FunctionsExpressionExpressionSchema } from "../expression/expression";
import { FunctionsExpressionObjectInputSchemaSchema } from "../expression/objectInputSchema";

export const FunctionsAlphaScalarPlaceholderScalarFunctionTaskExpressionSchema = z.object({
  depth: z.number().int().min(0).meta({ format: "uint64" }),
  min_branch_width: z.number().int().min(0).meta({ format: "uint64" }),
  max_branch_width: z.number().int().min(0).meta({ format: "uint64" }),
  min_leaf_width: z.number().int().min(0).meta({ format: "uint64" }),
  max_leaf_width: z.number().int().min(0).meta({ format: "uint64" }),
  name: z.string(),
  spec: z.string(),
  input_schema: FunctionsExpressionObjectInputSchemaSchema,
  skip: FunctionsExpressionExpressionSchema.nullable().optional(),
  input: FunctionsExpressionExpressionSchema,
}).meta({ title: "functions.alpha_scalar.PlaceholderScalarFunctionTaskExpression" });
export type FunctionsAlphaScalarPlaceholderScalarFunctionTaskExpression = z.infer<typeof FunctionsAlphaScalarPlaceholderScalarFunctionTaskExpressionSchema>;
