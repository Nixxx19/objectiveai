import z from "zod";
import { convert, type JsonSchema } from "../json_schema";

export const RemoteSchema = z.enum(["github", "filesystem"]).describe(
  "The remote source hosting the repository.",
);
export type Remote = z.infer<typeof RemoteSchema>;
export const RemoteJsonSchema: JsonSchema = convert(RemoteSchema);
