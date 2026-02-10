import { Command } from "commander";
import { Claude } from "./index";

// If spawned by a parent agent, exit when the parent dies.
const parentPid = process.env.OBJECTIVEAI_PARENT_PID
  ? parseInt(process.env.OBJECTIVEAI_PARENT_PID, 10)
  : undefined;
if (parentPid) {
  const watchdog = setInterval(() => {
    try {
      process.kill(parentPid, 0);
    } catch {
      clearInterval(watchdog);
      process.exit(1);
    }
  }, 3000);
  watchdog.unref();
}

const program = new Command();

program
  .name("objectiveai-function-agent")
  .description("Autonomous agent for creating ObjectiveAI Functions");

program
  .command("invent")
  .description("Invent a new ObjectiveAI Function")
  .argument("[spec]", "Optional spec string for SPEC.md")
  .option("--name <name>", "Function name for name.txt")
  .option("--depth <n>", "Depth level (0=vector, >0=function tasks)", parseInt)
  .option("--api-base <url>", "API base URL")
  .option("--api-key <key>", "ObjectiveAI API key")
  .option("--instructions <text>", "Extra instructions for the invent agent")
  .option("--git-user-name <name>", "Git author/committer name")
  .option("--git-user-email <email>", "Git author/committer email")
  .action(async (spec, opts) => {
    await Claude.invent({
      spec,
      name: opts.name,
      depth: opts.depth,
      apiBase: opts.apiBase,
      apiKey: opts.apiKey,
      instructions: opts.instructions,
      gitUserName: opts.gitUserName,
      gitUserEmail: opts.gitUserEmail,
    });
  });

program.parse();
