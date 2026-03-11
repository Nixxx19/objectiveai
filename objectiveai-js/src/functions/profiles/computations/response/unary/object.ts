import { z } from "zod";

export const FunctionsProfilesComputationsResponseUnaryObjectSchema = z.literal("function.profile.computation").meta({ title: "functions.profiles.computations.response.unary.Object" });
export type FunctionsProfilesComputationsResponseUnaryObject = z.infer<typeof FunctionsProfilesComputationsResponseUnaryObjectSchema>;
