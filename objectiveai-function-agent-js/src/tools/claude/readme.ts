import { tool } from "@anthropic-ai/claude-agent-sdk";
import { resultFromResult } from "./util";
import z from "zod";
import { readReadme, writeReadme } from "../markdown";
import { ToolState } from "./toolState";

export function makeReadReadme(state: ToolState) {
  return tool(
    "ReadReadme",
    "Read README.md",
    {},
    async () => resultFromResult(readReadme()),
  );
}

export function makeWriteReadme(state: ToolState) {
  return tool(
    "WriteReadme",
    "Write README.md",
    { content: z.string() },
    async ({ content }) => resultFromResult(writeReadme(content)),
  );
}
