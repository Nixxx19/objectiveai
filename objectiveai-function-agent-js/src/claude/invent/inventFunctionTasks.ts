import { query } from "@anthropic-ai/claude-agent-sdk";
import { execSync } from "child_process";
import { existsSync, readFileSync } from "fs";
import { AgentOptions, LogFn } from "../../agentOptions";
import {
  getCurrentRevision,
  resetToRevision,
  hasUncommittedChanges,
  hasUntrackedFiles,
  resetAndUpdateSubmodule,
  pushOrCreateUpstream,
} from "../../github";
import { promptResources } from "../promptResources";
import { getNextPlanIndex, getPlanPath } from "../planIndex";
import { createFileLogger } from "../../logging";
import { prepare } from "../prepare";
import { getSlashCwd, getBackslashCwd } from "../../util";

// Main loop for inventing a new function (no issues)
async function inventFunctionTasksLoop(
  log: LogFn,
  sessionId?: string,
): Promise<string | undefined> {
  const nextPlanIndex = getNextPlanIndex();
  const planPath = getPlanPath(nextPlanIndex);

  // Store current revision for potential rollback
  const initialRevision = getCurrentRevision();

  const maxAttempts = 5;
  let attempt = 0;
  let success = false;
  let lastFailureReasons: string[] = [];

  while (attempt < maxAttempts && !success) {
    attempt++;
    log(`Invent loop attempt ${attempt}/${maxAttempts}`);

    // Build the prompt - full on first attempt, short on retry
    let prompt: string;

    if (attempt === 1) {
      prompt = `${promptResources([
        "OBJECTIVEAI_INDEX.md",
        "SPEC.md",
        "ESSAY.md",
        "ESSAY_TASKS.md",
        "function/type.json",
        "function/tasks.json",
        "function/description.json",
        "function/input_schema.json",
        "function/input_maps.json",
        "function/output.json",
        "function/output_length.json",
        "function/input_split.json",
        "function/input_merge.json",
        "github/name.json",
        "github/description.json",
        "inputs.json",
        "serverLog.txt",
        "compiledTasks.json",
        "ts-node build.ts",
        "ts-node commitAndPush.ts <message>",
        "ts-node spawnFunctionAgents.ts <json_array>",
        "ts-node getSubFunctionCommits.ts",
        "ts-node installRustLogs.ts",
      ])}
You are inventing a new ObjectiveAI Function. Your goal is to complete the implementation, add example inputs, ensure all tests pass, and leave the repository in a clean state.

## Important: Schema References. Read schema definitions in objectiveai-js in order to understand what types should be contained by files within function/.

## Phase 1: Planning

Write your implementation plan to \`${planPath}\`. Include:
- The input schema structure and field descriptions
- Whether any input maps are needed for mapped task execution
- What the function definition will look like
- What expressions need to be written
- What test inputs will cover edge cases and diverse scenarios

## Phase 2: Implementation

Create a TODO list and execute each item:

### Task Structure

This function must use **function tasks** (type: \`scalar.function\` or \`vector.function\`). You must create **at least 2 sub-functions** by spawning child agents:

1. Analyze ESSAY_TASKS.md and create a spec for each sub-function describing:
   - What it evaluates (purpose, not implementation details)
   - The input schema it expects
   - Whether it's scalar or vector
   - Key evaluation criteria

2. Run \`ts-node spawnFunctionAgents.ts '<json_array_of_specs>'\`
   - Example: \`ts-node spawnFunctionAgents.ts '["Spec for task 0...", "Spec for task 1..."]'\`

3. Parse the output after \`=== SPAWN_RESULTS ===\` to get \`{owner, repository, commit}\` for each

4. Create function tasks in \`function/tasks.json\` referencing those sub-functions:
   \`\`\`json
   {
     "type": "scalar.function",
     "owner": "<owner>",
     "repository": "<repository>",
     "commit": "<commit>",
     "input": {"$starlark": "..."}
   }
   \`\`\`

5. Handle any errors in the spawn results

**Retrying Failed Sub-Functions**
If a sub-function fails (result contains \`{error: "..."}\`), you can retry specific indices:
- Pass \`null\` for indices that succeeded and should be skipped
- Example: \`ts-node spawnFunctionAgents.ts '[null, "Retry spec for task 1", null]'\`
- This deletes \`sub_functions/1/\` and re-spawns only that agent
- Results will show \`{skipped: true}\` for null indices

**Reading Sub-Functions**
After spawning, sub-functions are available in \`sub_functions/<index>/\`:
- Read their \`function.json\`, \`function/\` files, \`inputs.json\`, etc. to understand what was created
- Each sub-function is a complete ObjectiveAI function repository

**Getting Commit SHAs**
To retrieve the current commit SHA for each sub-function:
- Run \`ts-node getSubFunctionCommits.ts\`
- Parse the output after \`=== SUB_FUNCTION_COMMITS ===\` to get \`{index, owner, repository, commit, path}\` for each
- Use these values to update \`function/tasks.json\` with the correct references

### Function Definition
- Edit files in \`function/\` directory to define the function
- **Use Starlark expressions** (\`{"$starlark": "..."}\`) for most expressions - it's Python-like and more readable
- Only use JMESPath (\`{"$jmespath": "..."}\`) for very simple field access expressions
- Starlark example: \`{"$starlark": "input['items'][0]"}\`
- JMESPath example: \`{"$jmespath": "input.name"}\` (simple field access only)

### Expression Context
Expressions receive a single object with these fields:
- \`input\` - Always present, the function input
- \`map\` - Present in mapped tasks, the current map element
- \`tasks\` - Present in output expressions, array of task results

### Test Inputs
- Edit \`inputs.json\` to add diverse test inputs (minimum 10, maximum 100)
- **Diversity in structure**: Include edge cases like empty arrays, single items, boundary values, missing optional fields, maximum lengths
- **Diversity in intended output**: Cover the full range of expected scores (low, medium, high quality inputs that should produce different outputs)

### Build and Test
- Run \`ts-node build.ts\` to compile function.json and execute tests
- If tests fail, read \`serverLog.txt\` and \`compiledTasks.json\` for error details
- Fix issues and repeat until all tests pass

### Debugging
- Read \`compiledTasks.json\` to see how expressions are compiled for each input
- If expression errors occur, check the Starlark/JMESPath syntax

### Rust Logging (Advanced Debugging)
If you need deeper debugging into the ObjectiveAI runtime:
1. Edit files in the objectiveai submodule to add logging:
   - \`objectiveai/objectiveai-rs/src/\` - Core Rust SDK
   - \`objectiveai/objectiveai-api/src/\` - API server logic
   - \`objectiveai/objectiveai-rs-wasm-js/src/\` - WASM bindings
2. Run \`ts-node installRustLogs.ts\` to rebuild the WASM with your changes
3. Run \`ts-node build.ts\` to test - logs will appear in \`serverLog.txt\`

## Phase 3: Verify SPEC.md Compliance

Before finalizing, verify that everything adheres to SPEC.md:
- Re-read SPEC.md carefully
- Ensure the function definition, inputs, and outputs match what SPEC.md describes
- If anything contradicts SPEC.md, fix it to match the spec
- **SPEC.md is the universal source of truth** - the final product must not contradict it

## Phase 4: Finalize

Once all tests pass and SPEC.md compliance is verified:
- Commit your changes using \`ts-node commitAndPush.ts "<message>"\`
- Ensure there are no uncommitted changes or untracked files

## Important Notes

- **SPEC.md is the universal source of truth** - never contradict it
- **No API key is needed for tests** - tests run against a local server
- **Prefer Starlark over JMESPath** - Starlark is more readable and powerful
- **Only modify function/*.json files when necessary**:
  - If the build fails due to invalid/missing values
  - If a field is undefined and needs to be set
`;
    } else {
      // On retry, send a short message about what failed
      prompt = `Your previous attempt failed:
${lastFailureReasons.map((r) => `- ${r}`).join("\n")}

Please try again. Remember to:
1. Run \`ts-node build.ts\` to compile and test
2. Commit your changes using \`ts-node commitAndPush.ts "<message>"\`
`;
    }

    const slashCwd = getSlashCwd();
    const backslashCwd = getBackslashCwd();

    const stream = query({
      prompt,
      options: {
        allowedTools: [
          "Bash(ls*)",
          "Bash(cd)",
          "Bash(cat)",
          "Bash(diff)",
          "Bash(ts-node build.ts)",
          "Bash(npx ts-node build.ts)",
          `Bash(cd ${slashCwd} && ts-node build.ts)`,
          `Bash(cd ${backslashCwd} && ts-node build.ts)`,
          `Bash(cd ${slashCwd} && npx ts-node build.ts)`,
          `Bash(cd ${backslashCwd} && npx ts-node build.ts)`,
          "Bash(ts-node commitAndPush.ts *)",
          "Bash(npx ts-node commitAndPush.ts *)",
          `Bash(cd ${slashCwd} && ts-node commitAndPush.ts *)`,
          `Bash(cd ${backslashCwd} && ts-node commitAndPush.ts *)`,
          `Bash(cd ${slashCwd} && npx ts-node commitAndPush.ts *)`,
          `Bash(cd ${backslashCwd} && npx ts-node commitAndPush.ts *)`,
          "Bash(ts-node spawnFunctionAgents.ts *)",
          "Bash(npx ts-node spawnFunctionAgents.ts *)",
          `Bash(cd ${slashCwd} && ts-node spawnFunctionAgents.ts *)`,
          `Bash(cd ${backslashCwd} && ts-node spawnFunctionAgents.ts *)`,
          `Bash(cd ${slashCwd} && npx ts-node spawnFunctionAgents.ts *)`,
          `Bash(cd ${backslashCwd} && npx ts-node spawnFunctionAgents.ts *)`,
          "Bash(ts-node getSubFunctionCommits.ts)",
          "Bash(npx ts-node getSubFunctionCommits.ts)",
          `Bash(cd ${slashCwd} && ts-node getSubFunctionCommits.ts)`,
          `Bash(cd ${backslashCwd} && ts-node getSubFunctionCommits.ts)`,
          `Bash(cd ${slashCwd} && npx ts-node getSubFunctionCommits.ts)`,
          `Bash(cd ${backslashCwd} && npx ts-node getSubFunctionCommits.ts)`,
          "Bash(ts-node installRustLogs.ts)",
          "Bash(npx ts-node installRustLogs.ts)",
          `Bash(cd ${slashCwd} && ts-node installRustLogs.ts)`,
          `Bash(cd ${backslashCwd} && ts-node installRustLogs.ts)`,
          `Bash(cd ${slashCwd} && npx ts-node installRustLogs.ts)`,
          `Bash(cd ${backslashCwd} && npx ts-node installRustLogs.ts)`,
          "Glob",
          "Grep",
          "Read",
          "WebFetch",
          "WebSearch",
          "Edit(inputs.json)",
          "Edit(./inputs.json)",
          `Edit(${slashCwd}/inputs.json)`,
          `Edit(${backslashCwd}\\inputs.json)`,
          "Write(inputs.json)",
          "Write(./inputs.json)",
          `Write(${slashCwd}/inputs.json)`,
          `Write(${backslashCwd}\\inputs.json)`,
          "Edit(function/description.json)",
          "Edit(./function/description.json)",
          `Edit(${slashCwd}/function/description.json)`,
          `Edit(${backslashCwd}\\function\\description.json)`,
          "Write(function/description.json)",
          "Write(./function/description.json)",
          `Write(${slashCwd}/function/description.json)`,
          `Write(${backslashCwd}\\function\\description.json)`,
          "Edit(function/input_schema.json)",
          "Edit(./function/input_schema.json)",
          `Edit(${slashCwd}/function/input_schema.json)`,
          `Edit(${backslashCwd}\\function\\input_schema.json)`,
          "Write(function/input_schema.json)",
          "Write(./function/input_schema.json)",
          `Write(${slashCwd}/function/input_schema.json)`,
          `Write(${backslashCwd}\\function\\input_schema.json)`,
          "Edit(function/input_maps.json)",
          "Edit(./function/input_maps.json)",
          `Edit(${slashCwd}/function/input_maps.json)`,
          `Edit(${backslashCwd}\\function\\input_maps.json)`,
          "Write(function/input_maps.json)",
          "Write(./function/input_maps.json)",
          `Write(${slashCwd}/function/input_maps.json)`,
          `Write(${backslashCwd}\\function\\input_maps.json)`,
          "Edit(function/tasks.json)",
          "Edit(./function/tasks.json)",
          `Edit(${slashCwd}/function/tasks.json)`,
          `Edit(${backslashCwd}\\function\\tasks.json)`,
          "Write(function/tasks.json)",
          "Write(./function/tasks.json)",
          `Write(${slashCwd}/function/tasks.json)`,
          `Write(${backslashCwd}\\function\\tasks.json)`,
          "Edit(function/output.json)",
          "Edit(./function/output.json)",
          `Edit(${slashCwd}/function/output.json)`,
          `Edit(${backslashCwd}\\function\\output.json)`,
          "Write(function/output.json)",
          "Write(./function/output.json)",
          `Write(${slashCwd}/function/output.json)`,
          `Write(${backslashCwd}\\function\\output.json)`,
          "Edit(function/output_length.json)",
          "Edit(./function/output_length.json)",
          `Edit(${slashCwd}/function/output_length.json)`,
          `Edit(${backslashCwd}\\function\\output_length.json)`,
          "Write(function/output_length.json)",
          "Write(./function/output_length.json)",
          `Write(${slashCwd}/function/output_length.json)`,
          `Write(${backslashCwd}\\function\\output_length.json)`,
          "Edit(function/input_split.json)",
          "Edit(./function/input_split.json)",
          `Edit(${slashCwd}/function/input_split.json)`,
          `Edit(${backslashCwd}\\function\\input_split.json)`,
          "Write(function/input_split.json)",
          "Write(./function/input_split.json)",
          `Write(${slashCwd}/function/input_split.json)`,
          `Write(${backslashCwd}\\function\\input_split.json)`,
          "Edit(function/input_merge.json)",
          "Edit(./function/input_merge.json)",
          `Edit(${slashCwd}/function/input_merge.json)`,
          `Edit(${backslashCwd}\\function\\input_merge.json)`,
          "Write(function/input_merge.json)",
          "Write(./function/input_merge.json)",
          `Write(${slashCwd}/function/input_merge.json)`,
          `Write(${backslashCwd}\\function\\input_merge.json)`,
          "Edit(github/description.json)",
          "Edit(./github/description.json)",
          `Edit(${slashCwd}/github/description.json)`,
          `Edit(${backslashCwd}\\github\\description.json)`,
          "Write(github/description.json)",
          "Write(./github/description.json)",
          `Write(${slashCwd}/github/description.json)`,
          `Write(${backslashCwd}\\github\\description.json)`,
          "Edit(README.md)",
          "Edit(./README.md)",
          `Edit(${slashCwd}/README.md)`,
          `Edit(${backslashCwd}\\README.md)`,
          "Write(README.md)",
          "Write(./README.md)",
          `Write(${slashCwd}/README.md)`,
          `Write(${backslashCwd}\\README.md)`,
          `Edit(plans/${nextPlanIndex}.md)`,
          `Edit(./plans/${nextPlanIndex}.md)`,
          `Edit(${slashCwd}/plans/${nextPlanIndex}.md)`,
          `Edit(${backslashCwd}\\plans\\${nextPlanIndex}.md)`,
          `Write(plans/${nextPlanIndex}.md)`,
          `Write(./plans/${nextPlanIndex}.md)`,
          `Write(${slashCwd}/plans/${nextPlanIndex}.md)`,
          `Write(${backslashCwd}\\plans\\${nextPlanIndex}.md)`,
          "Edit(./objectiveai/objectiveai-api/src/**)",
          "Edit(objectiveai/objectiveai-api/src/**)",
          `Edit(${slashCwd}/objectiveai/objectiveai-api/src/**)`,
          `Edit(${backslashCwd}\\objectiveai\\objectiveai-api\\src\\**)`,
          "Edit(./objectiveai/objectiveai-rs/src/**)",
          "Edit(objectiveai/objectiveai-rs/src/**)",
          `Edit(${slashCwd}/objectiveai/objectiveai-rs/src/**)`,
          `Edit(${backslashCwd}\\objectiveai\\objectiveai-rs\\src\\**)`,
          "Edit(./objectiveai/objectiveai-rs-wasm-js/src/**)",
          "Edit(objectiveai/objectiveai-rs-wasm-js/src/**)",
          `Edit(${slashCwd}/objectiveai/objectiveai-rs-wasm-js/src/**)`,
          `Edit(${backslashCwd}\\objectiveai\\objectiveai-rs-wasm-js\\src\\**)`,
        ],
        disallowedTools: ["AskUserQuestion"],
        permissionMode: "dontAsk",
        resume: sessionId,
      },
    });

    // Run the assistant
    for await (const message of stream) {
      if (message.type === "system" && message.subtype === "init") {
        sessionId = message.session_id;
      }
      log(message);
    }

    // Validate the assistant's work
    log("Validating assistant's work...");

    // Reset and update objectiveai submodule
    log("Resetting and updating objectiveai submodule...");
    resetAndUpdateSubmodule();

    // Run build (which includes tests)
    log("Running build and tests...");
    let buildSuccess = false;
    try {
      execSync("ts-node build.ts", { stdio: "inherit" });
      buildSuccess = true;
    } catch {
      log("Build or tests failed.");
    }

    // Verify success conditions
    lastFailureReasons = [];

    if (!buildSuccess) {
      lastFailureReasons.push(
        "Build or tests failed. Read serverLog.txt and compiledTasks.json for details.",
      );
      log("Failed: Build or tests failed.");
    }

    const hasChanges = hasUncommittedChanges() || hasUntrackedFiles();
    if (hasChanges) {
      lastFailureReasons.push(
        "There are uncommitted changes or untracked files. Commit them with ts-node commitAndPush.ts <message>.",
      );
      log("Failed: There are uncommitted changes or untracked files.");
    }

    const descriptionPath = "github/description.json";
    const hasDescription = (() => {
      if (!existsSync(descriptionPath)) return false;
      let content = readFileSync(descriptionPath, "utf-8").trim();
      if (!content || content === "null") return false;
      if (content.startsWith('"') && content.endsWith('"')) {
        content = content.slice(1, -1);
      }
      return content.length > 0;
    })();
    if (!hasDescription) {
      lastFailureReasons.push(
        "github/description.json is empty. Write a description for the GitHub repository.",
      );
      log("Failed: github/description.json is empty.");
    }

    const readmePath = "README.md";
    const hasReadme =
      existsSync(readmePath) &&
      readFileSync(readmePath, "utf-8").trim().length > 0;
    if (!hasReadme) {
      lastFailureReasons.push(
        "README.md is empty. Write a README for the repository.",
      );
      log("Failed: README.md is empty.");
    }

    if (lastFailureReasons.length === 0) {
      success = true;
      log("Success: All conditions met.");
    }
  }

  // If all attempts failed, reset to initial revision
  if (!success) {
    log("All attempts failed. Resetting to initial revision.");
    resetToRevision(initialRevision);
    throw new Error("Invent loop failed after maximum attempts.");
  }

  // Push the commits (creating upstream repository if needed)
  log("Pushing commits...");
  pushOrCreateUpstream();

  return sessionId;
}

// Main entry point for inventing a new function
export async function inventFunctionTasks(
  options: AgentOptions = {},
): Promise<void> {
  const log = options.log ?? createFileLogger().log;

  // Run preparation (init + steps 1-8)
  const sessionId = await prepare({ ...options, log });

  // Run invent loop
  log("=== Invent Loop: Creating new function ===");
  await inventFunctionTasksLoop(log, sessionId);

  log("=== ObjectiveAI Function invention complete ===");
}
