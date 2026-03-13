/**
 * Strict roundtrip test: Zod schema → JSON Schema must exactly match the
 * originals in objectiveai-json-schema/.
 *
 * RULES FOR MODIFYING THIS FILE
 * =============================
 *
 * 1. This test MUST NEVER read, load, or deserialize the original JSON schema
 *    files. The harness handles that. Any attempt to read the originals from
 *    this file would defeat the purpose of the test — it would be cheating.
 *    The JSON schemas must be fully reconstructible from the Zod schemas alone.
 *
 * 2. This test MUST be entirely generic. It must work unchanged even if every
 *    existing JSON schema is deleted and replaced with completely different
 *    schemas. No schema-specific special cases, no hardcoded titles, no
 *    xfail lists. If a schema fails, the fix belongs in install-zod.cjs or
 *    in the normalization logic below — never in a per-schema workaround.
 *
 * 3. To make tests pass, you may modify:
 *    - This file (test-zod-roundtrip.cjs) — conversion logic
 *    - scripts/install-zod.cjs — the Zod code generator
 *
 * 4. You MUST NEVER modify:
 *    - test-zod-roundtrip-harness.cjs — the comparison harness
 *    - Any file in objectiveai-json-schema/ — the canonical source of truth
 *
 * DESIGN CHOICE
 * =============
 *
 * This test uses a custom Zod → JSON Schema converter that walks the Zod
 * internal schema tree (_zod.def) directly, rather than using Zod's built-in
 * toJSONSchema(). This produces the correct output format from the start with
 * no cleanup passes needed. It is simpler, smaller, and does not depend on
 * Zod's toJSONSchema output format, which would require extensive
 * post-processing to strip $schema, $defs, additionalProperties:false,
 * propertyNames, safe integer bounds, and other artifacts.
 *
 * This is an information-loss and reconstructibility test. It proves that the
 * Zod schemas contain enough information to perfectly reconstruct the original
 * JSON schemas, with no data lost during the JSON Schema → Zod → JSON Schema
 * round trip.
 */

const SDK = require("../dist/index.cjs");
const harness = require("./test-zod-roundtrip-harness.cjs");

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/**
 * Detect if a Zod schema is JsonValueSchema (the "any JSON value" type).
 * It's a union of [string, number, boolean, null, array(lazy→self), record(string, lazy→self)]
 * where the lazy getters' results share the same options array identity.
 */
function isJsonValueSchema(schema) {
  if (schema._zod.def.type !== "union") return false;
  const opts = schema._zod.def.options;
  if (!opts || opts.length !== 6) return false;
  const types = opts.map((o) => o._zod.def.type);
  if (!["string", "number", "boolean", "null", "array", "record"].every((t) => types.includes(t))) {
    return false;
  }
  // Verify the array element and record value are lazy self-references
  const arr = opts.find((o) => o._zod.def.type === "array");
  const rec = opts.find((o) => o._zod.def.type === "record");
  const arrEl = arr?._zod.def.element;
  const recVal = rec?._zod.def.valueType;
  if (arrEl?._zod.def.type !== "lazy" || recVal?._zod.def.type !== "lazy") return false;
  // The lazy getter's result should share the same options (identity check)
  const lazyResult = arrEl._zod.def.getter();
  return lazyResult._zod.def.options === opts;
}

/** "agent.Agent" → "AgentAgentSchema" */
function titleToSchemaName(title) {
  return title
    .split(/[._]/)
    .filter(Boolean)
    .map((s) => s[0].toUpperCase() + s.slice(1))
    .join("") + "Schema";
}

// ---------------------------------------------------------------------------
// Custom Zod → JSON Schema converter
//
// Walks the Zod v4 internal tree (_zod.def) and emits JSON Schema objects
// matching the conventions of the objectiveai-json-schema builder.
// ---------------------------------------------------------------------------

