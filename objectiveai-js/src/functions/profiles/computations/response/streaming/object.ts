import { z } from "zod";

export const FunctionsProfilesComputationsResponseStreamingObjectSchema = z.literal("function.profile.computation.chunk").meta({ title: "functions.profiles.computations.response.streaming.Object" });
export type FunctionsProfilesComputationsResponseStreamingObject = z.infer<typeof FunctionsProfilesComputationsResponseStreamingObjectSchema>;
