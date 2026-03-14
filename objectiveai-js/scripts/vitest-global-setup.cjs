// Vitest globalSetup: spawns the objectiveai-api server before all tests
// and tears it down after. Sets OBJECTIVEAI_TEST_PORT for http.test.ts files.

const { spawnServer } = require("./spawn-api-server.cjs");

let server;

module.exports = async function setup() {
  server = await spawnServer();
  process.env.OBJECTIVEAI_TEST_PORT = String(server.port);

  return async function teardown() {
    if (server) server.kill();
  };
};
