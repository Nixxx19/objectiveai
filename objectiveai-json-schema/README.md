# ObjectiveAI JSON Schema

Generated JSON Schema files for every public serializable type in `objectiveai-rs`.

## Structure

Each type gets a `{title}.json` file where the title uses dot-separated module paths:

```
agent.Agent.json
agent.completions.message.Message.json
functions.executions.RetryToken.json
vector.completions.VectorCompletion.json
```

Schemas have `$defs` stripped and `$ref` targets rewritten to bare type names (e.g. `"$ref": "agent.AgentBase"` instead of `"$ref": "#/$defs/agent.AgentBase"`).

## Rebuilding

```bash
cargo run --package objectiveai-json-schema-builder
```

The builder:
1. Clears all files in this directory (except `builder/`)
2. Calls `objectiveai::json_schemas()` to get all schemas
3. Strips `$defs` and rewrites `$ref` targets
4. Writes each schema as `{title}.json`

## Builder

Source: `builder/src/main.rs`
