import { tool } from "@anthropic-ai/claude-agent-sdk";
import { resultFromResult, textResult } from "./util";
import {
  checkOutputLength,
  delOutputLength,
  editOutputLength,
  readOutputLength,
  readOutputLengthSchema,
} from "../function";
import z from "zod";
import { formatZodSchema } from "../schema";
import { ToolState } from "./toolState";

export function makeReadOutputLength(state: ToolState) {
  return tool(
    "ReadOutputLength",
    "Read the Function's `output_length` field",
    {},
    async () => resultFromResult(readOutputLength()),
  );
}

export function makeReadOutputLengthSchema(state: ToolState) {
  return tool(
    "ReadOutputLengthSchema",
    "Read the schema for Function `output_length` field",
    {},
    async () => textResult(formatZodSchema(readOutputLengthSchema())),
  );
}

export function makeEditOutputLength(state: ToolState) {
  return tool(
    "EditOutputLength",
    "Edit the Function's `output_length` field",
    { value: z.unknown().nullable() },
    async ({ value }) => resultFromResult(editOutputLength(value)),
  );
}

export function makeDelOutputLength(state: ToolState) {
  return tool(
    "DelOutputLength",
    "Delete the Function's `output_length` field",
    {},
    async () => resultFromResult(delOutputLength()),
  );
}

export function makeCheckOutputLength(state: ToolState) {
  return tool(
    "CheckOutputLength",
    "Validate the Function's `output_length` field",
    {},
    async () => resultFromResult(checkOutputLength()),
  );
}
