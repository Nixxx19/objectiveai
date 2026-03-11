import { z } from "zod";
import { FunctionsRemoteSchema } from "../../../remote";

export const FunctionsProfilesComputationsRequestFunctionRemoteRequestPathSchema = z.object({
  fremote: FunctionsRemoteSchema,
  fowner: z.string(),
  frepository: z.string(),
  fcommit: z.string().nullable().optional(),
}).meta({ title: "functions.profiles.computations.request.FunctionRemoteRequestPath" });
export type FunctionsProfilesComputationsRequestFunctionRemoteRequestPath = z.infer<typeof FunctionsProfilesComputationsRequestFunctionRemoteRequestPathSchema>;
