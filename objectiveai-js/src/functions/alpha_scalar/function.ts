import z from "zod";
import { ObjectInputSchemaSchema } from "../expression/input.js";
import {
  AlphaScalarBranchTaskExpressionsSchema,
  AlphaScalarLeafTaskExpressionsSchema,
} from "./task.js";
import { convert, type JSONSchema } from "../../json_schema.js";

export const AlphaScalarBranchRemoteFunctionSchema = z
  .object({
    type: z.literal("alpha.scalar.branch.function"),
    description: z
      .string()
      .describe("The description of the alpha scalar branch function."),
    input_schema: ObjectInputSchemaSchema.describe(
      "The input schema for the alpha scalar function.",
    ),
    tasks: AlphaScalarBranchTaskExpressionsSchema,
  })
  .describe('An alpha scalar branch remote function. "function.json"')
  .meta({ title: "AlphaScalarBranchRemoteFunction" });
export type AlphaScalarBranchRemoteFunction = z.infer<typeof AlphaScalarBranchRemoteFunctionSchema>;
export const AlphaScalarBranchRemoteFunctionJsonSchema: JSONSchema = convert(AlphaScalarBranchRemoteFunctionSchema);

export const AlphaScalarLeafRemoteFunctionSchema = z
  .object({
    type: z.literal("alpha.scalar.leaf.function"),
    description: z
      .string()
      .describe("The description of the alpha scalar leaf function."),
    input_schema: ObjectInputSchemaSchema.describe(
      "The input schema for the alpha scalar function.",
    ),
    tasks: AlphaScalarLeafTaskExpressionsSchema,
  })
  .describe('An alpha scalar leaf remote function. "function.json"')
  .meta({ title: "AlphaScalarLeafRemoteFunction" });
export type AlphaScalarLeafRemoteFunction = z.infer<typeof AlphaScalarLeafRemoteFunctionSchema>;
export const AlphaScalarLeafRemoteFunctionJsonSchema: JSONSchema = convert(AlphaScalarLeafRemoteFunctionSchema);

export const AlphaScalarRemoteFunctionSchema = z
  .discriminatedUnion("type", [
    AlphaScalarBranchRemoteFunctionSchema,
    AlphaScalarLeafRemoteFunctionSchema,
  ])
  .describe('An alpha scalar remote function. "function.json"')
  .meta({ title: "AlphaScalarRemoteFunction" });
export type AlphaScalarRemoteFunction = z.infer<typeof AlphaScalarRemoteFunctionSchema>;
export const AlphaScalarRemoteFunctionJsonSchema: JSONSchema = convert(AlphaScalarRemoteFunctionSchema);
