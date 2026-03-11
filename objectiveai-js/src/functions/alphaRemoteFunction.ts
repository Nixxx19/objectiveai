import { z } from "zod";
import { FunctionsAlphaScalarRemoteFunctionSchema } from "./alpha_scalar/remoteFunction";
import { FunctionsAlphaVectorRemoteFunctionSchema } from "./alpha_vector/remoteFunction";

export const FunctionsAlphaRemoteFunctionSchema = z.union([FunctionsAlphaScalarRemoteFunctionSchema, FunctionsAlphaVectorRemoteFunctionSchema]).meta({ title: "functions.AlphaRemoteFunction" });
export type FunctionsAlphaRemoteFunction = z.infer<typeof FunctionsAlphaRemoteFunctionSchema>;
