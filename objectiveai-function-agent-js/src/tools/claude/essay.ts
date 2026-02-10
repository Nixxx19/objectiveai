import { tool } from "@anthropic-ai/claude-agent-sdk";
import { resultFromResult } from "./util";
import z from "zod";
import { readEssay, writeEssay } from "../markdown";
import { ToolState } from "./toolState";

export function makeReadEssay(state: ToolState) {
  return tool("ReadEssay", "Read ESSAY.md", {}, async () =>
    resultFromResult(readEssay()),
  );
}

export function makeWriteEssay(state: ToolState) {
  return tool(
    "WriteEssay",
    "Write ESSAY.md",
    { content: z.string() },
    async ({ content }) => resultFromResult(writeEssay(content)),
  );
}
