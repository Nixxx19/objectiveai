import { query } from "@anthropic-ai/claude-agent-sdk";
import { existsSync, readFileSync } from "fs";
import { LogFn } from "../../agentOptions";
import { promptResources } from "../promptResources";
import { getSlashCwd, getBackslashCwd } from "../../util";

// Step 4 - Create function/type.json (if needed)
export async function createFunctionTypeJson(
  log: LogFn,
  sessionId?: string,
): Promise<string | undefined> {
  // Check if function/type.json exists and is a valid function type
  const functionTypePath = "function/type.json";
  const functionTypeValid = () => {
    if (!existsSync(functionTypePath)) {
      return false;
    }
    let content = readFileSync(functionTypePath, "utf-8").trim();
    if (!content || content === "null") {
      return false;
    }
    // Remove surrounding quotes if both present
    if (content.startsWith('"') && content.endsWith('"')) {
      content = content.slice(1, -1);
    }
    return content === "scalar.function" || content === "vector.function";
  };

  // Query
  const slashCwd = getSlashCwd();
  const backslashCwd = getBackslashCwd();

  if (!functionTypeValid()) {
    const stream = query({
      prompt:
        promptResources(["OBJECTIVEAI_INDEX.md", "SPEC.md"]) +
        'Create function/type.json specifying the function type ("scalar.function" or "vector.function").',
      options: {
        allowedTools: [
          "Bash(ls*)",
          "Bash(cd)",
          "Bash(cat)",
          "Bash(diff)",
          "Glob",
          "Grep",
          "Read",
          "WebFetch",
          "WebSearch",
          "Edit(function/type.json)",
          "Edit(./function/type.json)",
          `Edit(${slashCwd}/function/type.json)`,
          `Edit(${backslashCwd}\\function\\type.json)`,
          "Write(function/type.json)",
          "Write(./function/type.json)",
          `Write(${slashCwd}/function/type.json)`,
          `Write(${backslashCwd}\\function\\type.json)`,
        ],
        disallowedTools: ["AskUserQuestion"],
        permissionMode: "dontAsk",
        resume: sessionId,
      },
    });

    // Agent creates function/type.json
    for await (const message of stream) {
      if (message.type === "system" && message.subtype === "init") {
        sessionId = message.session_id;
      }
      log(message);
    }
  }

  let retry = 1;
  while (!functionTypeValid()) {
    if (retry > 10) {
      throw new Error(
        "function/type.json is invalid after createFunctionTypeJson phase",
      );
    }
    const stream = query({
      prompt:
        "function/type.json is invalid after your createFunctionTypeJson phase." +
        ' Create function/type.json specifying the function type ("scalar.function" or "vector.function") based on SPEC.md.',
      options: {
        allowedTools: [
          "Bash(ls*)",
          "Bash(cd)",
          "Bash(cat)",
          "Bash(diff)",
          "Glob",
          "Grep",
          "Read",
          "WebFetch",
          "WebSearch",
          "Edit(function/type.json)",
          "Edit(./function/type.json)",
          `Edit(${slashCwd}/function/type.json)`,
          `Edit(${backslashCwd}\\function\\type.json)`,
          "Write(function/type.json)",
          "Write(./function/type.json)",
          `Write(${slashCwd}/function/type.json)`,
          `Write(${backslashCwd}\\function\\type.json)`,
        ],
        disallowedTools: ["AskUserQuestion"],
        permissionMode: "dontAsk",
        resume: sessionId,
      },
    });

    // Agent retries function/type.json creation
    for await (const message of stream) {
      if (message.type === "system" && message.subtype === "init") {
        sessionId = message.session_id;
      }
      log(message);
    }
    retry += 1;
  }

  // Return session ID for resuming
  return sessionId;
}
