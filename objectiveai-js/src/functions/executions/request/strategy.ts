import z from "zod";
import { convert, type JsonSchema } from "../../../json_schema";

export const StrategyDefaultSchema = z
  .object({
    type: z.literal("default"),
  })
  .describe("Default strategy for function execution.");
export type StrategyDefault = z.infer<typeof StrategyDefaultSchema>;
export const StrategyDefaultJsonSchema: JsonSchema = convert(
  StrategyDefaultSchema,
);

export const StrategySwissSystemSchema = z
  .object({
    type: z.literal("swiss_system"),
    pool: z
      .number()
      .int()
      .positive()
      .optional()
      .nullable()
      .describe("How many vector responses for each execution. Default is 10."),
    rounds: z
      .number()
      .int()
      .positive()
      .optional()
      .nullable()
      .describe("How many sequential rounds of comparison. Default is 3."),
  })
  .describe("Swiss system strategy for vector function execution.");
export type StrategySwissSystem = z.infer<typeof StrategySwissSystemSchema>;
export const StrategySwissSystemJsonSchema: JsonSchema = convert(
  StrategySwissSystemSchema,
);

export const StrategySchema = z
  .discriminatedUnion("type", [
    StrategyDefaultSchema,
    StrategySwissSystemSchema,
  ])
  .describe("Strategy for function execution.")
  .meta({ title: "FunctionExecutionStrategy" });
export type Strategy = z.infer<typeof StrategySchema>;
export const StrategyJsonSchema: JsonSchema = convert(StrategySchema);
