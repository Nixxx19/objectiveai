<!-- This file documents the builder's behavior. If the builder crate changes (normalizations, key ordering, guarantees), update this file to match. -->

# objectiveai-json-schema

Generated JSON Schema files for every public serializable type in `objectiveai-rs`. Each file is named `{title}.json` where the title uses dot-separated module paths (e.g., `functions.executions.RetryToken.json`).

## Rebuilding

```bash
cargo run --package objectiveai-json-schema-builder
```

The builder (`builder/src/main.rs`) calls `objectiveai::json_schemas()`, normalizes each schema, orders keys canonically, and writes one file per type. It clears the output directory (except `builder/`) on each run.

## Builder Normalizations

These transformations are applied by `normalize()` in `main.rs` before writing:

1. **Remove `$defs`** — All definitions are flattened; `$defs` sections are deleted.
2. **Remove `$schema`** — The meta-schema URI is stripped.
3. **Convert `oneOf` to `anyOf`** — All `oneOf` arrays become `anyOf`.
4. **Flatten single-variant `anyOf`** — If `anyOf` has exactly one variant, its keys are merged into the parent object (the `anyOf` wrapper is removed).
5. **Rewrite `$ref` targets** — `"#"` becomes the schema's own title (self-reference). `"#/$defs/Name"` becomes the bare type name `"Name"`.
6. **Remove `required`** — All `required` arrays are stripped.
7. **Convert `const` to single-element `enum`** — `{"const": "x"}` becomes `{"enum": ["x"]}`.
8. **Convert nullable type arrays to `anyOf`** — `{"type": ["string", "null"]}` becomes `{"anyOf": [{"type": "string"}, {"type": "null"}]}`. Type-specific constraints (`items`, `properties`, `additionalProperties`, `minimum`, `maximum`, `format`, `pattern`, `minItems`, `maxItems`, `enum`) move into the non-null variant. Metadata keys (`description`, `default`, etc.) stay on the outer object.
9. **Resolve numeric `format` to explicit bounds** — For `type: "integer"`, the format string (e.g., `"int8"`, `"uint64"`) is resolved to `minimum`/`maximum` values matching the Rust integer type's range, then `format` is removed. For `type: "number"`, bounds default to `f32::MIN`/`f32::MAX` and `format` is removed. Pre-existing `minimum`/`maximum` values are preserved (not overwritten).

## Key Ordering

Applied by `order_keys()` in `main.rs` after normalization:

**Outside `properties`:** Keys are sorted by canonical position:

```
title, description, type, enum, anyOf, $ref, properties,
additionalProperties, items, minItems, maxItems, minimum,
maximum, pattern, format, default
```

Unknown keys (not in this list) sort to the end.

**Inside `properties`:** Keys (field names) are sorted alphabetically.

This ordering is applied recursively at every level of nesting.

## Guarantees (enforced by `builder/tests/schema_properties.rs`)

All guarantees below apply **outside of `properties` objects**. Inside `properties`, keys are user-defined field names and are not subject to keyword-level constraints (but their values — the sub-schemas — are).

| # | Guarantee | Test |
|---|-----------|------|
| 1 | Only 16 allowed keywords: `title`, `description`, `type`, `enum`, `anyOf`, `$ref`, `properties`, `additionalProperties`, `items`, `minItems`, `maxItems`, `minimum`, `maximum`, `pattern`, `format`, `default` | `only_allowed_keywords` |
| 2 | Keywords appear in canonical order (see above) | `keywords_in_canonical_order` |
| 3 | Property keys inside `properties` are sorted alphabetically | `properties_keys_sorted_alphabetically` |
| 4 | No `$schema` keyword anywhere | `no_schema_property` |
| 5 | `type` is always a string, never an array | `no_type_arrays_outside_properties` |
| 6 | No `required` keyword | `no_required_outside_properties` |
| 7 | No `oneOf` keyword | `no_one_of_outside_properties` |
| 8 | `anyOf` and `$ref` never coexist as siblings on the same object | `no_any_of_with_sibling_ref` |
| 9 | No `const` keyword | `no_const_outside_properties` |
| 10 | No `format` on `type: "integer"` or `type: "number"` (bounds are explicit via `minimum`/`maximum`) | `no_numeric_format` |
| 11 | `minimum` never exceeds `maximum` | `minimum_never_exceeds_maximum` |
| 12 | Every `$ref` target resolves to an existing schema title | `all_refs_resolve` |
| 13 | `anyOf` inside `properties` is exactly 2 variants: one non-null type + `{"type": "null"}` (multi-variant unions only at root) | `anyof_in_properties_is_nullable_only` |

## Implications for SDK Code Generators

Code generators (Python, TypeScript, etc.) that consume these schemas can rely on:

- **No `$defs`/`$schema`/`required`/`oneOf`/`const`** — These never appear.
- **No type arrays** — Nullability is always expressed as `anyOf: [{type: T}, {type: "null"}]`.
- **No numeric format** — Integer/number types always have explicit `minimum`/`maximum` instead.
- **Clean `$ref` targets** — Always bare type names (e.g., `"agent.Agent"`), never JSON Pointer paths.
- **Deterministic key order** — Both keyword order and property field order are stable and predictable.
- **No ambiguous union+ref** — An object never has both `anyOf` and `$ref`.
- **No multi-variant unions in properties** — `anyOf` inside `properties` is always nullable (exactly 2 variants, one being `{"type": "null"}`). Multi-variant `anyOf` only appears at the root level.
