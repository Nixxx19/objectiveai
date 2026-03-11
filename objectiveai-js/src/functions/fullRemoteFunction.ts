import { z } from "zod";
import { FunctionsAlphaRemoteFunctionSchema } from "./alphaRemoteFunction";
import { FunctionsRemoteFunctionSchema } from "./remoteFunction";

export const FunctionsFullRemoteFunctionSchema = z.union([FunctionsAlphaRemoteFunctionSchema, FunctionsRemoteFunctionSchema]).meta({ title: "functions.FullRemoteFunction" });
export type FunctionsFullRemoteFunction = z.infer<typeof FunctionsFullRemoteFunctionSchema>;
