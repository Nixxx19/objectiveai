import { EnsembleLlmBaseSchema } from "src/ensemble_llm/ensembleLlm";
import z from "zod";
import { convert, type JsonSchema } from "../../../jsonSchema";

export const ModelSchema = z
  .union([
    z
      .string()
      .describe("The unique identifier for the Ensemble LLM.")
      .meta({ title: "EnsembleLlmId" }),
    EnsembleLlmBaseSchema,
  ])
  .describe(
    "The Ensemble LLM to use for this completion. May be a unique ID or an inline definition."
  );
export type Model = z.infer<typeof ModelSchema>;
export const ModelJsonSchema: JsonSchema = convert(ModelSchema);

export const FallbackModelsSchema = z
  .array(ModelSchema)
  .describe("Fallback Ensemble LLMs to use if the primary Ensemble LLM fails.");
export type FallbackModels = z.infer<typeof FallbackModelsSchema>;
export const FallbackModelsJsonSchema: JsonSchema = convert(FallbackModelsSchema);
