// Spawns an objectiveai-api server on a system-assigned port.
//
// Usage:
//   const { spawn: spawnServer } = require("./spawn-api-server.cjs");
//   const { process, port, baseUrl, kill } = await spawnServer();
//
// The server is started with MOCK_DELAY_MS=0 so tests run fast.
// Call kill() to shut it down.

const { spawn } = require("child_process");
const net = require("net");
const path = require("path");

const API_CRATE_DIR = path.resolve(__dirname, "../../objectiveai-api");

/** Find a free port by binding to port 0 and reading the assigned port. */
function getFreePort() {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.listen(0, "127.0.0.1", () => {
      const port = server.address().port;
      server.close(() => resolve(port));
    });
    server.on("error", reject);
  });
}

/** Wait until the server is accepting connections on the given port. */
function waitForReady(port, timeoutMs = 120000) {
  const start = Date.now();
  return new Promise((resolve, reject) => {
    function tryConnect() {
      if (Date.now() - start > timeoutMs) {
        return reject(new Error(`Server did not become ready within ${timeoutMs}ms`));
      }
      const socket = net.createConnection({ host: "127.0.0.1", port }, () => {
        socket.destroy();
        resolve();
      });
      socket.on("error", () => {
        setTimeout(tryConnect, 100);
      });
    }
    tryConnect();
  });
}

/**
 * Spawn an objectiveai-api server on a free port.
 * Returns { process, port, baseUrl, kill }.
 */
async function spawnServer(opts = {}) {
  const port = await getFreePort();
  const env = {
    ...process.env,
    PORT: String(port),
    ADDRESS: "127.0.0.1",
    MOCK_DELAY_MS: "0",
    ...opts.env,
  };

  const child = spawn("cargo", ["run", "--package", "objectiveai-api"], {
    cwd: API_CRATE_DIR,
    env,
    stdio: ["ignore", "pipe", "pipe"],
  });

  // Collect stderr for diagnostics on failure
  let stderr = "";
  child.stderr.on("data", (chunk) => {
    stderr += chunk.toString();
  });

  const kill = () => {
    child.kill("SIGTERM");
  };

  try {
    await waitForReady(port);
  } catch (err) {
    kill();
    throw new Error(`Failed to start API server:\n${stderr}\n${err.message}`);
  }

  return {
    process: child,
    port,
    baseUrl: `http://127.0.0.1:${port}`,
    kill,
  };
}

module.exports = { spawnServer };
