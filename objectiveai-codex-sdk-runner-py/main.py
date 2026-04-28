#!/usr/bin/env python3
"""ObjectiveAI Codex SDK Runner.

Runs the official OpenAI Codex Python SDK (``openai-codex-sdk``) and streams
thread events to stdout as NDJSON. Designed to be spawned as a subprocess by
``objectiveai-api``.

Authentication is inherited from the user's ``~/.codex/auth.json`` (written
by ``codex login``). The SDK shells out to the ``codex`` binary which reads
that file — we do nothing special for auth. The ``codex`` binary is found
either by the SDK's vendored bundle or via system PATH.
"""

from __future__ import annotations

__version__ = "2.0.0"

import argparse
import asyncio
import json
import sys
from typing import Any

from openai_codex_sdk import (
    Codex,
    LocalImageInput,
    TextInput,
)


# ---------------------------------------------------------------------------
# Input parsing
# ---------------------------------------------------------------------------
# ``--input`` is a JSON object representing a single user message. Shape:
#
#   {
#     "content": "string"                     # plain text content, OR
#                | [                          # an ordered list of parts:
#                  {"type": "text", "text": "..."},
#                  {"type": "local_image", "path": "..."}
#                ],
#     "name": "optional-author-name"          # rendered as a "[name] :"
#                                             # prefix text part if present
#   }


def _parse_input(raw: Any) -> list[Any]:
    """Parse a single user message JSON object into Codex SDK input items."""
    if not isinstance(raw, dict):
        raise ValueError("--input must be a JSON object representing a user message")

    name = raw.get("name")
    if name is not None and not isinstance(name, str):
        raise ValueError("--input.name must be a string")

    content = raw.get("content")
    if content is None:
        raise ValueError("--input is missing required field: content")

    items: list[Any] = []

    # Optional name → leading "[name] :" text part, mirroring Claude's prompt.rs.
    if isinstance(name, str) and name:
        items.append(TextInput(type="text", text=f"[{name}] :"))

    if isinstance(content, str):
        items.append(TextInput(type="text", text=content))
    elif isinstance(content, list):
        for idx, part in enumerate(content):
            if not isinstance(part, dict) or "type" not in part:
                raise ValueError(
                    f"--input.content[{idx}] must be an object with a 'type' field"
                )
            t = part["type"]
            try:
                if t == "text":
                    items.append(TextInput(type="text", text=part["text"]))
                elif t == "local_image":
                    items.append(
                        LocalImageInput(type="local_image", path=part["path"])
                    )
                else:
                    raise ValueError(
                        f"--input.content[{idx}] has unknown type: {t!r}"
                    )
            except KeyError as e:
                raise ValueError(
                    f"--input.content[{idx}] missing required field: {e.args[0]}"
                )
    else:
        raise ValueError("--input.content must be a string or an array of parts")

    return items


# ---------------------------------------------------------------------------
# Event serialization
# ---------------------------------------------------------------------------
# Each ThreadEvent from the SDK is a pydantic model. We emit it as NDJSON
# using the model's own dump (camelCase via alias-aware mode is not needed
# here — the SDK's wire format is already snake_case on the event layer).


def _serialize_event(event: Any) -> dict[str, Any]:
    if hasattr(event, "model_dump"):
        return event.model_dump(mode="json", by_alias=False, exclude_none=False)
    if isinstance(event, dict):
        return event
    return {"_repr": repr(event)}


def _emit(obj: dict[str, Any]) -> None:
    sys.stdout.write(json.dumps(obj) + "\n")
    sys.stdout.flush()


# ---------------------------------------------------------------------------
# CLI argument parsing
# ---------------------------------------------------------------------------


