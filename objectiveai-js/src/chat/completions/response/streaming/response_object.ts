import z from "zod";
import { convert, type JsonSchema } from "../../../../json_schema";

export const ResponseObjectSchema = z.literal("chat.completion.chunk");
export type ResponseObject = z.infer<typeof ResponseObjectSchema>;
export const ResponseObjectJsonSchema: JsonSchema = convert(ResponseObjectSchema);
