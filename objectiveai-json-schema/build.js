const { readdir, rm, writeFile } = require("node:fs/promises");
const { join } = require("node:path");
const { pathToFileURL } = require("node:url");

const ROOT = __dirname;

async function clean() {
  const entries = await readdir(ROOT, { withFileTypes: true });
  for (const entry of entries) {
    if (entry.isFile() && (entry.name.endsWith(".json") || entry.name === "lengths.csv")) {
      await rm(join(ROOT, entry.name));
    }
  }
}

function isZodSchema(val) {
  return (
    typeof val === "object" &&
    val !== null &&
    typeof val.meta === "function" &&
    typeof val.parse === "function"
  );
}

function walk(obj, results, visited) {
  if (visited.has(obj)) return;
  visited.add(obj);

  for (const key of Object.keys(obj)) {
    if (key.startsWith("__") || key === "default") continue;
    const val = obj[key];
    if (typeof val !== "object" || val === null) continue;

    if (
      key.endsWith("Schema") &&
      !key.endsWith("JsonSchema") &&
      isZodSchema(val)
    ) {
      let meta;
      try {
        meta = val.meta();
      } catch {
        continue;
      }
      if (!meta || !meta.title || meta.wrapper) continue;
      results.set(meta.title, val);
    } else if (
      /^[A-Z]/.test(key) &&
      !key.endsWith("Schema") &&
      !key.endsWith("JsonSchema") &&
      !Array.isArray(val) &&
      typeof val.parse !== "function"
    ) {
      walk(val, results, visited);
    }
  }
}

async function build() {
  await clean();

  const distPath = join(ROOT, "..", "objectiveai-js", "dist", "index.js");
  const mod = await import(pathToFileURL(distPath).href);
  const { convert } = mod;

  const results = new Map();
  walk(mod, results, new Set());

  const lengths = [];
  for (const [title, schema] of results) {
    const value = convert(schema);
    const json = JSON.stringify(value, null, 2) + "\n";
    const fileName = `${title}.json`;
    await writeFile(join(ROOT, fileName), json);

    const lineCount = json.split("\n").length - 1;
    lengths.push({ path: fileName, lines: lineCount });
  }

  lengths.sort((a, b) => b.lines - a.lines);
  const csv =
    "path,line_length\n" +
    lengths.map(({ path, lines }) => `${path},${lines}`).join("\n") +
    "\n";
  await writeFile(join(ROOT, "lengths.csv"), csv);

  console.log(`Written ${results.size} JSON schema files`);
}

build().catch(console.error);