def _truthy_flag(parser: argparse.ArgumentParser, name: str, help_text: str) -> None:
    """Add a tri-state flag: --name / --no-name / absent (None)."""
    group = parser.add_mutually_exclusive_group()
    group.add_argument(
        f"--{name}",
        dest=name.replace("-", "_"),
        action="store_const",
        const=True,
        default=None,
        help=help_text,
    )
    group.add_argument(
        f"--no-{name}",
        dest=name.replace("-", "_"),
        action="store_const",
        const=False,
        help=f"Disable: {help_text}",
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run the OpenAI Codex Python SDK and stream events as NDJSON to stdout.",
    )
    parser.add_argument(
        "--model",
        default=None,
        help="Codex model identifier (e.g. gpt-5). Optional if --resume is used.",
    )
    parser.add_argument(
        "--input",
        required=True,
        help='Turn input: a JSON object representing a single user message. '
        'Shape: {"content": "..." | [{"type":"text","text":"..."},'
        '{"type":"local_image","path":"..."}], "name": "optional-name"}',
    )
    parser.add_argument(
        "--effort",
        choices=["minimal", "low", "medium", "high"],
        default=None,
        help="Model reasoning effort.",
    )
    parser.add_argument(
        "--resume",
        default=None,
        help="Thread id to resume instead of starting a new thread.",
    )
    _truthy_flag(parser, "web-search-enabled", "Allow the agent to use web search.")
    return parser.parse_args()


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def _only_set(d: dict[str, Any]) -> dict[str, Any]:
    return {k: v for k, v in d.items() if v is not None}


async def run(args: argparse.Namespace) -> int:
    input_ = _parse_input(json.loads(args.input))

    if not args.resume and not args.model:
        raise ValueError("--model is required unless --resume is given")

    codex = Codex()

    thread_options = _only_set({
        "model": args.model,
        "model_reasoning_effort": args.effort,
        "web_search_enabled": args.web_search_enabled,
    })

    if args.resume:
        thread = codex.resume_thread(args.resume, thread_options)
    else:
        thread = codex.start_thread(thread_options)

    streamed = await thread.run_streamed(input_)

    exit_code = 0
    async for event in streamed.events:
        _emit(_serialize_event(event))
        event_type = getattr(event, "type", None)
        if event_type == "turn.failed" or event_type == "error":
            exit_code = 1

    return exit_code


def _silence_proactor_pipe_warnings() -> None:
    """Workaround for a long-standing CPython bug on Windows.

    On Windows, asyncio's `_ProactorBasePipeTransport` and
    `BaseSubprocessTransport` `__del__` methods run during interpreter
    shutdown and may try to `repr` themselves for a debug log;
    `__repr__` calls `fileno()` on the underlying pipe; if the pipe is
    already closed (which is *normal* at shutdown — the OS or the SDK
    subprocess we drove has gone away) `fileno()` raises
    `ValueError: I/O operation on closed pipe`. Python prints the trace
    as `Exception ignored in:` on stderr.

    Two distinct entry points hit this:
      - `_ProactorBasePipeTransport.__del__` directly.
      - `BaseSubprocessTransport.__del__` → `__repr__` → child pipe's
        `_ProactorBasePipeTransport.__repr__` → `fileno()`.

    Both must be wrapped — patching only the pipe-transport's `__del__`
    leaves the subprocess-transport path open. The exception is
    harmless (the transport already released everything it owned) but
    anything downstream that treats stderr as a failure signal (e.g.
    objectiveai-api wrapping our exit) sees it and reports a 500.

    Refs: bpo-39232, gh-91555.
    """
    if sys.platform != "win32":
        return

    def _wrap_del(cls: Any) -> None:
        original = cls.__del__

        def _patched(self, *a: Any, **kw: Any) -> None:
            try:
                original(self, *a, **kw)
            except (ValueError, OSError):
                # Closed-pipe race during shutdown — drop it. Anything
                # else propagates so real bugs still surface.
                pass

        cls.__del__ = _patched  # type: ignore[method-assign]

    try:
        from asyncio.proactor_events import _ProactorBasePipeTransport  # type: ignore[attr-defined]
        _wrap_del(_ProactorBasePipeTransport)
    except Exception:
        pass
    try:
        from asyncio.base_subprocess import BaseSubprocessTransport
        _wrap_del(BaseSubprocessTransport)
    except Exception:
        pass


def main() -> None:
    # Suppress the cosmetic Windows asyncio shutdown warning before we
    # start the loop, so even crashes-during-cleanup don't leak it.
    _silence_proactor_pipe_warnings()

    args = parse_args()
    try:
        code = asyncio.run(run(args))
    except Exception as e:  # noqa: BLE001 - surface everything to stderr
        sys.stderr.write(f"{type(e).__name__}: {e}\n")
        sys.exit(1)
    sys.exit(code)


if __name__ == "__main__":
    main()
