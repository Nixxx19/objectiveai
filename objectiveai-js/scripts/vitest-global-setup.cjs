// Vitest globalSetup: spawns the objectiveai-api server before all tests
// and tears it down after. Sets OBJECTIVEAI_TEST_PORT for http.test.ts files.
//
// If OBJECTIVEAI_TEST_PORT is already set, skips spawning (assumes server is running).

const { spawn } = require("child_process");
const path = require("path");

let child;

module.exports = async function setup() {
  if (process.env.OBJECTIVEAI_TEST_PORT) {
    return async function teardown() {};
  }

  const script = path.resolve(__dirname, "../../test-spawn-api-server");

  child = spawn("bash", [script], {
    stdio: ["ignore", "pipe", "pipe"],
  });

  const port = await new Promise((resolve, reject) => {
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk) => {
      stdout += chunk.toString();
      const line = stdout.split("\n")[0].trim();
      if (line && /^\d+$/.test(line)) resolve(line);
    });
    child.stderr.on("data", (chunk) => { stderr += chunk.toString(); });
    child.on("close", (code) => {
      if (code !== 0) reject(new Error(`test-spawn-api-server failed:\n${stderr}`));
    });
  });

  process.env.OBJECTIVEAI_TEST_PORT = port;

  return async function teardown() {
    if (child) {
      child.kill("SIGTERM");
      child.stdout.destroy();
      child.stderr.destroy();
    }
  };
};
