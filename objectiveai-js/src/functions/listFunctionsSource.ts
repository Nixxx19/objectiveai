import { z } from "zod";

export const FunctionsListFunctionsSourceSchema = z.enum(["all","mock","filesystem","objectiveai"]).describe("Source filter for listing functions.").meta({ title: "functions.ListFunctionsSource" });
export type FunctionsListFunctionsSource = z.infer<typeof FunctionsListFunctionsSourceSchema>;
