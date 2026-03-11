import { z } from "zod";
import { FunctionsExpressionInputSchema } from "../../../expression/input";
import { FunctionsProfilesComputationsRequestTargetSchema } from "./target";

export const FunctionsProfilesComputationsRequestDatasetItemSchema = z.object({
  input: FunctionsExpressionInputSchema,
  target: FunctionsProfilesComputationsRequestTargetSchema,
}).meta({ title: "functions.profiles.computations.request.DatasetItem" });
export type FunctionsProfilesComputationsRequestDatasetItem = z.infer<typeof FunctionsProfilesComputationsRequestDatasetItemSchema>;
