# ObjectiveAI

**The agentic collective judgment harness.**

[Website](https://objectiveai.dev) · [Discord](https://discord.gg/gbNFHensby) · [GitHub](https://github.com/ObjectiveAI/objectiveai)

[![Release](https://img.shields.io/github/v/release/ObjectiveAI/objectiveai?label=release&color=blue)](https://github.com/ObjectiveAI/objectiveai/releases/latest)
[![Crates.io](https://img.shields.io/crates/v/objectiveai-sdk?label=crates.io%20%2F%20objectiveai-sdk)](https://crates.io/crates/objectiveai-sdk)
[![npm](https://img.shields.io/npm/v/@objectiveai/sdk?label=npm%20%2F%20%40objectiveai%2Fsdk)](https://www.npmjs.com/package/@objectiveai/sdk)
[![PyPI](https://img.shields.io/pypi/v/objectiveai-sdk?label=pypi%20%2F%20objectiveai-sdk)](https://pypi.org/project/objectiveai-sdk/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## Packages

SDKs published to language-native registries. Pick the one for your stack:

| Language | Package | Install |
|---|---|---|
| Rust | [`objectiveai-sdk`](https://crates.io/crates/objectiveai-sdk) | `cargo add objectiveai-sdk` |
| TypeScript | [`@objectiveai/sdk`](https://www.npmjs.com/package/@objectiveai/sdk) | `npm i @objectiveai/sdk` |
| Python | [`objectiveai-sdk`](https://pypi.org/project/objectiveai-sdk/) | `pip install objectiveai-sdk` |
| Go | [`objectiveai-sdk-go`](https://pkg.go.dev/github.com/ObjectiveAI/objectiveai/objectiveai-sdk-go) | `go get github.com/ObjectiveAI/objectiveai/objectiveai-sdk-go` |

Additional crates on crates.io: [`objectiveai-api`](https://crates.io/crates/objectiveai-api), [`objectiveai-cli`](https://crates.io/crates/objectiveai-cli), [`objectiveai-mcp-cli`](https://crates.io/crates/objectiveai-mcp-cli), [`objectiveai-mcp-proxy`](https://crates.io/crates/objectiveai-mcp-proxy), [`objectiveai-mcp-filesystem`](https://crates.io/crates/objectiveai-mcp-filesystem), [`objectiveai-sdk-macros`](https://crates.io/crates/objectiveai-sdk-macros). Additional PyPI package: [`objectiveai-cocoindex`](https://pypi.org/project/objectiveai-cocoindex/).

## Binaries

Install all four prebuilt binaries with one command:

```bash
curl -fsSL https://raw.githubusercontent.com/ObjectiveAI/objectiveai/main/install.sh | bash
. "$HOME/.objectiveai/env"
```

| Binary | What it does | Download |
|---|---|---|
| `objectiveai` | CLI + embedded viewer | [latest](https://github.com/ObjectiveAI/objectiveai/releases/latest) |
| `objectiveai-api` | API server | [latest](https://github.com/ObjectiveAI/objectiveai/releases/latest) |
| `objectiveai-viewer` | Standalone Tauri desktop app | [latest](https://github.com/ObjectiveAI/objectiveai/releases/latest) |
| `objectiveai-mcp` | MCP server (streamable HTTP) | [latest](https://github.com/ObjectiveAI/objectiveai/releases/latest) |

Supported platforms: Linux x86_64, Linux aarch64, macOS x86_64, macOS aarch64, Windows x86_64. See [Binaries & self-hosting](#binaries--self-hosting) for install flags and per-binary detail.

---

## What ObjectiveAI is

ObjectiveAI is an agentic collective judgment harness. It turns decisions — ranking candidates, classifying inputs, scoring outputs — into composable, trainable, reusable infrastructure backed by swarms of language models.

The motivating problem: a single model asked to pick the best answer from a list makes a discrete choice and discards everything else. The model had a full probability distribution over all options; the sampler collapsed it to one. That lost information is exactly what matters for calibration, ensembling, and learning. ObjectiveAI recovers it.

### The core arc

An agent has a decision to make. Instead of calling a model directly, it calls a **Function**. The Function fans out to a **Swarm** — a fixed collection of configured models called Agents. Each Agent votes on the available options. Votes combine using **learned weights** to produce a **score vector**: one score per option, summing to 1.

```
Agent needs a decision
        │
        ▼
  ┌─────────────┐
  │  Function   │  (composable, content-addressed, versioned)
  └──────┬──────┘
         │ fans out
         ▼
  ┌──────────────────────────────────┐
  │              Swarm               │
  │  ┌────────┐ ┌────────┐ ┌──────┐ │
  │  │ Agent  │ │ Agent  │ │ ...  │ │
  │  └───┬────┘ └───┬────┘ └──┬───┘ │
  └──────┼──────────┼─────────┼─────┘
         │  votes   │         │
         ▼          ▼         ▼
  ┌────────────────────────────────┐
  │  weighted combination (Profile)│
  └────────────────────────────────┘
         │
         ▼
  scores: [0.61, 0.28, 0.11]  (sums to 1)
         │
         ▼
  Agent picks — or delegates further
```

The output is not a single answer. It is a probability-like score vector over all options. The caller can threshold it, chain it into another Function, or use it raw.

### Why single-model picking loses

When a language model samples a discrete token — "Option A" — it throws away everything it computed about Options B through Z. Those log-probabilities encode real signal: how confident the model was, where it hedged, what it considered close.

ObjectiveAI bypasses the sampler. It reads the model's log-probability distribution directly, using a prefix tree to structure options so they fall within the model's logprob window. For a set of N responses, the tree assigns each option a unique decodable prefix; the model's probability mass over those prefixes becomes its vote vector. No discrete collapse. No lost signal.

This matters twice over: once per model, and once across models. Different models have different failure modes, different training distributions, different calibration profiles. Combining them with learned weights — weights that can be trained against ground truth — is strictly more powerful than picking the one model that scores highest on average.

### Functions are infrastructure

A Function is not a one-off API call. It is a reusable, composable decision pipeline:

- **Recursive** — Functions can call other Functions as subtasks. A scoring pipeline for a complex input can delegate subproblems to specialized sub-Functions.
- **Content-addressed** — Function IDs are deterministic hashes of their definitions. The same definition always resolves to the same ID; changes produce new IDs automatically.
- **Version-tracked** — Functions are referenced by ID and hosted on GitHub. You can pin to a commit SHA. The definition you ran six months ago is still reproducible.
- **Trainable** — A Profile is learned weights over a Function's task structure. Training provides labeled data; ObjectiveAI adjusts weights to minimize loss. The models themselves never change — only how their votes are combined.
- **Composable** — Functions compose into pipelines. The score vector from one Function becomes the input to the next. Swiss System tournament execution (`input_split` / `input_merge`) handles bracket-style elimination across large option sets.

The result is decision infrastructure that accumulates quality over time. A Function built today can be trained, improved, pinned to a known-good Profile, and reused across every agent in a system — or published for others to import by ID.

## Quick start

Install the CLI, API server, viewer, and MCP server from the latest release:

```bash
curl -fsSL https://raw.githubusercontent.com/ObjectiveAI/objectiveai/main/install.sh | bash
. "$HOME/.objectiveai/env"
```

Set your API key:

```bash
objectiveai api headers x-objectiveai-authorization config set "apk_your_key_here"
```

### CLI — run a vector completion

A function wraps a vector completion task and a profile specifies the swarm of agents. Both can be supplied inline as JSON. The example below asks two models to vote on which of three answers is best and returns a score vector:

```bash
objectiveai functions executions create standard \
  --function-inline '{
    "type": "vector.function",
    "tasks": [{
      "type": "vector.completion",
      "messages": [{"role": "user", "content": "Which capital city is largest by population?"}],
      "responses": ["Tokyo", "London", "Berlin"],
      "output": {"$jmespath": "output.scores"}
    }]
  }' \
  --profile-inline '{
    "agents": [
      {"upstream": "openrouter", "model": "openai/gpt-4o-mini"},
      {"upstream": "openrouter", "model": "anthropic/claude-3-haiku"}
    ]
  }' \
  --input-inline 'null'
```

The streamed output ends with a notification containing the scores vector:

```json
{"Notification":{"value":{"execution":{"output":{"Vector":[0.82,0.14,0.04]}}}}}
```

Each number is the combined vote share for that response, in the same order as `responses`. Values sum to 1.

### SDK — TypeScript

```typescript
import { ObjectiveAI, functionsExecutionsCreateFunctionExecution } from "@objectiveai/sdk";

const client = new ObjectiveAI({ authorization: process.env.OBJECTIVEAI_AUTHORIZATION });

const result = await functionsExecutionsCreateFunctionExecution(client, {
  function: {
    type: "vector.function",
    tasks: [{
      type: "vector.completion",
      messages: [{ role: "user", content: "Which capital city is largest by population?" }],
      responses: ["Tokyo", "London", "Berlin"],
      output: { $jmespath: "output.scores" },
    }],
  },
  profile: {
    agents: [
      { upstream: "openrouter", model: "openai/gpt-4o-mini" },
      { upstream: "openrouter", model: "anthropic/claude-3-haiku" },
    ],
  },
  input: null,
  stream: false,
});

console.log(result.output); // { Vector: [0.82, 0.14, 0.04] }
```

See [Core primitives](#core-primitives) for a full explanation of Agents, Swarms, Functions, and Profiles, and [SDKs](#sdks) for Python, Rust, Go, and .NET patterns including streaming.

## Core primitives

### Agents

An **Agent** is a fully-specified configuration of a single upstream model: model identity, prompt structure, decoding parameters, output mode, and provider preferences. Agents are content-addressed via XXHash3-128 — the same configuration always produces the same 22-character base62 ID. IDs are deterministic because the serialized configuration is hashed after normalization (empty fields stripped, defaults canonicalized). Two Agents with identical effective settings are the same Agent.

Agents are stored as `agent.json` in Git repositories and referenced by `owner/repo@commit`. They can also be defined inline anywhere a swarm or function accepts an agent slot.

```json
{
  "description": "Skeptical evaluator",
  "upstream": "openrouter",
  "model": "openai/gpt-4o",
  "output_mode": "json_schema",
  "temperature": 0.2,
  "top_logprobs": 20,
  "prefix_messages": [
    { "role": "system", "content": "You are a rigorous critic. Challenge assumptions." }
  ]
}
```

Each upstream (OpenRouter, Claude Agent SDK, Codex SDK) has its own agent type with its own parameter set. The `output_mode` field controls how the agent structures its vote during vector completions and is ignored for agent completions.

### Swarms

A **Swarm** is an ordered collection of Agents used together for collective judgment. Swarms are immutable and content-addressed — their ID is computed from the sorted `(full_id, count)` pairs of their constituent agents. Weights are **not** baked into the swarm definition; they are execution-time parameters supplied by a Profile or passed directly.

Each agent slot has a `count` (number of instances) and optional fallbacks. Duplicate agents are merged and their counts summed. The total agent count across all slots must be between 1 and 128.

```json
{
  "description": "Balanced judgment panel",
  "agents": [
    {
      "upstream": "openrouter",
      "model": "openai/gpt-4o",
      "output_mode": "json_schema",
      "prefix_messages": [
        { "role": "system", "content": "You are a rational skeptic. Ground every choice in logic." }
      ],
      "count": 2
    },
    {
      "upstream": "openrouter",
      "model": "anthropic/claude-sonnet-4-20250514",
      "output_mode": "tool_call",
      "suffix_messages": [
        { "role": "system", "content": "You are an intuitive thinker. Trust your instincts." }
      ],
      "count": 1
    }
  ]
}
```

Swarms are stored as `swarm.json` in Git repositories and shared across functions. Because weights are external, the same swarm can be reused with different weight configurations without creating a new swarm.

### Vector completions

Vector completions are the core execution primitive. Given a prompt and a set of candidate responses, each agent in the swarm votes for the response it judges best. Votes are combined with weights to produce a score vector — one score per response, summing to 1.

Two distinct vectors are present in every result:

- **`weights` vector** — total weight allocated to each response option, reflecting which responses received more agent attention.
- **`scores` vector** — final normalized scores after combining votes with agent weights. Always sums to 1. This is what callers use.

For discrete votes, an agent's full weight goes to its selected response. For probabilistic votes (via logprobs), weight is divided across responses according to the model's probability distribution.

```text
Prompt:    "Which approach best handles edge cases?"
Responses: ["defensive coding", "fuzzing", "formal verification", "property testing"]

Agent votes (weights = [1.0, 1.0, 1.0]):
  GPT-4o #1  -> "property testing"   (p=0.61), "formal verification" (p=0.28), ...
  GPT-4o #2  -> "formal verification" (p=0.55), "property testing"  (p=0.31), ...
  Claude     -> "property testing"   (p=0.72), "fuzzing"            (p=0.18), ...

Scores: [0.05, 0.11, 0.31, 0.53]
```

Responses can be text, images, video, audio, or files. The same mechanism applies regardless of modality.

#### Probabilistic voting via logprobs

Standard LLM sampling is lossy. When a model is 70% confident in option A and 30% confident in option B, the sampler forces a single discrete output — one of those two signals is discarded. ObjectiveAI bypasses the sampler entirely using logprobs.

Instead of asking each model "which option do you pick?" and getting one answer, ObjectiveAI extracts the model's full probability distribution over the response options simultaneously via `top_logprobs`. Each agent's vote becomes a probability vector rather than a point choice. A model that weakly prefers option A contributes proportionally less signal than one that strongly prefers it.

```text
Traditional:  Model outputs "A"            — loses the 30% signal for B entirely
ObjectiveAI:  Model vote = [0.70, 0.30, 0.00, 0.00] — full distribution preserved
```

For response sets larger than the logprobs limit (typically 20), a **prefix tree** structures the problem across multiple stages. Each response is assigned a unique prefix key (`` `A` ``, `` `B` ``, … `` `T` ``). The tree width matches the logprobs count — 20 leaves per branch. For larger sets, nested prefixes (`` `A``A` ``, `` `A``B` ``) extend coverage hierarchically. At each node, the model's logprob distribution over the next character captures its preferences at that level. The full tree is traversed in a single pass per agent, preserving probability information at every level. Implemented in `objectiveai-api/src/vector/completions/pfx.rs`. Supports voting over hundreds of options while respecting the logprobs API constraint.

The result is collective judgment that uses richer information than any individual model's sampled output would provide.

### Functions

**Functions** are composable scoring pipelines: data in, scores out. A Function is a list of **tasks** executed against an input. Each task is one of:

- A **vector completion** — runs the swarm and produces a score.
- A **nested function call** — references another `function.json` by `owner/repo@commit`.
- A **mapped operation** — runs a task N times over an indexed range, producing N outputs.

Functions are recursive: a function's tasks can themselves be functions, which can themselves contain vector completions or more nested functions. The composition is arbitrarily deep.

Functions produce either:

- **Scalar** — a single score in [0, 1].
- **Vector** — an array of scores summing to 1, one per output dimension.

The final output is the weighted average of all task outputs, with weights supplied by a Profile. Tasks carry `output` expressions (JMESPath or Starlark) that transform raw task results into the function's output type before averaging.

Functions are stored as `function.json` in Git repositories and referenced by `owner/repo` triple. They are content-addressed via their task structure and input schema.

```json
{
  "type": "alpha.scalar.leaf.function",
  "description": "Score response quality on a 0-1 scale",
  "input_schema": { "type": "object", "properties": { "response": { "type": "string" } } },
  "tasks": [
    {
      "type": "vector.completion",
      "messages": [{ "role": "user", "content": "Rate this response: {{input.response}}" }],
      "responses": ["poor", "mediocre", "good", "excellent"],
      "output": { "$starlark": "output['scores'][2] + output['scores'][3]" }
    }
  ]
}
```

### Profiles

ObjectiveAI does not fine-tune models. It learns weights.

A Profile is the result of training: given a dataset of `(input, expected_output)` pairs, ObjectiveAI executes the Function repeatedly, computes loss against expected outputs, and adjusts the weights over each task until they converge. The learned configuration — which tasks to trust more, which to discount — is stored as `profile.json`.

Profiles are GitHub-hosted and referenced by `owner/repo@commit`. Pinning a commit SHA is strongly recommended: the Profile's shape (number of tasks, their order) is tied to the function it was trained on, and that function may evolve. A mismatched Profile silently produces wrong weights.

```json
{
  "owner": "ObjectiveAI",
  "repo": "quality-scorer",
  "commit": "a3f8c21d..."
}
```

At execution time, the Function and Profile are independent inputs. The retrieval system fetches both, resolves the resource graph, and applies the learned weights to combine task outputs.

### The resource graph

All resources reference each other via `(owner, repository, commit)` triples. Content-addressing plus commit pinning makes the full graph reproducible from any entry point.

```text
agent.json  <-  swarm.json  <-  profile.json      function.json
                 (agents)        (swarms+weights)   (tasks + input_schema)

At execution:  function.json + profile.json  ->  scores
```

The Function and Profile are deliberately separate files. The same Function can be run with different Profiles (e.g. a domain-specific profile vs. a general-purpose profile). The same Profile cannot be applied to a structurally different Function — the task count and order must match.

Remote references resolve lazily: the retrieval system walks the graph starting from the execution request, fetching and caching each resource exactly once. Deduplication is by `(owner, repo, commit)` triple. All fetches are content-verified — a cached resource is never re-fetched if the commit SHA matches. Even deeply nested function graphs execute with minimal network overhead.

## Function invention

An agent facing a new decision problem can ask ObjectiveAI to build the scoring pipeline for it. Invention takes a natural-language description — a `spec` — and an optional set of examples, then runs a five-step agentic process (essay → input schema → essay tasks → tasks → description) that produces a complete, valid `function.json`: typed input schema, task tree with expressions, and description. The output is ready to commit, train against a dataset, and call immediately.

**Input to invention:**

- `spec` — plain text description of what the function should measure
- `name` — target repository name for publishing
- `depth`, `min_branch_width`, `max_branch_width`, `min_leaf_width`, `max_leaf_width` — tree shape constraints
- Optional: an agent to run the invention steps; a seed for reproducibility; a remote target (GitHub or local filesystem)

**Output:** a `function.json` with a JSON Schema `input_schema`, a `tasks` array of vector completions (or nested function references), and a `description`. The file is published to the configured remote automatically.

### Recursive invention

Setting `depth > 0` triggers recursive invention. The root function is invented first. Its task tree contains placeholder slots for child functions. The recursive client then spawns a concurrent child invention for each placeholder, resolving the full tree bottom-up. All streams are merged immediately — no waiting for siblings. The result is a multi-level decision tree where every node is itself an invented, deployable function.

Depth and width bounds control the shape: `min_branch_width` / `max_branch_width` govern non-leaf nodes; `min_leaf_width` / `max_leaf_width` govern leaves. A `depth=2`, `max_branch_width=3` invention produces up to nine leaf functions under three branch functions under one root — all invented concurrently, all published independently.

```bash
objectiveai functions inventions recursive create alpha-scalar \
  --name my-org/code-quality-scorer \
  --spec "Score a pull request diff on correctness, readability, and test coverage" \
  --depth 1 --min-branch-width 2 --max-branch-width 3
```

### The self-improvement loop

1. **Invent** — agent describes what it needs to judge; invention generates `function.json`.
2. **Train** — provide labeled examples; ObjectiveAI learns weights and writes `profile.json`.
3. **Deploy** — push both files to a Git repository; reference by `owner/repo@commit`.
4. **Use** — the same agent (or any agent) calls the function to score future inputs.

Each cycle produces a reusable, versioned judgment tool. An agent that executes this loop on demand gains scoring infrastructure calibrated to its own criteria — not to a pre-defined rubric. The system does not fine-tune models; it learns weights over fixed agents. The infrastructure improves; the models stay stable.

## SDKs

Every SDK exposes the same three surface areas: **Agent Completions** (single-model chat), **Vector Completions** (multi-agent voting that returns a score vector), and **Function Executions** (composable scoring pipelines). All three support streaming via Server-Sent Events. The API emits incremental chunks; each SDK merges them into an accumulating object using an immutable merge system (TypeScript), a mutable push system (Python, Rust, Go), or equivalent. Types are generated from a shared JSON Schema corpus derived from the Rust SDK, so field names and shapes are identical across languages.

### Languages

| Language | Package | Install | Runtime targets |
|---|---|---|---|
| Rust | `objectiveai-sdk` on crates.io | `cargo add objectiveai-sdk` | Any (async via `reqwest` + `tokio`) |
| TypeScript | `@objectiveai/sdk` on npm | `npm i @objectiveai/sdk` | Node.js, Deno, browser (CJS + ESM) |
| Python | `objectiveai-sdk` on PyPI | `pip install objectiveai-sdk` | CPython 3.10+ (includes PyO3 extension) |
| Go | `github.com/ObjectiveAI/objectiveai/objectiveai-sdk-go` | `go get github.com/ObjectiveAI/objectiveai/objectiveai-sdk-go` | Go 1.26+ |
| .NET | `ObjectiveAI` (NuGet — in progress) | not yet published | net10.0 |

### Streaming examples

The base URL defaults to `https://api.objectiveai.dev` in all SDKs. Auth is passed as `OBJECTIVEAI_AUTHORIZATION` (env var) or via the client constructor.

#### TypeScript

```typescript
import {
  ObjectiveAI,
  vectorCompletionsCreateVectorCompletion,
  vectorCompletionsResponseStreamingVectorCompletionChunkMerged,
} from "@objectiveai/sdk";

const client = new ObjectiveAI({ authorization: process.env.OBJECTIVEAI_AUTHORIZATION });

const stream = await vectorCompletionsCreateVectorCompletion(client, {
  stream: true,
  messages: [{ role: "user", content: "Which option is better?" }],
  swarm: { id: "swarm_abc123" },
  responses: ["Option A", "Option B"],
});

let acc: any = null;
for await (const chunk of stream) {
  acc = acc ? vectorCompletionsResponseStreamingVectorCompletionChunkMerged(acc, chunk)[0] : chunk;
}
console.log("scores:", acc?.scores);
```

#### Python

```python
import asyncio, os
from objectiveai_sdk.client import ObjectiveAI
from objectiveai_sdk.vector.completions.http import create_vector_completion

async def main() -> None:
    client = ObjectiveAI(authorization=os.environ.get("OBJECTIVEAI_AUTHORIZATION"))
    params = {
        "stream": True,
        "messages": [{"role": "user", "content": "Which option is better?"}],
        "swarm": {"id": "swarm_abc123"},
        "responses": ["Option A", "Option B"],
    }
    stream = await create_vector_completion(client, params)
    acc = None
    async for chunk in stream:
        if acc is None:
            acc = chunk
        else:
            acc.push(chunk)
    print("scores:", acc.scores if acc else None)

asyncio.run(main())
```

#### Rust

```rust
use futures::StreamExt;
use objectiveai_sdk::{HttpClient, vector::completions};

#[tokio::main]
async fn main() -> Result<(), objectiveai_sdk::HttpError> {
    let client = HttpClient::builder()
        .authorization(std::env::var("OBJECTIVEAI_AUTHORIZATION").ok())
        .build();

    let mut stream = completions::http::create_vector_completion_streaming(
        &client,
        completions::request::params(/* messages, swarm, responses */),
    ).await?;

    let mut acc: Option<completions::response::streaming::VectorCompletionChunk> = None;
    while let Some(Ok(chunk)) = stream.next().await {
        match &mut acc {
            Some(a) => a.push(&chunk),
            None => acc = Some(chunk),
        }
    }
    println!("scores: {:?}", acc.map(|a| a.scores));
    Ok(())
}
```

### Go and .NET

The Go SDK is fully auto-generated from the JSON Schema corpus. Types are strict-validated on unmarshal. The client exposes generic helpers `PostUnary[T]` / `PostStreaming[T]` / `GetUnary[T]` / `DeleteUnary[T]`; endpoint functions such as `VectorCompletionsCreateVectorCompletionStreaming` wrap these. A wazero-hosted WASM binary (compiled from the Rust core) provides chunk-to-unary conversion and merge verification without CGO.

The .NET SDK (`ObjectiveAI`, targeting net10.0) is in active development. The NuGet publish workflow is not yet wired up, so it must be built from source for now.

## Binaries & self-hosting

```bash
curl -fsSL https://raw.githubusercontent.com/ObjectiveAI/objectiveai/main/install.sh | bash
. "$HOME/.objectiveai/env"
```

All four binaries land in `~/.objectiveai/` and are added to `PATH`. The CLI (`objectiveai`) self-updates on startup; re-run the installer to upgrade `objectiveai-api`, `objectiveai-viewer`, and `objectiveai-mcp`.

### `objectiveai` (CLI)

The primary user-facing binary. Built with `clap` derive macros and emits newline-delimited JSON (NDJSON) on stdout. Top-level command groups: `agents`, `swarms`, `functions`, `vector`, `laboratories`, `plugins`, `logs`, `instructions`, `schemas`, `api`, `viewer`.

```bash
objectiveai agents list
objectiveai agents completions create --model openai/gpt-4o
objectiveai functions executions create standard --function ...
objectiveai plugins install github --owner ObjectiveAI --repository my-plugin
```

The default build embeds the Tauri viewer as a sidecar: running a streaming command opens a live viewer window backed by an in-process HTTP server. Pass `--no-viewer` at install time for a smaller build without the embedded viewer. JSON schemas for every public type are accessible at `objectiveai schemas list` / `objectiveai schemas output <name>`.

### `objectiveai-api`

Standalone HTTP API server. Run it with:

```bash
objectiveai-api
```

Key environment variables (all optional):

| Variable | Default | Effect |
|---|---|---|
| `ADDRESS` | `0.0.0.0` | Bind address |
| `PORT` | `5000` | Bind port |
| `OBJECTIVEAI_ADDRESS` | `https://api.objectiveai.dev` | Upstream ObjectiveAI address when proxying |
| `OBJECTIVEAI_AUTHORIZATION` | — | Bearer token for the ObjectiveAI API |
| `OPENROUTER_AUTHORIZATION` | — | Bearer token for OpenRouter |
| `GITHUB_AUTHORIZATION` | — | GitHub token for resource retrieval |
| `MCP_AUTHORIZATION` | — | Bearer token for outbound MCP calls |

The server is streaming-first: every layer (agent completions, vector completions, function executions, inventions) produces a typed stream of chunks and yields immediately to the HTTP response — nothing is buffered in the hot path.

### `objectiveai-viewer`

Standalone Tauri desktop application. Presents the same UI that the CLI embeds as a sidecar, but runs as a first-class window manager process rather than being spawned in-process by a CLI command. Reach for it when you want the viewer always open and decoupled from CLI invocations.

### `objectiveai-mcp`

MCP (Model Context Protocol) server built from `objectiveai-mcp-cli`. Exposes ObjectiveAI's tooling over the streamable-HTTP MCP transport so editors and agents (Claude, Cursor, etc.) can invoke it via the standard MCP protocol. Defaults to `0.0.0.0:3000`; override with `ADDRESS` and `PORT`.

Three crates make up the MCP surface:

- **`objectiveai-mcp-cli`** — the binary shipped as `objectiveai-mcp`. Wraps the CLI as MCP tools over streamable-HTTP.
- **`objectiveai-mcp-proxy`** — a multiplexing sidecar of `objectiveai-api`. Terminates an MCP client connection and forwards tool calls to an upstream MCP server or to ObjectiveAI-native tools. Embedded inside `objectiveai-api` at runtime.
- **`objectiveai-mcp-filesystem`** — MCP filesystem helpers (read/write/list) adapting the SDK's filesystem layer to MCP tool calls. Docker-injected into laboratory executions so agents running in sandboxed containers can access the ObjectiveAI filesystem layer.

### Install flags

Pass flags to `bash -s --` after the installer URL:

```bash
curl -fsSL https://raw.githubusercontent.com/ObjectiveAI/objectiveai/main/install.sh | bash -s -- --no-viewer
```

| Flag | Effect |
|---|---|
| `--no-viewer` | Skips the standalone `objectiveai-viewer`; installs the CLI variant without an embedded Tauri viewer (smaller binary). |
| `--no-api` | Skips `objectiveai-api`. |
| `--no-mcp` | Skips `objectiveai-mcp`. |
| `--cli-only` | Equivalent to `--no-viewer --no-api --no-mcp`. Only `objectiveai` is installed. |

Flags compose freely.

### Self-host vs hosted

The hosted API at `https://api.objectiveai.dev` requires no setup and is the default for the CLI and all SDKs. Run your own `objectiveai-api` when you need total control over data routing — for example, to point agents at private upstream providers not available on OpenRouter, to meet on-prem or air-gapped requirements, or to run the full execution pipeline locally without network egress. Configure the CLI to point at your instance with `objectiveai api mode set local` and `objectiveai api local address set http://localhost:5000`.

Supported platforms: Linux x86_64, Linux aarch64, macOS x86_64, macOS aarch64 (Apple Silicon), Windows x86_64.

## Plugins

A plugin is a binary that adds new top-level subcommands to the ObjectiveAI CLI, optionally paired with a viewer UI tab. Plugins are described by an `objectiveai.json` manifest at the repository root. The CLI dispatches any unknown top-level subcommand to the matching installed plugin binary, communicating over a JSONL protocol on stdout. The viewer surfaces plugins with a declared UI source as sandboxed iframe tabs, isolated from the host DOM.

### Installing a plugin

Install from a public GitHub repository:

```bash
# From the ObjectiveAI org (default whitelist — no extra flags needed).
objectiveai plugins install github --owner ObjectiveAI --repository my-plugin

# Pin to a specific commit.
objectiveai plugins install github --owner ObjectiveAI --repository my-plugin --commit-sha <sha>

# Third-party repository — requires explicit opt-in.
objectiveai plugins install github --owner third-party --repository my-plugin --allow-untrusted

# Replace an existing install (binary, viewer bundle, and manifest are rewritten).
objectiveai plugins install github --owner ObjectiveAI --repository my-plugin --upgrade
```

To print layout and manifest conventions for placing a plugin by hand in `~/.objectiveai/plugins/`:

```bash
objectiveai plugins install filesystem
```

### Plugin manifest

`objectiveai.json` at the repository root declares the plugin's metadata, platform binaries, and optional viewer source. All fields except `description` and `version` are optional.

| Field | Type | Notes |
|---|---|---|
| `description` | string | Required. One-line summary shown in listings. |
| `version` | string | Required. Used to construct release-asset URLs (`releases/download/v<version>/<asset>`). |
| `author` / `homepage` / `license` | string | Optional metadata. |
| `binaries` | object | Map of `<os>_<arch>` → release-asset filename. Supported keys: `linux_x86_64`, `linux_aarch64`, `windows_x86_64`, `windows_aarch64`, `macos_x86_64`, `macos_aarch64`. Declare only platforms you ship. |
| `viewer_zip` | string | Release-asset filename for the UI bundle (a zip with `index.html` at root). Mutually exclusive with `viewer_url`. |
| `viewer_url` | string | Remote URL loaded as the iframe `src` verbatim. Must be `https://` or `http://localhost`. Mutually exclusive with `viewer_zip`. |
| `viewer_routes` | array | HTTP routes the viewer's embedded axum server exposes on behalf of the plugin. |
| `mobile_ready` | bool | Opt-in for iOS/Android viewer builds. Defaults to false. |

Example:

```json
{
  "description": "Run wave-physics simulations from the CLI.",
  "version": "1.0.0",
  "author": "Example Corp",
  "license": "MIT",
  "binaries": {
    "linux_x86_64":   "psyops-linux-x86_64",
    "windows_x86_64": "psyops-windows-x86_64.exe",
    "macos_aarch64":  "psyops-macos-aarch64"
  },
  "viewer_zip": "psyops-viewer.zip"
}
```

### Building a plugin

A plugin binary reads its arguments from `argv` and writes JSONL to stdout. Each line must be one of three shapes:

```jsonc
{"type": "notification", "key": "value"}        // data to forward to the caller
{"type": "error", "level": "warn", "fatal": false, "message": "..."}
{"type": "command", "command": "agents list"}    // spawn a CLI command, fire-and-forget
```

The host parses stdout line-by-line; unparseable lines are forwarded as string notifications rather than dropped.

For the viewer, produce a static `dist/` with `index.html` at the root, zip it, and reference it in `viewer_zip`. For remote-hosted UIs, use `viewer_url`. The viewer posts events to the iframe via `postMessage`.

To iterate locally: place the binary at `~/.objectiveai/plugins/<name>/plugin[.exe]` and the manifest at `~/.objectiveai/plugins/<name>.json`, then invoke `objectiveai <name> <args>`. The `objectiveai-cli/test-fixtures/hello-plugin/` fixture is the minimal example — a single `main.rs` that reads `argv[1]` and emits one notification line.

For distribution, cut a GitHub release tagged `v<version>`, upload binaries and the viewer zip as release assets named exactly as declared in the manifest, then install with `plugins install github`.

Full reference: [PLUGINS.md](PLUGINS.md).

## Web app & ecosystem

### Web app

[objectiveai.dev](https://objectiveai.dev) is the production web interface, built with Next.js (App Router). The app provides browsing and detail views for the three core resource types: Functions (`/functions`, `/{owner}/{repo}`), Swarms (`/swarms`, `/{id}`), and Profiles (`/profiles`). From a function detail page, users can inspect the task tree, execute the function against a chosen swarm and profile, and view per-task vote breakdowns and aggregate scores. The profiles listing surfaces trained weight configurations available for reuse. A `/demo` route renders live component prototypes including the `FunctionTree` canvas visualization, vote matrices, decomposition views, and contribution waterfalls.

### Examples

The [`examples/`](examples/) directory collects real software built on top of ObjectiveAI, with links to full source repositories.

**[psychological-operations](examples/psychological-operations.md)** — an agentic X (Twitter) scraper and scoring pipeline ([repo](https://github.com/WiggidyW/psychological-operations)). It pairs human-driven Chrome automation with ObjectiveAI to rank scraped tweets along operator-defined axes. The project defines three primary objects: *Scrapes* (declarative search jobs that scroll and parse `x.com` into SQLite), *PsyOps* (scoring jobs that pull tagged posts and run them through an ObjectiveAI function using a chosen swarm, profile, and strategy — including Swiss System tournament-style ranking), and *Inventions* (wrappers around recursive function invention). A pilot study ranked tweets from 33 YC W22 CEO accounts along an *unsettlingness* axis using sub-functions invented by a Claude Opus agent; published artifacts are content-addressed and reproducible.

### Ecosystem

- **`objectiveai-claude-agent-sdk-runner`** — a long-lived Python stdio NDJSON server that runs concurrent Claude Agent SDK sessions on behalf of `objectiveai-api`. The Rust API caller spawns and multiplexes requests over a single stdin/stdout pair using a semaphore-backed FIFO queue; each request carries a string `id` for demultiplexing events from N concurrent streams.
- **`objectiveai-codex-sdk-runner`** — same architecture as the Claude runner but targets the OpenAI Codex SDK. Authentication is inherited from `~/.codex/auth.json`; the runner shells out to the `codex` binary and streams `ThreadEvent` objects back to the Rust caller.
- **`objectiveai-function-tree`** — a TypeScript/React package that renders a 2D canvas visualization of ObjectiveAI function execution trees. Exposes a `FunctionTree` component plus a headless `core` export and CSS; peer-depends on React 18+. Used internally by `objectiveai-web`.
- **`objectiveai-cocoindex`** ([PyPI](https://pypi.org/project/objectiveai-cocoindex/)) — a Python integration that wraps ObjectiveAI function executions as memoized [CocoIndex](https://github.com/cocoindex-io/cocoindex) processing components. The memo key combines the bound `(function, profile, strategy)` triple with the per-call input, making it safe to drop into indexing pipelines.
- **`objectiveai-github-discord-notifier`** — a Python FastAPI webhook server (Docker-deployable) that validates GitHub webhook signatures and forwards pull-request and issue events to a configured Discord channel.
- **`objectiveai-json-schema`** — generated JSON Schema files for every public serializable type in the Rust SDK, named using dot-separated module paths (e.g. `functions.executions.RetryToken.json`). Several hundred schemas cover agents, swarms, functions, profiles, vector completions, CLI output, MCP types, and more. These files drive code generation for the Go SDK and .NET SDK and can be used by any downstream tooling that needs machine-readable type definitions.
- **[ObjectiveAI-claude-code-1](https://github.com/ObjectiveAI-claude-code-1)** — an autonomous Claude Code agent that invents and publishes ObjectiveAI Functions without human intervention. Uses the Agent SDK to create, test, and deploy new scoring pipelines, closing the loop on the invention system.

## Repository structure

A single git repository contains the SDK core, server, clients, integrations, and tools.

```text
objectiveai/
│
├── # SDK core (Rust)
│   ├── objectiveai-sdk-rs/                    # Rust SDK — types, validation, compilation
│   ├── objectiveai-sdk-rs-macros/             # Procedural macros for the Rust SDK
│   ├── objectiveai-sdk-rs-cffi/               # C FFI bindings (expose SDK to C/C++)
│   ├── objectiveai-sdk-rs-pyo3/               # PyO3 bindings (Rust extension for Python)
│   └── objectiveai-sdk-rs-wasm-js/            # WASM bindings for browser / Node.js
│
├── # SDKs (other languages)
│   ├── objectiveai-sdk-js/                    # TypeScript/JavaScript SDK (npm)
│   ├── objectiveai-sdk-py/                    # Python SDK (PyPI)
│   ├── objectiveai-sdk-go/                    # Go SDK
│   └── objectiveai-dotnet/                    # .NET SDK (NuGet: ObjectiveAI)
│
├── # Server & binaries
│   ├── objectiveai-api/                       # API server (self-hostable or importable)
│   ├── objectiveai-cli/                       # Command-line interface
│   ├── objectiveai-viewer/                    # Desktop viewer app (Tauri)
│   └── objectiveai-mcp-cli/                   # MCP CLI binary (ships as objectiveai-mcp)
│
├── # MCP integration
│   ├── objectiveai-mcp-proxy/                 # MCP proxy — multiplexes tool calls
│   └── objectiveai-mcp-filesystem/            # MCP filesystem helpers
│
├── # Runners
│   ├── objectiveai-claude-agent-sdk-runner/   # Concurrent Claude Agent SDK runner
│   └── objectiveai-codex-sdk-runner/          # Concurrent OpenAI Codex SDK runner
│
├── # Web & tools
│   ├── objectiveai-web/                       # Next.js production web interface
│   ├── objectiveai-function-tree/             # 2D canvas function-tree visualizer
│   ├── objectiveai-cocoindex/                 # CocoIndex integration (Python)
│   ├── objectiveai-github-discord-notifier/   # GitHub webhook → Discord notifier
│   └── objectiveai-json-schema/               # Generated JSON Schema files
│
└── # Other
    ├── examples/                              # Usage examples
    ├── bin/                                   # Vendored build tool binaries
    └── *.sh                                   # Root scripts: build, install, publish, version
```

## Contributing & development

### Prerequisites

- **Rust** — stable toolchain via [rustup](https://rustup.rs/). No pinned `rust-toolchain.toml`; use the current stable release. `wasm-pack` and `maturin` are installed automatically by `build-bin.sh` into `./bin/`.
- **Node.js + pnpm 10.25.0** — the workspace `packageManager` field pins this version. Install pnpm via `corepack enable` or `npm i -g pnpm@10.25.0`.
- **Python** — required for `objectiveai-sdk-py` (PyO3/maturin extension build) and the Claude/Codex agent-SDK runners (PyInstaller).
- **Docker** — required for the `objectiveai-mcp-filesystem` musl cross-compilation step in `build.sh`.

### Build

```bash
pnpm install                 # JS workspace dependencies
cargo build --release        # Rust crates
bash build.sh                # full monorepo build in dependency order
bash build-bin.sh            # (re)install pinned build tools into ./bin/
```

`build.sh` generates JSON schemas, compiles WASM and CFFI bindings, builds all language SDKs (.NET, Go, Python, JS), and produces viewer artifacts.

### Test

```bash
bash test.sh                 # all suites in parallel (spawns a local API server)
cargo test                   # Rust workspace tests
pnpm test                    # JS/TS tests
```

`test.sh` exports `OBJECTIVEAI_TEST_PORT` and runs per-package `test.sh` scripts concurrently across `objectiveai-sdk-rs`, `objectiveai-api`, `objectiveai-json-schema`, `objectiveai-cli`, `objectiveai-mcp-proxy`, `objectiveai-sdk-js`, `objectiveai-sdk-py`, `objectiveai-sdk-go`, and `objectiveai-viewer`. Tests must not hit the production API — use the local server, mocks, or fixtures.

### Conventions

- **Package manager:** use `pnpm`, never `npm`. Filter to a single workspace package with `pnpm --filter <package-name> run <script>`.
- **No type re-exports in Rust.** When an import path is wrong, fix it at the call site. Never add re-export aliases or shim `pub use` entries to paper over a broken import.
- **`mod.rs` discipline.** `mod.rs` files contain only module declarations and re-export globs — no functions, structs, enums, traits, or impls. Every entry must be either `pub mod foo;` or `mod foo; pub use foo::*;`.
- **No network-hitting tests.** Tests must not contact the production API. Mock responses or use local fixtures.
- **Test failures are not pre-existing issues.** Every failure must be investigated and fixed; never dismiss one to move on.
- **Single shared version.** All packages share one version number. Bump atomically across Cargo.toml, package.json, pyproject.toml, .csproj, and all inter-package dependency references with `bash version.sh <new-version>`.
- **Publishing.** `bash publish.sh` orchestrates the full release across crates.io, PyPI, npm, the Go module proxy, and GitHub Releases in dependency-order waves, polling each registry until the new version is live before proceeding.

## License

[MIT](LICENSE).
