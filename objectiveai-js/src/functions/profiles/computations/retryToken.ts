import { z } from "zod";

export const FunctionsProfilesComputationsRetryTokenSchema = z.array(z.string().nullable()).meta({ title: "functions.profiles.computations.RetryToken" });
export type FunctionsProfilesComputationsRetryToken = z.infer<typeof FunctionsProfilesComputationsRetryTokenSchema>;
