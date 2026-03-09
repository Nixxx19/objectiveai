import z from "zod";
import { convert, type JsonSchema } from "../../../jsonSchema";

export const RoleSchema = z
  .enum(["assistant"])
  .describe("The role of the message author.");
export type Role = z.infer<typeof RoleSchema>;
export const RoleJsonSchema: JsonSchema = convert(RoleSchema);
