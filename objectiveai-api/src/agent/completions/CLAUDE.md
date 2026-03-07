# Agent Completions

## UpstreamClient Rules

1. **No error chunk as the first item.** If the upstream would fail before producing any non-error chunk, it must return `Err(...)` from `create` instead of yielding an error chunk into the stream.

2. **No empty streams.** If the upstream produces no chunks at all, it must return `Err(...)` from `create` instead of an empty stream.

These rules apply to all `UpstreamClient` implementations (openrouter, claude_agent_sdk, mock).

## Continuation

A `Continuation` carries the conversation state between successive
`create_streaming` calls. It contains:

- **`ContinuationItem::State`** — upstream-specific state (e.g. tool call
  count for mock, session data for Claude Agent SDK).
- **`ContinuationItem::UserMessage`** — a user message appended after the
  previous assistant turn.
- **`ContinuationItem::ToolMessage`** — a tool result appended after a tool
  call.

The upstream client receives both `params.messages` (the initial fixed
messages) and the continuation items. Together they form the full
conversation history: `params.messages` is the conversation prefix that
never changes, and the continuation items are everything that happened
after.

### How `create_streaming` uses messages + continuation

1. `params.messages` are merged with any agent-specific messages (e.g.
   prefix/suffix), prepared, and optionally transformed.
2. Continuation items are passed as `&[ContinuationItem<STATE>]` to the
   upstream client's `create()`, which appends them to the prepared
   messages to reconstruct the full conversation.

### Important: `params.messages` is fixed

Once set for the first call, `params.messages` must not change across
subsequent calls within the same conversation. New user turns (step
prompts, retry prompts) go onto the continuation as `UserMessage` items.

## Function Invention Client

The invention client orchestrates multi-step function invention. Each step
runs one or more agent completions with invention tools.

### Message flow

- **First step (no continuation):** The step prompt goes into
  `params.messages` as a user message. This establishes the fixed message
  prefix for the entire invention conversation.
- **Subsequent steps (continuation exists):** The step prompt is pushed
  as a `UserMessage` onto the continuation. `params.messages` remains
  empty (the fixed prefix from step 1 is already baked into the
  continuation's history).

### Retry flow

When a step's validation fails after the agent loop ends, the client
retries up to `max_step_retries` times (default 3). Each retry:

1. Constructs a retry prompt: `"{prompt}\n\nThe following error occurred: {error}\n\nPlease try again."`
2. Pushes it as a `UserMessage` onto the continuation.
3. Calls `create_streaming` again with the same `params` (empty messages)
   and the updated continuation.

This matches the pattern from `objectiveai-cli` (`runAgentStep` in
`src/agent/index.ts`).