/**
 * Convert a Zod schema to a JSON Schema object.
 *
 * @param {object} schema - A Zod schema instance
 * @param {Set<string>} allTitles - All known schema titles (for $ref detection)
 * @param {string} rootTitle - Title of the root schema being converted
 * @param {Set} [seen] - Cycle detection set
 * @returns {object} JSON Schema object
 */
function convert(schema, allTitles, rootTitle, seen) {
  if (!seen) seen = new Set();
  const def = schema._zod.def;

  // Detect "any JSON value" schemas (JsonValueSchema and its lazy wrappers).
  // JsonValueSchema is a union of [string, number, boolean, null, array, record]
  // where the array element and record value are lazy self-references.
  // These collapse to {} (any JSON value), preserving description if present.
  if (isJsonValueSchema(schema)) {
    const desc = schema.description;
    return desc ? { description: desc } : {};
  }

  // Check for title via .meta() — if it matches a known title, emit $ref
  const meta = typeof schema.meta === "function" ? schema.meta() : undefined;
  const title = meta?.title;
  if (title && allTitles.has(title)) {
    // Self-reference: if we've already seen this schema (entered it), emit $ref
    if (seen.has(schema)) {
      const result = { $ref: title };
      const desc = schema.description;
      const targetSchema = SDK[titleToSchemaName(title)];
      const targetDesc = targetSchema?.description;
      if (desc && desc !== targetDesc) {
        result.description = desc;
      }
      return result;
    }
    // Non-root ref: emit $ref immediately
    if (title !== rootTitle) {
      const result = { $ref: title };
      const desc = schema.description;
      const targetSchema = SDK[titleToSchemaName(title)];
      const targetDesc = targetSchema?.description;
      if (desc && desc !== targetDesc) {
        result.description = desc;
      }
      return result;
    }
    // Root schema first encounter: mark as seen and convert inline
    seen = new Set(seen).add(schema);
  }

  return convertInner(schema, allTitles, rootTitle, seen);
}

/** Convert the core of a Zod schema (after $ref check). */
function convertInner(schema, allTitles, rootTitle, seen) {
  const def = schema._zod.def;
  const type = def.type;
  let result;

  switch (type) {
    case "string":
      result = convertString(schema);
      break;
    case "number":
      result = convertNumber(schema);
      break;
    case "boolean":
      result = { type: "boolean" };
      break;
    case "null":
      result = { type: "null" };
      break;
    case "literal":
      result = convertLiteral(schema);
      break;
    case "enum":
      result = { type: "string", enum: Object.values(def.entries) };
      break;
    case "array":
      result = convertArray(schema, allTitles, rootTitle, seen);
      break;
    case "object":
      result = convertObject(schema, allTitles, rootTitle, seen);
      break;
    case "record":
      result = convertRecord(schema, allTitles, rootTitle, seen);
      break;
    case "union":
      result = convertUnion(schema, allTitles, rootTitle, seen);
      break;
    case "intersection":
      result = convertIntersection(schema, allTitles, rootTitle, seen);
      break;
    case "nullable":
      result = convertNullable(schema, allTitles, rootTitle, seen);
      break;
    case "optional":
      // optional is handled by the parent object; convert the inner type
      result = convert(def.innerType, allTitles, rootTitle, seen);
      break;
    case "default":
      result = convert(def.innerType, allTitles, rootTitle, seen);
      result = { ...result, default: def.defaultValue };
      break;
    case "lazy":
      result = convert(def.getter(), allTitles, rootTitle, seen);
      break;
    case "never":
      // Used for .strict() catchall — not directly emitted
      result = {};
      break;
    default:
      // Unknown type → any JSON value
      result = {};
      break;
  }

  // Apply description from this level (if not already set by a deeper level)
  const desc = schema.description;
  if (desc && !result.description) {
    result = { ...result, description: desc };
  }

  return result;
}

