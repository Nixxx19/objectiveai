import z from "zod";
import { convert, type JsonSchema } from "../../../../jsonSchema";

export const ResponseObjectSchema = z.literal("chat.completion");
export type ResponseObject = z.infer<typeof ResponseObjectSchema>;
export const ResponseObjectJsonSchema: JsonSchema = convert(ResponseObjectSchema);
