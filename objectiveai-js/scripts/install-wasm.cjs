const { spawnSync } = require("child_process");
const { mkdirSync, copyFileSync } = require("fs");
const path = require("path");

// Paths
const jsRoot = process.cwd(); // objectiveai-js
const repoRoot = path.resolve(jsRoot, ".."); // objectiveai
const wasmDir = path.join(repoRoot, "objectiveai", "objectiveai-wasm-js");
const pkgDir = path.join(wasmDir, "pkg");
const outDir = path.join(jsRoot, "src", "wasm");

// 1. Run wasm-pack build
console.log("⚙ Running wasm-pack build");

const result = spawnSync(
  "wasm-pack",
  ["build", "--target", "bundler", "--release", "--out-dir", "pkg"],
  {
    cwd: wasmDir,
    stdio: "inherit",
    shell: process.platform === "win32",
  }
);

if (result.status !== 0) {
  process.exit(result.status ?? 1);
}

// 2. Ensure destination directory exists
mkdirSync(outDir, { recursive: true });

// 3. Copy exact files
const files = [
  "objectiveai_wasm_js_bg.js",
  "objectiveai_wasm_js_bg.wasm",
  "objectiveai_wasm_js_bg.wasm.d.ts",
  "objectiveai_wasm_js.d.ts",
  "objectiveai_wasm_js.js",
];

for (const file of files) {
  const from = path.join(pkgDir, file);
  const to = path.join(outDir, file);
  copyFileSync(from, to);
  console.log(`✓ Copied ${file}`);
}
