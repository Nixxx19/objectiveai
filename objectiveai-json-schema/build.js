const { readFile, readdir, rm, mkdir, writeFile } = require("node:fs/promises");
const { join } = require("node:path");
const { pathToFileURL } = require("node:url");

const ROOT = __dirname;
const SRC = join(ROOT, "..", "objectiveai-js", "src");

async function clean() {
  const entries = await readdir(ROOT, { withFileTypes: true });
  for (const entry of entries) {
    const full = join(ROOT, entry.name);
    if (entry.isFile() && (entry.name.endsWith(".json") || entry.name === "lengths.csv")) {
      await rm(full);
    } else if (entry.isDirectory()) {
      await rm(full, { recursive: true });
    }
  }
}

async function getNamespaceMap(indexTsPath) {
  let content;
  try {
    content = await readFile(indexTsPath, "utf-8");
  } catch {
    return {};
  }
  const map = {};
  const re = /export\s+\*\s+as\s+(\w+)\s+from\s+["']\.\/([^"']+)["']/g;
  let m;
  while ((m = re.exec(content)) !== null) {
    map[m[1]] = m[2].replace(/\/index(\.js)?$/, "").replace(/\.js$/, "");
  }
  return map;
}

async function walk(obj, subDir, results, visited) {
  if (visited.has(obj)) return;
  visited.add(obj);

  const indexTs = join(SRC, subDir, "index.ts");
  const nsMap = await getNamespaceMap(indexTs);

  for (const key of Object.keys(obj)) {
    if (key.startsWith("__") || key === "default") continue;
    const val = obj[key];
    if (
      key.endsWith("JsonSchema") &&
      typeof val === "object" &&
      val !== null &&
      !Array.isArray(val)
    ) {
      const name = key.slice(0, -"JsonSchema".length);
      results.push({ dir: subDir, name, value: val });
    } else if (
      key in nsMap &&
      typeof val === "object" &&
      val !== null
    ) {
      const next = subDir ? `${subDir}/${nsMap[key]}` : nsMap[key];
      await walk(val, next, results, visited);
    }
  }
}

async function build() {
  await clean();

  const distPath = join(ROOT, "..", "objectiveai-js", "dist", "index.js");
  const mod = await import(pathToFileURL(distPath).href);

  const results = [];
  await walk(mod, "", results, new Set());

  const lengths = [];
  for (const { dir, name, value } of results) {
    const targetDir = dir ? join(ROOT, dir) : ROOT;
    await mkdir(targetDir, { recursive: true });
    const json = JSON.stringify(value, null, 2) + "\n";
    await writeFile(join(targetDir, `${name}.json`), json);

    const relPath = dir ? `${dir}/${name}.json` : `${name}.json`;
    const lineCount = json.split("\n").length - 1;
    lengths.push({ path: relPath, lines: lineCount });
  }

  lengths.sort((a, b) => b.lines - a.lines);
  const csv =
    "path,line_length\n" +
    lengths.map(({ path, lines }) => `${path},${lines}`).join("\n") +
    "\n";
  await writeFile(join(ROOT, "lengths.csv"), csv);

  console.log(`Written ${results.length} JSON schema files`);
}

build().catch(console.error);
