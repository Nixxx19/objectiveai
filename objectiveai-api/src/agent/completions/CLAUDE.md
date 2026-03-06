# Agent Completions

## UpstreamClient Rules

1. **No error chunk as the first item.** If the upstream would fail before producing any non-error chunk, it must return `Err(...)` from `create` instead of yielding an error chunk into the stream.

2. **No empty streams.** If the upstream produces no chunks at all, it must return `Err(...)` from `create` instead of an empty stream.

These rules apply to all `UpstreamClient` implementations (openrouter, claude_agent_sdk, mock).
