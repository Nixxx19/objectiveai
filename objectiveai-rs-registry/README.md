# objectiveai-rs-registry

Auto-generated JSON registry of every public `struct`, `enum`, and `type` alias in `objectiveai-rs`.

## Structure

Mirrors `objectiveai-rs/src/` — each `.rs` file with public types gets a corresponding `.rs.json` file:

```
objectiveai-rs-registry/
├── builder/                        # Rust binary that generates the registry
│   └── src/main.rs
└── src/                            # Generated output (mirrors objectiveai-rs/src/)
    ├── ensemble/
    │   └── ensemble.rs.json
    ├── functions/
    │   ├── function.rs.json
    │   └── check/
    │       └── example_inputs/
    │           ├── array.rs.json
    │           └── ...
    └── ...
```

## Entry format

Each JSON file is an array of entries:

```json
[
  {
    "name": "Ensemble",
    "full_name": "EnsembleEnsembleEnsemble",
    "kind": "struct",
    "path": "objectiveai-rs/src/ensemble/ensemble.rs",
    "line_start": 33,
    "line_end": 38
  }
]
```

| Field | Description |
|-------|-------------|
| `name` | Type name as written in source |
| `full_name` | PascalCased module path + name for disambiguation (e.g., `FunctionsCheckExampleInputsFileGenerator`) |
| `kind` | `struct`, `enum`, or `type` |
| `path` | Source file path relative to repo root |
| `line_start` | First line of the definition |
| `line_end` | Last line of the definition |

## Rebuilding

```bash
cargo run --package objectiveai-rs-registry-builder
```

The builder:
1. Wipes `objectiveai-rs-registry/src/` to prevent orphan files from deleted modules
2. Parses every `.rs` file in `objectiveai-rs/src/` using `syn` (Rust AST parser, no regexes)
3. Extracts all public structs, enums, and type aliases
4. Writes JSON files mirroring the source directory structure

## Searching

Find where a type is defined:

```bash
grep -r '"name": "Ensemble"' objectiveai-rs-registry/src/
```

Find all types in a module:

```bash
cat objectiveai-rs-registry/src/ensemble/ensemble.rs.json
```

## Limitations

- Macro-generated types (e.g., types produced inside `macro_rules!` invocations) are not visible to the parser. Types decorated with derive/attribute macros are handled fine.
