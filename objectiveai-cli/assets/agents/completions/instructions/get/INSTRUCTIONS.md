# Agent Completion — Spawn Instructions

You are about to run `objectiveai agents spawn …`.
Read all of the following before constructing the command.

## What `spawn` does

`agents spawn` fires a child agent in the background and exits
immediately. It emits one notification on stdout:

```
{"type":"notification","agent_id":"<local-id>"}
```

Where `<local-id>` is the spawned agent's local lineage segment
(the trailing segment past your own caller id). The actual
completion streams into log files on disk; this command does **not**
wait for the agent to finish.

**Never use the built-in `Monitor` tool on an `agents spawn`
invocation.** It exits within a second; there is nothing to
monitor at the CLI level. Use the polling commands below instead.

## Providing messages

Exactly one of these flags is required:

- `--simple "<text>"` — quickest. The text becomes a single user
  message (`{ role: "user", content: "<text>" }`).
- `--messages-inline '<json-array>'` — full inline messages array.
- `--messages-file <path>` — same as `--messages-inline` but read
  from a JSON file on disk.
- `--messages-python-inline '<python>'` — Python expression that
  produces a messages list.
- `--messages-python-file <path>` — same, from a `.py` file.

## Watching progress

The spawned agent writes log files under
`${config_base_dir}/logs/agents/completions/response/...`. Poll
them at any time:

```
objectiveai agents completions logs get <local-id>
```

The root log only contains **references** to other logs. Walk
each `messages` reference down to its per-message file to see
what the assistant actually wrote:

```
objectiveai agents completions messages logs get <local-id> <message-index>
```

For multi-turn loops, the continuation chain lives at:

```
objectiveai agents completions continuations logs get <local-id>
```

To enumerate every agent you've spawned plus the timestamp of
each one's latest response:

```
objectiveai agents list-active
```

To drain queue items (request envelopes, assistant responses,
tool responses, notifications) for one or more spawned agents:

```
objectiveai agents read-pending <local-id> [<local-id> ...]
```

Both `list-active` output and `read-pending` input speak
lineage-relative ids — the caller prefix is implicit.

## `get` vs `subscribe`

Default to `get`. It reads what is currently on disk and exits —
fast, predictable, idempotent. You can call it as often as you
want with no side effects.

Switch to `subscribe` **only after** you have already issued `get`,
confirmed that the field you need is not yet present, and you
specifically need to block until the next on-disk write. If `get`
already gives you the data, use `get`.

## Use the CLI directly

The CLI has built-in jq filtering on every `logs get` (pass the
expression as the second positional argument) and emits structured
JSON from every command. **Do not pipe through `python`, `bash`,
`jq`, `awk`, `grep`, etc.** unless shell composition is genuinely
required — the CLI does the same shaping in-process, faster, and
without the quoting hazards of escaping a pipeline.
