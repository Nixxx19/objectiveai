import z from "zod";
import { convert, type JsonSchema } from "../../../json_schema";

export const FunctionToolChoiceFunctionSchema = z
  .object({
    name: z
      .string()
      .describe("The name of the function the assistant will call."),
  })
  .meta({ title: "FunctionToolChoiceFunction" });
export type FunctionToolChoiceFunction = z.infer<
  typeof FunctionToolChoiceFunctionSchema
>;
export const FunctionToolChoiceFunctionJsonSchema: JsonSchema = convert(
  FunctionToolChoiceFunctionSchema,
);

export const FunctionToolChoiceSchema = z
  .object({
    type: z.literal("function"),
    function: FunctionToolChoiceFunctionSchema,
  })
  .describe("Specify a function tool for the assistant to call.")
  .meta({ title: "ToolChoiceFunction" });
export type FunctionToolChoice = z.infer<typeof FunctionToolChoiceSchema>;
export const FunctionToolChoiceJsonSchema: JsonSchema = convert(
  FunctionToolChoiceSchema,
);

export const ToolChoiceSchema = z
  .union([
    z.literal("none"),
    z.literal("auto"),
    z.literal("required"),
    FunctionToolChoiceSchema,
  ])
  .describe("Specifies tool call behavior for the assistant.")
  .meta({ title: "ToolChoice" });
export type ToolChoice = z.infer<typeof ToolChoiceSchema>;
export const ToolChoiceJsonSchema: JsonSchema = convert(ToolChoiceSchema);