function convertLiteral(schema) {
  const values = [...schema._zod.def.values];
  const result = { enum: values };
  // Add type annotation matching the value's JS type
  const litValue = values[0];
  if (typeof litValue === "string") return { type: "string", ...result };
  if (typeof litValue === "number") return { type: "number", ...result };
  if (typeof litValue === "boolean") return { type: "boolean", ...result };
  return result;
}

function convertString(schema) {
  const result = { type: "string" };
  // Check for regex pattern
  if (schema._zod.def.checks) {
    for (const check of schema._zod.def.checks) {
      if (check._zod?.def?.check === "string_format" && check._zod.def.pattern) {
        result.pattern = check._zod.def.pattern.source;
      }
    }
  }
  // Check for format in meta
  const meta = typeof schema.meta === "function" ? schema.meta() : undefined;
  if (meta?.format) {
    result.format = meta.format;
  }
  return result;
}

function convertNumber(schema) {
  const isInt = schema.isInt === true;
  const result = { type: isInt ? "integer" : "number" };

  // Read min/max directly from checks (accessors clamp to safe integer range)
  if (schema._zod.def.checks) {
    for (const check of schema._zod.def.checks) {
      const cdef = check._zod?.def;
      if (!cdef) continue;
      if (cdef.check === "greater_than" && cdef.inclusive) result.minimum = cdef.value;
      if (cdef.check === "less_than" && cdef.inclusive) result.maximum = cdef.value;
    }
  }

  return result;
}

function convertArray(schema, allTitles, rootTitle, seen) {
  const result = { type: "array" };
  if (schema._zod.def.element) {
    result.items = convert(schema._zod.def.element, allTitles, rootTitle, seen);
  }
  // Check for minItems/maxItems in checks
  if (schema._zod.def.checks) {
    for (const check of schema._zod.def.checks) {
      const cdef = check._zod?.def;
      if (cdef?.check === "min_size") result.minItems = cdef.value;
      if (cdef?.check === "max_size") result.maxItems = cdef.value;
    }
  }
  return result;
}

function convertObject(schema, allTitles, rootTitle, seen) {
  const result = { type: "object" };
  const shape = schema._zod.def.shape;

  if (shape && Object.keys(shape).length > 0) {
    const properties = {};
    for (const [key, propSchema] of Object.entries(shape)) {
      // Unwrap optional — the optional wrapper itself just signals optionality
      let inner = propSchema;
      if (inner._zod.def.type === "optional") {
        inner = inner._zod.def.innerType;
      }
      properties[key] = convert(inner, allTitles, rootTitle, seen);
    }
    result.properties = properties;
  }

  // Handle additionalProperties
  const catchall = schema._zod.def.catchall;
  if (catchall) {
    if (catchall._zod.def.type === "never") {
      result.additionalProperties = false;
    } else {
      result.additionalProperties = convert(catchall, allTitles, rootTitle, seen);
    }
  }

  return result;
}

function convertRecord(schema, allTitles, rootTitle, seen) {
  const result = { type: "object" };
  if (schema._zod.def.valueType) {
    const valSchema = convert(schema._zod.def.valueType, allTitles, rootTitle, seen);
    // {} (any JSON value) → additionalProperties: true
    result.additionalProperties = Object.keys(valSchema).length === 0 ? true : valSchema;
  }
  return result;
}

function convertUnion(schema, allTitles, rootTitle, seen) {
  const options = schema._zod.def.options;

  // Check if all options are literals with no individual descriptions → emit as flat enum
  const allLiterals = options.every((o) => o._zod.def.type === "literal");
  if (allLiterals) {
    const anyHasDesc = options.some((o) => o.description);
    if (!anyHasDesc) {
      const values = options.flatMap((o) => [...o._zod.def.values]);
      return { enum: values };
    }
    // Literals with descriptions → anyOf of typed enums
    // (each variant carries its own description)
  }

  // Check for nullable pattern: [...variants, null-typed option]
  const nullIdx = options.findIndex((o) => o._zod.def.type === "null");
  if (nullIdx !== -1) {
    const nonNull = options.filter((_, i) => i !== nullIdx);
    const inner = nonNull.length === 1
      ? convert(nonNull[0], allTitles, rootTitle, seen)
      : { anyOf: nonNull.map((o) => convert(o, allTitles, rootTitle, seen)) };
    return { anyOf: [inner, { type: "null" }] };
  }

  return { anyOf: options.map((o) => convert(o, allTitles, rootTitle, seen)) };
}

