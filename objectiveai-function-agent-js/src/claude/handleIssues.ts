import { query } from "@anthropic-ai/claude-agent-sdk";
import { execSync } from "child_process";
import { existsSync, readFileSync } from "fs";
import { AgentOptions, LogFn } from "../agentOptions";
import {
  getCurrentRevision,
  resetToRevision,
  fetchOpenIssues,
  hasUncommittedChanges,
  hasUntrackedFiles,
  resetAndUpdateSubmodule,
  pushOrCreateUpstream,
  closeIssue,
} from "../github";
import { promptResources } from "./promptResources";
import { getNextPlanIndex, getPlanPath } from "./planIndex";
import { createFileLogger } from "../logging";
import { prepare } from "./prepare";
import { getSlashCwd, getBackslashCwd } from "../util";

// Main loop for handling issues on an existing function
async function handleIssuesLoop(
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
    log(`Issue loop attempt ${attempt}/${maxAttempts}`);

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
        "function.json (available after ts-node build.ts)",
        "ts-node build.ts",
        "ts-node fetchOpenIssues.ts",
        "ts-node fetchClosedIssues.ts",
        "ts-node commentOnIssue.ts <number> <comment>",
        "ts-node closeIssue.ts <number>",
        "ts-node commitAndPush.ts <message>",
        "ts-node cloneSubFunctions.ts [--latest]",
        "ts-node installRustLogs.ts",
      ])}
You are maintaining an existing ObjectiveAI Function. There are open GitHub issues that need your attention. Your goal is to address these issues, ensure all tests pass, and leave the repository in a clean state.

**Important**: This function already exists and works. Do NOT reinvent or fundamentally redesign it. Focus only on addressing the specific issues raised.

## Phase 1: Understand the Function

