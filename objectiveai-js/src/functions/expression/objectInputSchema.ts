import { z } from "zod";
import { FunctionsExpressionInputSchemaSchema } from "./inputSchema";

export const FunctionsExpressionObjectInputSchemaSchema = z.object({
  description: z.string().nullable().describe("Human-readable description of the object.").optional(),
  properties: z.record(z.string(), FunctionsExpressionInputSchemaSchema).describe("Schema for each property in the object."),
  required: z.array(z.string()).nullable().describe("List of property names that must be present.").optional(),
}).describe("Schema for an object input with named properties.").meta({ title: "functions.expression.ObjectInputSchema" });
export type FunctionsExpressionObjectInputSchema = z.infer<typeof FunctionsExpressionObjectInputSchemaSchema>;
