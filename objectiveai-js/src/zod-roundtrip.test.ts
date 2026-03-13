/**
 * Roundtrip test: Zod schema → JSON Schema must match the original
 * objectiveai-json-schema/ files, ensuring no information is lost
 * during the Zod conversion.
 */
import { describe, it, expect } from "vitest";
import { z, toJSONSchema } from "zod";
import { JsonValueSchema } from "./json";
import * as fs from "fs";
import * as path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const SCHEMA_DIR = path.resolve(__dirname, "../../objectiveai-json-schema");

const SAFE_INT_MIN = -9007199254740991;
const SAFE_INT_MAX = 9007199254740991;

// ---------------------------------------------------------------------------
// Helpers from install-zod.cjs (must match exactly)
// ---------------------------------------------------------------------------

function titleToPascal(title: string): string {
  return title
    .split(/[._]/)
    .filter(Boolean)
    .map((s) => s[0].toUpperCase() + s.slice(1))
    .join("");
}

// ---------------------------------------------------------------------------
// Load JSON schemas from objectiveai-json-schema/
// ---------------------------------------------------------------------------

function loadJsonSchemas(): Map<string, any> {
  const files = fs.readdirSync(SCHEMA_DIR).filter((f) => f.endsWith(".json"));
  const schemas = new Map<string, any>();
  for (const file of files) {
    const content = JSON.parse(
      fs.readFileSync(path.join(SCHEMA_DIR, file), "utf-8"),
    );
    if (content.title) {
      schemas.set(content.title, content);
    }
  }
  return schemas;
}

// ---------------------------------------------------------------------------
// Convert Zod → JSON Schema with our conventions
// ---------------------------------------------------------------------------

function zodToJsonSchema(
  schema: z.ZodType,
  allTitles: Set<string>,
  rootTitle: string,
  titleDescriptions: Map<string, string>,
): any {
  const result = toJSONSchema(schema, {
    reused: "inline",
    override(ctx) {
      const js = ctx.jsonSchema;
      if (!js || typeof js !== "object") return;

      // JsonValueSchema (and its inner z.lazy refs, and .describe() wrappers) → {} (any JSON value)
      if (
        ctx.zodSchema === JsonValueSchema ||
        (ctx.zodSchema as any)?._zod?.def?.options === (JsonValueSchema as any)._zod.def.options ||
        ((ctx.zodSchema as any)?._zod?.def?.type === "lazy" &&
         (ctx.zodSchema as any)._zod.def.getter() === JsonValueSchema)
      ) {
        const desc = js.description;
        for (const key of Object.keys(js)) delete js[key];
        if (desc) js.description = desc;
        return;
      }

      // Strip additionalProperties: false from non-strict objects
      // (Zod adds it to all objects, but only .strict() should produce it)
      if (
        js.additionalProperties === false &&
        js.type === "object" &&
        (ctx.zodSchema?._zod?.def as any)?.catchall?.type !== "never"
      ) {
        delete js.additionalProperties;
      }

      // Emit $ref for schemas whose title matches a known schema title,
      // but not for the root schema itself (mutate in place)
      if (
        "title" in js &&
        typeof js.title === "string" &&
        allTitles.has(js.title) &&
        js.title !== rootTitle
      ) {
        const title = js.title;
        const desc = js.description;
        const ownDesc = titleDescriptions.get(title);
        for (const key of Object.keys(js)) delete js[key];
        js.$ref = title;
        // Preserve description only if it differs from the schema's own definition
        if (desc && desc !== ownDesc) {
          js.description = desc;
        }
      }
    },
  });

  // Resolve internal $defs references before cleaning
  const resolved = resolveDefs(result, result.$defs || {});
  return cleanConverted(resolved);
}

