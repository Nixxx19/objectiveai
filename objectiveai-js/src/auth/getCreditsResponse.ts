import { z } from "zod";

export const AuthGetCreditsResponseSchema = z.object({
  credits: z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()]).describe("The current available credit balance."),
  total_credits_purchased: z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()]).describe("The total amount of credits ever purchased."),
  total_credits_used: z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()]).describe("The total amount of credits consumed by API usage."),
}).describe("Response containing the user's credit balance information.\n\nCredits are the billing unit for ObjectiveAI. This response provides\na complete view of the user's credit status.").meta({ title: "auth.GetCreditsResponse" });
export type AuthGetCreditsResponse = z.infer<typeof AuthGetCreditsResponseSchema>;
