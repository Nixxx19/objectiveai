import z from "zod";
import { convert, type JsonSchema } from "../../../jsonSchema";

export const OpenRouterByokApiKeySchema = z.object({
  api_key: z.string().describe("The OpenRouter API key."),
});
export type OpenRouterByokApiKey = z.infer<typeof OpenRouterByokApiKeySchema>;
export const OpenRouterByokApiKeyJsonSchema: JsonSchema = convert(
  OpenRouterByokApiKeySchema,
);