/** Replace internal $ref: "#/$defs/..." with the actual $defs content. */
function resolveDefs(obj: any, defs: Record<string, any>, resolving = new Set<string>()): any {
  if (obj === null || typeof obj !== "object") return obj;
  if (Array.isArray(obj)) return obj.map((v) => resolveDefs(v, defs, resolving));

  // If this is an internal $defs $ref, resolve it
  if (obj.$ref && typeof obj.$ref === "string" && obj.$ref.startsWith("#/$defs/")) {
    const defName = obj.$ref.slice("#/$defs/".length);
    // Break cycles (e.g., JsonValueSchema's recursive self-refs) → {}
    if (resolving.has(defName)) return {};
    const def = defs[defName];
    if (def) {
      const inner = new Set(resolving);
      inner.add(defName);
      const resolved = resolveDefs(def, defs, inner);
      // Preserve sibling properties (e.g., description) from the $ref node
      const siblings = Object.keys(obj).filter((k) => k !== "$ref");
      if (siblings.length === 0) return resolved;
      return { ...resolved, ...Object.fromEntries(siblings.map((k) => [k, obj[k]])) };
    }
  }

  const result: any = {};
  for (const [key, value] of Object.entries(obj)) {
    result[key] = resolveDefs(value, defs, resolving);
  }
  return result;
}

// ---------------------------------------------------------------------------
// Normalization: bring both representations to a common form
// ---------------------------------------------------------------------------

/** Clean the Zod→JSON Schema output: strip $schema, $defs, additionalProperties: false,
 *  Zod's auto-generated safe integer bounds, and flatten nested anyOf. */
function cleanConverted(obj: any): any {
  if (obj === null || typeof obj !== "object") return obj;
  if (Array.isArray(obj)) return obj.map(cleanConverted);

  const result: any = {};
  for (const [key, value] of Object.entries(obj)) {
    if (key === "$schema") continue;
    if (key === "$defs") continue;
    // Strip additionalProperties: {} (semantically default — any additional properties allowed)
    if (key === "additionalProperties" && typeof value === "object" && value !== null && !Array.isArray(value) && Object.keys(value).length === 0) continue;

    // Strip Zod's auto-generated safe integer bounds on integer types
    if (key === "minimum" && value === SAFE_INT_MIN && obj.type === "integer") continue;
    if (key === "maximum" && value === SAFE_INT_MAX && obj.type === "integer") continue;

    // Strip propertyNames (Zod adds { type: "string" } for z.record())
    if (key === "propertyNames") continue;

    // Flatten nested anyOf: anyOf: [anyOf: [A, B], C] → anyOf: [A, B, C]
    if (key === "anyOf" && Array.isArray(value)) {
      const flat: any[] = [];
      for (const item of value) {
        const cleaned = cleanConverted(item);
        // If item is JUST an anyOf wrapper (no other keys), flatten it
        if (cleaned && typeof cleaned === "object" && !Array.isArray(cleaned) &&
            cleaned.anyOf && Object.keys(cleaned).length === 1) {
          flat.push(...cleaned.anyOf);
        } else {
          flat.push(cleaned);
        }
      }
      result[key] = flat;
      continue;
    }

    result[key] = cleanConverted(value);
  }
  return result;
}

/** Normalize the original JSON Schema: convert oneOf→anyOf, type-array
 *  nullables to anyOf form, and single-element enum to const
 *  (matching what Zod's toJSONSchema produces). */
