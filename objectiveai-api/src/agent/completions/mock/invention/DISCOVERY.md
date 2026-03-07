# Mock Invention Step Discovery

How to determine which `InventionStep` variant we're in, given only the
invention tool names (and optionally the prompt text).

## Tool name conventions

Schema tools have the prefix `Read` and suffix `JsonSchema`, e.g.
`ReadObjectInputSchemaJsonSchema`. Invention tools from the state machine
have short names: `ReadSpec`, `WriteEssay`, `ReadEssay`, `WriteInputSchema`,
`ReadInputSchema`, `WriteEssayTasks`, `ReadEssayTasks`, `AppendTask`,
`DeleteTask`, `ReadTask`, `ReadTasksLength`, `CheckFunction`,
`WriteDescription`, `ReadDescription`.

Schema tool names are generated from the Zod schema title with
`Read{Title}JsonSchema`. The relevant schema tools per step are:

- `ReadObjectInputSchemaJsonSchema` — scalar input_schema step
- `ReadAlphaVectorFunctionInputSchemaJsonSchema` — vector input_schema step
- `ReadAlphaScalarVectorCompletionTaskExpressionJsonSchema` — scalar leaf tasks step
- `ReadAlphaScalarPlaceholderScalarFunctionTaskExpressionJsonSchema` — scalar branch tasks step
- `ReadAlphaVectorVectorCompletionTaskExpressionJsonSchema` — vector leaf tasks step
- `ReadAlphaVectorPlaceholderVectorFunctionTaskExpressionJsonSchema` — vector branch tasks step
- `ReadAlphaVectorPlaceholderScalarFunctionTaskExpressionJsonSchema` — also present in vector branch tasks step
- `ReadMessagesJsonSchema` — leaf tasks steps (both scalar and vector)
- `ReadVectorResponsesJsonSchema` — leaf tasks steps (both scalar and vector)
- `ReadInputValueJsonSchema` — branch tasks steps (both scalar and vector)

Note: schema tools may have transitive `$ref` dependencies that add extra
`Read*JsonSchema` tools. Discovery should match on the primary schema tool
names listed above rather than trying to enumerate all transitive deps.

## Step detection (by tool names)

### 1. WriteEssay present, WriteInputSchema absent → Essay step

- **Scalar vs Vector**: Cannot distinguish from tools alone. Both get
  `[ReadSpec, WriteEssay]`. Must inspect prompt for "Scalar Function" vs
  "Vector Function".

### 2. WriteInputSchema present → InputSchema step

- **Scalar**: `ReadObjectInputSchemaJsonSchema` present among tool names.
- **Vector**: `ReadAlphaVectorFunctionInputSchemaJsonSchema` present.

### 3. WriteEssayTasks present, AppendTask absent → EssayTasks step

- Tools: `[ReadSpec, ReadEssay, ReadInputSchema, WriteEssayTasks]`
- Identical across all 4 routes. Cannot distinguish scalar/vector or
  leaf/branch from tools or prompt.

### 4. AppendTask present → Tasks step

This is the most differentiable step. Presence of specific schema tools
determines the exact route:

- **Scalar Leaf**: `ReadAlphaScalarVectorCompletionTaskExpressionJsonSchema`
  present. Also has `ReadMessagesJsonSchema`, `ReadVectorResponsesJsonSchema`.
- **Scalar Branch**: `ReadAlphaScalarPlaceholderScalarFunctionTaskExpressionJsonSchema`
  present. Also has `ReadInputValueJsonSchema`.
- **Vector Leaf**: `ReadAlphaVectorVectorCompletionTaskExpressionJsonSchema`
  present. Also has `ReadMessagesJsonSchema`, `ReadVectorResponsesJsonSchema`.
- **Vector Branch**: `ReadAlphaVectorPlaceholderVectorFunctionTaskExpressionJsonSchema`
  present. Also has `ReadAlphaVectorPlaceholderScalarFunctionTaskExpressionJsonSchema`
  and `ReadInputValueJsonSchema`.

Simplified decision tree for Tasks step:
1. Has `ReadMessagesJsonSchema`? → Leaf. Check for `AlphaScalar` vs `AlphaVector`
   in the task expression schema tool name.
2. Has `ReadInputValueJsonSchema`? → Branch. Check for `AlphaScalar` vs `AlphaVector`
   in the placeholder task expression schema tool name.

### 5. WriteDescription present → Description step

- Tools: `[ReadSpec, ReadEssay, ReadInputSchema, ReadEssayTasks, ReadTask,
  ReadTasksLength, WriteDescription]`
- Identical across all 4 routes. Cannot distinguish.

### 6. Readme step

No invention tools are provided for the readme step — it is not an agent
completion step. `write_readme` is called directly on the state. The mock
client will never see this step.

## Summary table

| Step        | Key tool(s)            | Scalar vs Vector          | Leaf vs Branch               |
|-------------|------------------------|---------------------------|------------------------------|
| Essay       | WriteEssay             | Prompt only               | N/A                          |
| InputSchema | WriteInputSchema       | Schema tool name          | N/A                          |
| EssayTasks  | WriteEssayTasks        | Indistinguishable         | Indistinguishable            |
| Tasks       | AppendTask             | Schema tool name          | Schema tool name             |
| Description | WriteDescription       | Indistinguishable         | Indistinguishable            |
| Readme      | (none)                 | N/A                       | N/A                          |
