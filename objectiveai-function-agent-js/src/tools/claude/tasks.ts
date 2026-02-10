import { tool } from "@anthropic-ai/claude-agent-sdk";
import { ToolState } from "./toolState";
import { resultFromResult, textResult } from "./util";
import {
  appendTask,
  checkTasks,
  delTask,
  delTasks,
  editTask,
  readTasks,
  readTasksSchema,
  readMessagesSchema,
  readToolsSchema,
  readResponsesSchema,
} from "../function";
import z from "zod";
import { formatZodSchema } from "../schema";

export function makeReadTasks(state: ToolState) {
  return tool(
    "ReadTasks",
    "Read the Function's `tasks` field",
    {},
    async () => resultFromResult(readTasks()),
  );
}

export function makeReadTasksSchema(state: ToolState) {
  return tool(
    "ReadTasksSchema",
    "Read the schema for Function `tasks` field",
    {},
    async () => textResult(formatZodSchema(readTasksSchema())),
  );
}

export function makeAppendTask(state: ToolState) {
  return tool(
    "AppendTask",
    "Append a task to the Function's `tasks` array",
    { value: z.record(z.string(), z.unknown()) },
    async ({ value }) => resultFromResult(appendTask(value)),
  );
}

export function makeEditTask(state: ToolState) {
  return tool(
    "EditTask",
    "Replace a task at a specific index in the Function's `tasks` array",
    {
      index: z.number().int().nonnegative(),
      value: z.record(z.string(), z.unknown()),
    },
    async ({ index, value }) => resultFromResult(editTask(index, value)),
  );
}

export function makeDelTask(state: ToolState) {
  return tool(
    "DelTask",
    "Delete a task at a specific index from the Function's `tasks` array",
    { index: z.int().nonnegative() },
    async ({ index }) => resultFromResult(delTask(index)),
  );
}

export function makeDelTasks(state: ToolState) {
  return tool(
    "DelTasks",
    "Delete all tasks from the Function's `tasks` array",
    {},
    async () => resultFromResult(delTasks()),
  );
}

export function makeCheckTasks(state: ToolState) {
  return tool(
    "CheckTasks",
    "Validate the Function's `tasks` field",
    {},
    async () => resultFromResult(checkTasks()),
  );
}

export function makeReadMessagesExpressionSchema(state: ToolState) {
  return tool(
    "ReadMessagesExpressionSchema",
    "Read the schema for the `messages` field of a vector.completion task",
    {},
    async () => textResult(formatZodSchema(readMessagesSchema())),
  );
}

export function makeReadToolsExpressionSchema(state: ToolState) {
  return tool(
    "ReadToolsExpressionSchema",
    "Read the schema for the `tools` field of a vector.completion task",
    {},
    async () => textResult(formatZodSchema(readToolsSchema())),
  );
}

export function makeReadResponsesExpressionSchema(state: ToolState) {
  return tool(
    "ReadResponsesExpressionSchema",
    "Read the schema for the `responses` field of a vector.completion task",
    {},
    async () => textResult(formatZodSchema(readResponsesSchema())),
  );
}