function normalizeOriginal(obj: any, titleDescs?: Map<string, string>, allSchemas?: Map<string, any>): any {
  if (obj === null || typeof obj !== "object") return obj;
  if (Array.isArray(obj)) return obj.map((v) => normalizeOriginal(v, titleDescs, allSchemas));

  const result: any = {};
  for (const [key, value] of Object.entries(obj)) {
    if (key === "$schema") continue;
    // Strip additionalProperties: true (semantically default — any additional properties allowed)
    if (key === "additionalProperties" && value === true) continue;

    // Convert single-element enum to const
    if (key === "enum" && Array.isArray(value) && value.length === 1) {
      result["const"] = value[0];
      continue;
    }

    // Convert oneOf → anyOf
    if (key === "oneOf") {
      result["anyOf"] = normalizeOriginal(value, titleDescs, allSchemas);
      continue;
    }

    // Convert type-array nullables: type: ["X", "null"] → anyOf form
    if (key === "type" && Array.isArray(value)) {
      const nonNull = value.filter((t: string) => t !== "null");
      const isNullable = value.includes("null");

      if (isNullable && nonNull.length === 1) {
        // Collect constraints: type-specific go into inner, description/default stay outer
        const innerSchema: any = { type: nonNull[0] };
        const outerResult: any = {};

        for (const [k2, v2] of Object.entries(obj)) {
          if (k2 === "$schema" || k2 === "type") continue;
          // Type-specific constraints go on the inner type
          if (["items", "properties", "required", "additionalProperties",
               "minimum", "maximum", "format", "pattern",
               "minItems", "maxItems"].includes(k2)) {
            innerSchema[k2] = normalizeOriginal(v2, titleDescs, allSchemas);
          } else {
            // description, default, and anything else stay on outer
            outerResult[k2] = normalizeOriginal(v2, titleDescs, allSchemas);
          }
        }

        outerResult["anyOf"] = [normalizeOriginal(innerSchema, titleDescs, allSchemas), { type: "null" }];
        return outerResult;
      }

      if (nonNull.length > 1) {
        // type: ["string", "number"] → anyOf: [{type: "string", ...}, {type: "number", ...}]
        const outerResult: any = {};
        const stringConstraints: any = {};
        const numberConstraints: any = {};

        for (const [k2, v2] of Object.entries(obj)) {
          if (k2 === "$schema" || k2 === "type") continue;
          // pattern only applies to strings
          if (k2 === "pattern") {
            stringConstraints[k2] = normalizeOriginal(v2, titleDescs, allSchemas);
          } else if (["format", "minimum", "maximum"].includes(k2)) {
            // These go on all applicable variants
            numberConstraints[k2] = normalizeOriginal(v2, titleDescs, allSchemas);
            stringConstraints[k2] = normalizeOriginal(v2, titleDescs, allSchemas);
          } else {
            outerResult[k2] = normalizeOriginal(v2, titleDescs, allSchemas);
          }
        }

        const variants = nonNull.map((t: string) => {
          const constraints = t === "string" ? stringConstraints :
                             (t === "number" || t === "integer") ? numberConstraints : {};
          const v: any = { type: t, ...constraints };
          return normalizeOriginal(v, titleDescs, allSchemas);
        });

        if (isNullable) {
          variants.push({ type: "null" });
        }

        outerResult["anyOf"] = variants;
        return outerResult;
      }

      result[key] = value;
      continue;
    }

    result[key] = normalizeOriginal(value, titleDescs, allSchemas);
  }

  // Handle $ref nodes
  if (result.$ref) {
    // $ref with sibling properties (adjacently-tagged enum): resolve and merge
    if (result.properties && allSchemas) {
      const target = allSchemas.get(result.$ref);
      if (target) {
        const normalized = normalizeOriginal(target, titleDescs, allSchemas);
        const merged: any = {};
        // Keep description from the variant (usage-site), not the referenced schema
        if (result.description) {
          merged.description = result.description;
        }
        merged.type = normalized.type || result.type || "object";
        // Merge properties: target first, sibling overrides
        merged.properties = {
          ...(normalized.properties || {}),
          ...(result.properties || {}),
        };
        // Merge required arrays
        const reqSet = new Set([
          ...(normalized.required || []),
          ...(result.required || []),
        ]);
        if (reqSet.size > 0) merged.required = [...reqSet];
        // Carry over other target fields (like additionalProperties)
        for (const [k, v] of Object.entries(normalized)) {
          if (!["$schema", "title", "description", "type", "properties", "required"].includes(k)) {
            merged[k] = v;
          }
        }
        return merged;
      }
    }
    // $ref without sibling properties: keep $ref, preserve non-matching description
    const cleaned: any = { $ref: result.$ref };
    if (result.description && titleDescs && result.description !== titleDescs.get(result.$ref)) {
      cleaned.description = result.description;
    }
    return cleaned;
  }
  if (result.anyOf && Array.isArray(result.anyOf)) {
    result.anyOf = result.anyOf.map((variant: any) => {
      if (variant && variant.$ref) {
        // $ref with sibling properties inside anyOf
        if (variant.properties && allSchemas) {
          const target = allSchemas.get(variant.$ref);
          if (target) {
            const normalized = normalizeOriginal(target, titleDescs, allSchemas);
            const merged: any = {};
            if (variant.description) merged.description = variant.description;
            merged.type = normalized.type || variant.type || "object";
            merged.properties = {
              ...(normalized.properties || {}),
              ...(variant.properties || {}),
            };
            const reqSet = new Set([
              ...(normalized.required || []),
              ...(variant.required || []),
            ]);
            if (reqSet.size > 0) merged.required = [...reqSet];
            for (const [k, v] of Object.entries(normalized)) {
              if (!["$schema", "title", "description", "type", "properties", "required"].includes(k)) {
                merged[k] = v;
              }
            }
            return merged;
          }
        }
        // $ref without sibling properties
        const cleaned: any = { $ref: variant.$ref };
        if (variant.description && titleDescs && variant.description !== titleDescs.get(variant.$ref)) {
          cleaned.description = variant.description;
        }
        return cleaned;
      }
      return variant;
    });
  }

  // Flatten schemas with both anyOf and properties into allOf form
  // (serde flatten produces flat, Zod .and() produces allOf)
  if (result.anyOf && result.properties) {
    const anyOfPart = { anyOf: result.anyOf };
    const objectPart: any = { type: result.type || "object", properties: result.properties };
    if (result.required) objectPart.required = result.required;

    const allOfResult: any = { allOf: [anyOfPart, objectPart] };
    // Keep top-level metadata
    if (result.title) allOfResult.title = result.title;
    if (result.description) allOfResult.description = result.description;
    return allOfResult;
  }

  return result;
}

