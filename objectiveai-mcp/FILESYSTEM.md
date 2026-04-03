# Claude Code Tools

Tool definitions from the [Claude Code Rust implementation](https://github.com/soongenwong/claudecode).

Source file: `rust/crates/tools/src/lib.rs`

## bash

Execute a shell command in the current workspace.

- **Description:** [L220](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L220)
- **Input Schema:** [L221-L232](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L221-L232)

## read_file — Read

Read a text file from the workspace.

- **Description:** [L237](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L237)
- **Input Schema:** [L238-L247](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L238-L247)

## write_file — Write

Write a text file in the workspace.

- **Description:** [L252](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L252)
- **Input Schema:** [L253-L261](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L253-L261)

## edit_file — Edit

Replace text in a workspace file.

- **Description:** [L266](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L266)
- **Input Schema:** [L267-L277](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L267-L277)

## glob_search — Glob

Find files by glob pattern.

- **Description:** [L282](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L282)
- **Input Schema:** [L283-L291](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L283-L291)

## grep_search — Grep

Search file contents with a regex pattern.

- **Description:** [L296](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L296)
- **Input Schema:** [L297-L316](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L297-L316)

## WebFetch

Fetch a URL, convert it into readable text, and answer a prompt about it.

- **Description:** [L322-L323](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L322-L323)
- **Input Schema:** [L324-L332](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L324-L332)

## WebSearch

Search the web for current information and return cited results.

- **Description:** [L337](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L337)
- **Input Schema:** [L338-L353](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L338-L353)

## TodoWrite

Update the structured task list for the current session.

- **Description:** [L358](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L358)
- **Input Schema:** [L359-L382](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L359-L382)

## Skill

Load a local skill definition and its instructions.

- **Description:** [L386](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L386)
- **Input Schema:** [L387-L395](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L387-L395)

## Agent

Launch a specialized agent task and persist its handoff metadata.

- **Description:** [L400](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L400)
- **Input Schema:** [L401-L412](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L401-L412)

## ToolSearch

Search for deferred or specialized tools by exact name or keywords.

- **Description:** [L417](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L417)
- **Input Schema:** [L418-L426](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L418-L426)

## NotebookEdit

Replace, insert, or delete a cell in a Jupyter notebook.

- **Description:** [L431](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L431)
- **Input Schema:** [L432-L443](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L432-L443)

## Sleep

Wait for a specified duration without holding a shell process.

- **Description:** [L448](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L448)
- **Input Schema:** [L449-L456](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L449-L456)

## SendUserMessage

Send a message to the user.

- **Description:** [L461](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L461)
- **Input Schema:** [L462-L477](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L462-L477)

## Config

Get or set Claude Code settings.

- **Description:** [L482](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L482)
- **Input Schema:** [L483-L493](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L483-L493)

## StructuredOutput

Return structured output in the requested format.

- **Description:** [L498](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L498)
- **Input Schema:** [L499-L502](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L499-L502)

## REPL

Execute code in a REPL-like subprocess.

- **Description:** [L507](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L507)
- **Input Schema:** [L508-L517](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L508-L517)

## PowerShell

Execute a PowerShell command with optional timeout.

- **Description:** [L522](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L522)
- **Input Schema:** [L523-L534](https://github.com/soongenwong/claudecode/blob/main/rust/crates/tools/src/lib.rs#L523-L534)
