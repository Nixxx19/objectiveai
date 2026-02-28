import z from "zod";
import { MessageSchema } from "./message";
import { FinishReasonSchema } from "../finishReason";
import { LogprobsSchema } from "../logprobs";
import { convert, type JsonSchema } from "../../../../jsonSchema";

export const ChoiceSchema = z
  .object({
    message: MessageSchema,
    finish_reason: FinishReasonSchema,
    index: z
      .uint32()
      .describe("The index of the choice in the list of choices."),
    logprobs: LogprobsSchema.nullable(),
  })
  .describe("A choice in a unary chat completion response.");
export type Choice = z.infer<typeof ChoiceSchema>;
export const ChoiceJsonSchema: JsonSchema = convert(ChoiceSchema);
