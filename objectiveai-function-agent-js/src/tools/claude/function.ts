import { tool } from "@anthropic-ai/claude-agent-sdk";
import { checkFunction, readFunction, readFunctionSchema } from "../function";
import { resultFromResult, textResult } from "./util";
import { formatZodSchema } from "../schema";
import { ToolState } from "./toolState";

export function makeReadFunction(state: ToolState) {
  return tool(
    "ReadFunction",
    "Read the full Function",
    {},
    async () => resultFromResult(readFunction()),
  );
}

export function makeReadFunctionSchema(state: ToolState) {
  return tool(
    "ReadFunctionSchema",
    "Read the schema for Function",
    {},
    async () => textResult(formatZodSchema(readFunctionSchema())),
  );
}

export function makeCheckFunction(state: ToolState) {
  return tool(
    "CheckFunction",
    "Validate the full Function",
    {},
    async () => resultFromResult(checkFunction()),
  );
}
