import { z } from "zod";
import { PrefixedUuidSchema } from "../prefixedUuid";

export const AuthApiKeyWithMetadataSchema = z.object({
  api_key: PrefixedUuidSchema.describe("The API key itself."),
  created: z.string().meta({ format: "date-time" }).describe("The timestamp when the API key was created (RFC 3339 format)."),
  expires: z.string().meta({ format: "date-time" }).nullable().describe("The timestamp when the API key expires, or `None` if it does not expire.").optional(),
  disabled: z.string().meta({ format: "date-time" }).nullable().describe("The timestamp when the API key was disabled, or `None` if it is active.").optional(),
  name: z.string().describe("The user-provided name of the API key."),
  description: z.string().nullable().describe("The user-provided description of the API key, or `None` if not provided.").optional(),
}).describe("An ObjectiveAI API Key with associated metadata.\n\nThis struct contains the API key itself along with information about\nwhen it was created, when it expires (if ever), whether it has been\ndisabled, and user-provided name and description.").meta({ title: "auth.ApiKeyWithMetadata" });
export type AuthApiKeyWithMetadata = z.infer<typeof AuthApiKeyWithMetadataSchema>;
