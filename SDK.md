# SDK Strategy

## Overview

All SDKs are auto-generated from a single source of truth: the JSON Schema files in `objectiveai-json-schema/`. Each SDK has a generation script that reads these schemas and produces idiomatic types, and a roundtrip test that verifies the generated types perfectly reconstruct the original schemas.

## JSON Schema Source

`objectiveai-json-schema/` contains ~317 `.json` files, one per public serializable type in `objectiveai-rs`. Names use dot-separated module paths (e.g., `functions.executions.RetryToken.json`).

**Rebuilding schemas:**
```bash
cargo run --package objectiveai-json-schema-builder
```

The builder enforces 12 structural guarantees (canonical key order, no `$defs`, no `oneOf`, all `$ref` targets resolve, etc.) — see [`objectiveai-json-schema/builder/tests/schema_properties.rs`](objectiveai-json-schema/builder/tests/schema_properties.rs).

## Auto-Generation

Each SDK has a script that reads the JSON schemas and generates idiomatic types.

| SDK | Script | Generates |
|-----|--------|-----------|
| TypeScript | [`objectiveai-js/scripts/install-zod.cjs`](objectiveai-js/scripts/install-zod.cjs) | Zod schemas + barrel exports (`generatedIndex.ts`) |
| Python | [`objectiveai-py/scripts/install_pydantic.py`](objectiveai-py/scripts/install_pydantic.py) | Pydantic models + `__init__.py` barrel exports |

## Roundtrip Testing

Each SDK has a test harness (forbidden from modification) and a roundtrip test that converts generated types back to JSON Schema and asserts equality with the originals.

| SDK | Harness | Test |
|-----|---------|------|
| TypeScript | [`objectiveai-js/src/tests/test-zod-roundtrip-harness.ts`](objectiveai-js/src/tests/test-zod-roundtrip-harness.ts) | [`objectiveai-js/src/tests/test-zod-roundtrip.test.ts`](objectiveai-js/src/tests/test-zod-roundtrip.test.ts) |
| Python | [`objectiveai-py/tests/test_pydantic_roundtrip_harness.py`](objectiveai-py/tests/test_pydantic_roundtrip_harness.py) | [`objectiveai-py/tests/test_pydantic_roundtrip.py`](objectiveai-py/tests/test_pydantic_roundtrip.py) |

The tests are 100% generic — no schema-specific logic or hardcoded titles. If a generated type doesn't perfectly reconstruct its source schema, the test fails.

## Adding a New SDK

1. Write a generation script that reads `objectiveai-json-schema/*.json` and produces idiomatic types for the target language
2. Write a roundtrip test harness that loads original schemas and provides an assertion function
3. Write a roundtrip test that converts generated types back to JSON Schema and asserts equality
4. The generation script and roundtrip test should be fully generic — they must work for all schemas without any schema-specific logic
