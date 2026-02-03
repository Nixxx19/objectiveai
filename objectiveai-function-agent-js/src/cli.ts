import { Claude } from "./index";

async function main(): Promise<void> {
  const command = process.argv[2];
  const spec = process.argv[3];

  switch (command) {
    case "invent":
      await Claude.invent({ spec });
      break;
    case "handle-issues":
      await Claude.handleIssues({ spec });
      break;
    default:
      console.log("Usage: objectiveai-function-agent <command> [spec]");
      console.log("");
      console.log("Commands:");
      console.log("  invent         Create a new ObjectiveAI Function");
      console.log("  handle-issues  Handle GitHub issues on an existing function");
      console.log("");
      console.log("Options:");
      console.log("  [spec]         Optional spec string for SPEC.md");
      process.exit(1);
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
