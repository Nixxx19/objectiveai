import z from "zod";
import { AlphaVectorFunctionInputSchemaSchema } from "./expression/input.js";
import {
  AlphaVectorBranchTaskExpressionsSchema,
  AlphaVectorLeafTaskExpressionsSchema,
} from "./task.js";
import { convert, type JSONSchema } from "../../json_schema.js";

export const AlphaVectorBranchRemoteFunctionSchema = z
  .object({
    type: z.literal("alpha.vector.branch.function"),
    description: z
      .string()
      .describe("The description of the alpha vector branch function."),
    input_schema: AlphaVectorFunctionInputSchemaSchema.describe(
      "The input schema for the alpha vector function.",
    ),
    tasks: AlphaVectorBranchTaskExpressionsSchema,
  })
  .describe('An alpha vector branch remote function. "function.json"')
  .meta({ title: "AlphaVectorBranchRemoteFunction" });
export type AlphaVectorBranchRemoteFunction = z.infer<typeof AlphaVectorBranchRemoteFunctionSchema>;
export const AlphaVectorBranchRemoteFunctionJsonSchema: JSONSchema = convert(AlphaVectorBranchRemoteFunctionSchema);

export const AlphaVectorLeafRemoteFunctionSchema = z
  .object({
    type: z.literal("alpha.vector.leaf.function"),
    description: z
      .string()
      .describe("The description of the alpha vector leaf function."),
    input_schema: AlphaVectorFunctionInputSchemaSchema.describe(
      "The input schema for the alpha vector function.",
    ),
    tasks: AlphaVectorLeafTaskExpressionsSchema,
  })
  .describe('An alpha vector leaf remote function. "function.json"')
  .meta({ title: "AlphaVectorLeafRemoteFunction" });
export type AlphaVectorLeafRemoteFunction = z.infer<typeof AlphaVectorLeafRemoteFunctionSchema>;
export const AlphaVectorLeafRemoteFunctionJsonSchema: JSONSchema = convert(AlphaVectorLeafRemoteFunctionSchema);

export const AlphaVectorRemoteFunctionSchema = z
  .discriminatedUnion("type", [
    AlphaVectorBranchRemoteFunctionSchema,
    AlphaVectorLeafRemoteFunctionSchema,
  ])
  .describe('An alpha vector remote function. "function.json"')
  .meta({ title: "AlphaVectorRemoteFunction" });
export type AlphaVectorRemoteFunction = z.infer<typeof AlphaVectorRemoteFunctionSchema>;
export const AlphaVectorRemoteFunctionJsonSchema: JSONSchema = convert(AlphaVectorRemoteFunctionSchema);
