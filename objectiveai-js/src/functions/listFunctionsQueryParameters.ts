import { z } from "zod";
import { FunctionsListFunctionsSourceSchema } from "./listFunctionsSource";

export const FunctionsListFunctionsQueryParametersSchema = z.object({
  source: FunctionsListFunctionsSourceSchema.nullable().describe("Optional source filter for listing functions.").optional(),
}).describe("Query parameters for the list functions endpoint.").meta({ title: "functions.ListFunctionsQueryParameters" });
export type FunctionsListFunctionsQueryParameters = z.infer<typeof FunctionsListFunctionsQueryParametersSchema>;
