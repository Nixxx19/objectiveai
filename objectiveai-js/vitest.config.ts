import { defineConfig } from "vitest/config";
import path from "path";

export default defineConfig({
  resolve: {
    alias: {
      src: path.resolve(__dirname, "src"),
    },
  },
  test: {
    include: ["src/**/*.test.ts"],
    testTimeout: 0,
    globalSetup: ["./scripts/vitest-global-setup.cjs"],
  },
});
