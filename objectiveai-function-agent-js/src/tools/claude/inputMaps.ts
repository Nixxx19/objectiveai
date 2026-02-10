import { tool } from "@anthropic-ai/claude-agent-sdk";
import { resultFromResult, textResult } from "./util";
import {
  appendInputMap,
  checkInputMaps,
  delInputMap,
  delInputMaps,
  editInputMaps,
  readInputMaps,
  readInputMapsSchema,
} from "../function";
import z from "zod";
import { formatZodSchema } from "../schema";
import { ToolState } from "./toolState";

export function makeReadInputMaps(state: ToolState) {
  return tool(
    "ReadInputMaps",
    "Read the Function's `input_maps` field",
    {},
    async () => resultFromResult(readInputMaps()),
  );
}

export function makeReadInputMapsSchema(state: ToolState) {
  return tool(
    "ReadInputMapsSchema",
    "Read the schema for Function `input_maps` field",
    {},
    async () => textResult(formatZodSchema(readInputMapsSchema())),
  );
}

export function makeEditInputMaps(state: ToolState) {
  return tool(
    "EditInputMaps",
    "Edit the Function's `input_maps` field",
    { value: z.unknown().nullable() },
    async ({ value }) => resultFromResult(editInputMaps(value)),
  );
}

export function makeAppendInputMap(state: ToolState) {
  return tool(
    "AppendInputMap",
    "Append an input map to the Function's `input_maps` array",
    { value: z.unknown() },
    async ({ value }) => resultFromResult(appendInputMap(value)),
  );
}

export function makeDelInputMap(state: ToolState) {
  return tool(
    "DelInputMap",
    "Delete an input map at a specific index from the Function's `input_maps` array",
    { index: z.int().nonnegative() },
    async ({ index }) => resultFromResult(delInputMap(index)),
  );
}

export function makeDelInputMaps(state: ToolState) {
  return tool(
    "DelInputMaps",
    "Delete the Function's `input_maps` field",
    {},
    async () => resultFromResult(delInputMaps()),
  );
}

export function makeCheckInputMaps(state: ToolState) {
  return tool(
    "CheckInputMaps",
    "Validate the Function's `input_maps` field",
    {},
    async () => resultFromResult(checkInputMaps()),
  );
}
