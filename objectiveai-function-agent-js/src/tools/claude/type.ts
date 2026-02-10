import { tool } from "@anthropic-ai/claude-agent-sdk";
import { resultFromResult, textResult } from "./util";
import { checkType, editType, readType, readTypeSchema } from "../function";
import z from "zod";
import { formatZodSchema } from "../schema";
import { ToolState } from "./toolState";

export function makeReadType(state: ToolState) {
  return tool(
    "ReadType",
    "Read the Function's `type` field",
    {},
    async () => resultFromResult(readType()),
  );
}

export function makeReadTypeSchema(state: ToolState) {
  return tool(
    "ReadTypeSchema",
    "Read the schema for Function `type` field",
    {},
    async () => textResult(formatZodSchema(readTypeSchema())),
  );
}

export function makeEditType(state: ToolState) {
  return tool(
    "EditType",
    "Edit the Function's `type` field",
    { value: z.string() },
    async ({ value }) => resultFromResult(editType(value)),
  );
}

export function makeCheckType(state: ToolState) {
  return tool(
    "CheckType",
    "Validate the Function's `type` field",
    {},
    async () => resultFromResult(checkType()),
  );
}