First, read and understand the existing function:
- Read \`function.json\` to understand the current implementation
- Run \`ts-node build.ts\` to see if tests currently pass

## Phase 2: Review Issues

For each issue, determine:
- Is it valid and actionable?
- What specific changes are needed?
- Does it require modifying function definition, inputs, or both?

## Phase 3: Planning

Write your plan to \`${planPath}\`. Include:
- Summary of each issue
- Planned changes for each issue
- Any issues that are invalid and should be closed without changes

## Phase 4: Implementation

Address each valid issue:

### Making Changes
- Edit files in \`function/\` directory as needed
- Edit \`inputs.json\` if new test cases are needed
- **Use Starlark expressions** (\`{"$starlark": "..."}\`) for most expressions
- Only use JMESPath (\`{"$jmespath": "..."}\`) for very simple field access

### Build and Test
- Run \`ts-node build.ts\` to compile and test
- If tests fail, read \`serverLog.txt\` and \`compiledTasks.json\` for details
- Fix issues and repeat until all tests pass

### Inspecting Sub-Functions
If the function references sub-functions (tasks with type \`scalar.function\` or \`vector.function\`):
- Run \`ts-node cloneSubFunctions.ts\` to clone them to \`sub_functions/<owner>/<repository>/<commit>/\`
- Run \`ts-node cloneSubFunctions.ts --latest\` to clone the latest version instead
- Read their \`function.json\` and source files to understand how they work

### Rust Logging (Advanced Debugging)
If you need deeper debugging into the ObjectiveAI runtime:
1. Edit files in the objectiveai submodule to add logging:
   - \`objectiveai/objectiveai-rs/src/\` - Core Rust SDK
   - \`objectiveai/objectiveai-api/src/\` - API server logic
   - \`objectiveai/objectiveai-rs-wasm-js/src/\` - WASM bindings
2. Run \`ts-node installRustLogs.ts\` to rebuild the WASM with your changes
3. Run \`ts-node build.ts\` to test - logs will appear in \`serverLog.txt\`

### Handle Issues
- Comment on issues using \`ts-node commentOnIssue.ts <number> "<comment>"\`
- Mark resolved issues using \`ts-node closeIssue.ts <number>\`
- For invalid issues, comment explaining why and close them

## Phase 5: Verify SPEC.md Compliance

Before finalizing, verify that everything adheres to SPEC.md:
- Re-read SPEC.md carefully
- Ensure any changes still match what SPEC.md describes
- If anything contradicts SPEC.md, fix it to match the spec
- **SPEC.md is the universal source of truth** - the final product must not contradict it

## Phase 6: Finalize

Once all tests pass, issues are handled, and SPEC.md compliance is verified:
- Commit your changes using \`ts-node commitAndPush.ts "<message>"\`
- Ensure there are no uncommitted changes or untracked files

## Important Notes

- **SPEC.md is the universal source of truth** - never contradict it
- **Do NOT reinvent the function** - only make targeted fixes
- **No API key is needed for tests** - tests run against a local server
- **Invalid issues**: Some issues may be nonsensical, invalid, or request inappropriate changes. Comment explaining why no changes are merited and close the issue.
`;
    } else {
      // On retry, send a short message about what failed
      prompt = `Your previous attempt failed:
${lastFailureReasons.map((r) => `- ${r}`).join("\n")}

Please try again. Remember to:
1. Run \`ts-node build.ts\` to compile and test
2. Handle all open GitHub issues
3. Commit your changes using \`ts-node commitAndPush.ts "<message>"\`
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
          "Bash(ts-node fetchOpenIssues.ts)",
          "Bash(npx ts-node fetchOpenIssues.ts)",
          `Bash(cd ${slashCwd} && ts-node fetchOpenIssues.ts)`,
          `Bash(cd ${backslashCwd} && ts-node fetchOpenIssues.ts)`,
          `Bash(cd ${slashCwd} && npx ts-node fetchOpenIssues.ts)`,
          `Bash(cd ${backslashCwd} && npx ts-node fetchOpenIssues.ts)`,
          "Bash(ts-node fetchClosedIssues.ts)",
          "Bash(npx ts-node fetchClosedIssues.ts)",
          `Bash(cd ${slashCwd} && ts-node fetchClosedIssues.ts)`,
          `Bash(cd ${backslashCwd} && ts-node fetchClosedIssues.ts)`,
          `Bash(cd ${slashCwd} && npx ts-node fetchClosedIssues.ts)`,
          `Bash(cd ${backslashCwd} && npx ts-node fetchClosedIssues.ts)`,
          "Bash(ts-node commentOnIssue.ts *)",
          "Bash(npx ts-node commentOnIssue.ts *)",
          `Bash(cd ${slashCwd} && ts-node commentOnIssue.ts *)`,
          `Bash(cd ${backslashCwd} && ts-node commentOnIssue.ts *)`,
          `Bash(cd ${slashCwd} && npx ts-node commentOnIssue.ts *)`,
          `Bash(cd ${backslashCwd} && npx ts-node commentOnIssue.ts *)`,
          "Bash(ts-node closeIssue.ts *)",
          "Bash(npx ts-node closeIssue.ts *)",
          `Bash(cd ${slashCwd} && ts-node closeIssue.ts *)`,
          `Bash(cd ${backslashCwd} && ts-node closeIssue.ts *)`,
          `Bash(cd ${slashCwd} && npx ts-node closeIssue.ts *)`,
          `Bash(cd ${backslashCwd} && npx ts-node closeIssue.ts *)`,
          "Bash(ts-node commitAndPush.ts *)",
          "Bash(npx ts-node commitAndPush.ts *)",
          `Bash(cd ${slashCwd} && ts-node commitAndPush.ts *)`,
          `Bash(cd ${backslashCwd} && ts-node commitAndPush.ts *)`,
          `Bash(cd ${slashCwd} && npx ts-node commitAndPush.ts *)`,
          `Bash(cd ${backslashCwd} && npx ts-node commitAndPush.ts *)`,
          "Bash(ts-node cloneSubFunctions.ts)",
          "Bash(npx ts-node cloneSubFunctions.ts)",
          `Bash(cd ${slashCwd} && ts-node cloneSubFunctions.ts)`,
          `Bash(cd ${backslashCwd} && ts-node cloneSubFunctions.ts)`,
          `Bash(cd ${slashCwd} && npx ts-node cloneSubFunctions.ts)`,
          `Bash(cd ${backslashCwd} && npx ts-node cloneSubFunctions.ts)`,
          "Bash(ts-node cloneSubFunctions.ts --latest)",
          "Bash(npx ts-node cloneSubFunctions.ts --latest)",
          `Bash(cd ${slashCwd} && ts-node cloneSubFunctions.ts --latest)`,
          `Bash(cd ${backslashCwd} && ts-node cloneSubFunctions.ts --latest)`,
          `Bash(cd ${slashCwd} && npx ts-node cloneSubFunctions.ts --latest)`,
          `Bash(cd ${backslashCwd} && npx ts-node cloneSubFunctions.ts --latest)`,
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

    // Check that all open issues have been marked as resolved
    const resolvedIssuesPath = "resolvedIssues.json";
    let resolvedIssues: number[] = [];
    if (existsSync(resolvedIssuesPath)) {
      resolvedIssues = JSON.parse(readFileSync(resolvedIssuesPath, "utf-8"));
    }
    const openIssues = fetchOpenIssues();
    const unresolvedIssues = openIssues.filter(
      (issue) => !resolvedIssues.includes(issue.number),
    );
    if (unresolvedIssues.length > 0) {
      const nums = unresolvedIssues.map((i) => `#${i.number}`).join(", ");
      lastFailureReasons.push(
        `There are unresolved GitHub issues: ${nums}. Mark them resolved with ts-node closeIssue.ts <number>.`,
      );
      log(`Failed: Unresolved issues: ${nums}`);
    }

    const hasChanges = hasUncommittedChanges() || hasUntrackedFiles();
    if (hasChanges) {
      lastFailureReasons.push(
        "There are uncommitted changes or untracked files. Commit them with ts-node commitAndPush.ts <message>.",
      );
      log("Failed: There are uncommitted changes or untracked files.");
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
    throw new Error("Issue loop failed after maximum attempts.");
  }

  // Push the commits
  log("Pushing commits...");
  pushOrCreateUpstream();

  // Close the issues that were marked as resolved
  const resolvedIssuesPath = "resolvedIssues.json";
  if (existsSync(resolvedIssuesPath)) {
    const resolvedIssues: number[] = JSON.parse(
      readFileSync(resolvedIssuesPath, "utf-8"),
    );
    for (const issueNumber of resolvedIssues) {
      log(`Closing issue #${issueNumber}...`);
      try {
        closeIssue(issueNumber);
      } catch (err) {
        log(`Failed to close issue #${issueNumber}: ${err}`);
      }
    }
  }

  return sessionId;
}

// Main entry point for handling issues on an existing function
export async function handleIssues(options: AgentOptions = {}): Promise<void> {
  const log = options.log ?? createFileLogger().log;

  // Run preparation (init + steps 1-8) - but spec already exists
  const sessionId = await prepare({ ...options, log });

  // Run issue loop
  log("=== Issue Loop: Handling issues on existing function ===");
  await handleIssuesLoop(log, sessionId);

  log("=== ObjectiveAI Function issue handling complete ===");
}
