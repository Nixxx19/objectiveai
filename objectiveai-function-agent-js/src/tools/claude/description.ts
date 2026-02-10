import { tool } from "@anthropic-ai/claude-agent-sdk";
import { resultFromResult, textResult } from "./util";
import {
  checkDescription,
  editDescription,
  readDescription,
  readDescriptionSchema,
} from "../function";
import z from "zod";
import { formatZodSchema } from "../schema";
import { ToolState } from "./toolState";

export function makeReadDescription(state: ToolState) {
  return tool(
    "ReadDescription",
    "Read the Function's `description` field",
    {},
    async () => resultFromResult(readDescription()),
  );
}

export function makeReadDescriptionSchema(state: ToolState) {
  return tool(
    "ReadDescriptionSchema",
    "Read the schema for Function `description` field",
    {},
    async () => textResult(formatZodSchema(readDescriptionSchema())),
  );
}

export function makeEditDescription(state: ToolState) {
  return tool(
    "EditDescription",
    "Edit the Function's `description` field",
    { value: z.string() },
    async ({ value }) => resultFromResult(editDescription(value)),
  );
}

export function makeCheckDescription(state: ToolState) {
  return tool(
    "CheckDescription",
    "Validate the Function's `description` field",
    {},
    async () => resultFromResult(checkDescription()),
  );
}
