import { z } from "zod";
import { AuthListApiKeyItemSchema } from "./listApiKeyItem";

export const AuthListApiKeyResponseSchema = z.object({
  data: z.array(AuthListApiKeyItemSchema).describe("The list of API keys with their metadata and usage costs."),
}).describe("Response containing a list of API keys.").meta({ title: "auth.ListApiKeyResponse" });
export type AuthListApiKeyResponse = z.infer<typeof AuthListApiKeyResponseSchema>;