function convertNullable(schema, allTitles, rootTitle, seen) {
  const inner = convert(schema._zod.def.innerType, allTitles, rootTitle, seen);
  return { anyOf: [inner, { type: "null" }] };
}

function convertIntersection(schema, allTitles, rootTitle, seen) {
  // Intersection (.and()) is used for $ref + properties (adjacently-tagged enums)
  // and for anyOf + properties (serde flatten).
  // Left side is typically the union/ref, right side is the object with extra properties.
  const left = convert(schema._zod.def.left, allTitles, rootTitle, seen);
  const right = convert(schema._zod.def.right, allTitles, rootTitle, seen);

  // If left is a $ref node, merge right's properties as siblings
  if (left.$ref) {
    const result = {};
    if (left.description) result.description = left.description;
    if (right.type) result.type = right.type;
    result.$ref = left.$ref;
    if (right.properties) result.properties = right.properties;
    if (right.additionalProperties !== undefined) {
      result.additionalProperties = right.additionalProperties;
    }
    return result;
  }

  // If left has anyOf (serde flatten), combine with right's object part
  if (left.anyOf) {
    const result = {};
    if (left.description) result.description = left.description;
    result.anyOf = left.anyOf;
    if (right.properties) result.properties = right.properties;
    if (right.type) result.type = right.type;
    return result;
  }

  // Fallback: merge both sides
  return { ...left, ...right };
}

// ---------------------------------------------------------------------------
// Top-level schema conversion
// ---------------------------------------------------------------------------

/**
 * Convert a top-level Zod schema to JSON Schema, including the title and
 * root-level description.
 */
function convertTopLevel(schema, allTitles) {
  const meta = typeof schema.meta === "function" ? schema.meta() : {};
  const title = meta?.title;
  const desc = schema.description;

  // Convert without $ref-ifying the root itself (but add it to seen for self-ref detection)
  const seen = new Set();
  seen.add(schema);
  const result = convertInner(schema, allTitles, title, seen);

  // Add title and description at the top
  const output = {};
  if (title) output.title = title;
  if (desc) output.description = desc;
  Object.assign(output, result);
  // If description was already in result, the top-level one wins
  if (desc) output.description = desc;

  return output;
}

// ---------------------------------------------------------------------------
// Test runner
// ---------------------------------------------------------------------------

let passed = 0;
let failed = 0;
const failures = [];

for (const title of harness.ALL_TITLES) {
  const schemaName = titleToSchemaName(title);
  const zodSchema = SDK[schemaName];

  if (!zodSchema) {
    console.log(`  \u2717 ${title} — missing export: ${schemaName}`);
    failed++;
    failures.push({ title, error: `Missing Zod schema export: ${schemaName}` });
    continue;
  }

  try {
    const converted = convertTopLevel(zodSchema, harness.ALL_TITLES);
    harness.assertSchemaMatches(title, converted);
    passed++;
  } catch (err) {
    console.log(`  \u2717 ${title}`);
    failed++;
    failures.push({ title, error: err.message });
  }
}

console.log(`\n${passed} passed, ${failed} failed out of ${passed + failed}`);

if (failures.length > 0) {
  console.log("\nFailures:\n");
  for (const { title, error } of failures.slice(0, 10)) {
    console.log(`--- ${title} ---`);
    console.log(error.slice(0, 2000));
    console.log();
  }
  if (failures.length > 10) {
    console.log(`... and ${failures.length - 10} more`);
  }
  process.exit(1);
}

console.log("\nAll schemas match!");