/** Sort `required` arrays for order-independent comparison. */
function sortRequiredArrays(obj: any): any {
  if (obj === null || typeof obj !== "object") return obj;
  if (Array.isArray(obj)) return obj.map(sortRequiredArrays);

  const result: any = {};
  for (const [key, value] of Object.entries(obj)) {
    if (key === "required" && Array.isArray(value)) {
      result[key] = [...value].sort();
    } else {
      result[key] = sortRequiredArrays(value);
    }
  }
  return result;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("Zod roundtrip", async () => {
  // Dynamically import all generated schemas
  const modules = await Promise.all([
    import("./agent/index"),
    import("./auth/index"),
    import("./ensemble/index"),
    import("./functions/index"),
    import("./vector/index"),
    import("./prefixedUuid"),
    import("./responseError"),
  ]);

  const zodSchemas = new Map<string, z.ZodType>();
  for (const mod of modules) {
    for (const [key, value] of Object.entries(mod)) {
      if (key.endsWith("Schema") && value instanceof z.ZodType) {
        zodSchemas.set(key, value);
      }
    }
  }

  const jsonSchemas = loadJsonSchemas();
  const allTitles = new Set(jsonSchemas.keys());

  // Build map of title → description for distinguishing own vs usage-site descriptions
  const titleDescriptions = new Map<string, string>();
  for (const [title, schema] of jsonSchemas) {
    if (schema.description) {
      titleDescriptions.set(title, schema.description);
    }
  }

  for (const [title, originalJson] of jsonSchemas) {
    const schemaName = titleToPascal(title) + "Schema";

    it(`${title}`, () => {
      const zodSchema = zodSchemas.get(schemaName);
      expect(
        zodSchema,
        `Missing Zod schema export: ${schemaName}`,
      ).toBeDefined();

      const converted = zodToJsonSchema(zodSchema!, allTitles, title, titleDescriptions);
      const expected = normalizeOriginal(originalJson, titleDescriptions, jsonSchemas);
      expect(sortRequiredArrays(converted)).toEqual(sortRequiredArrays(expected));
    });
  }
});
