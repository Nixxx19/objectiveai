# objectiveai-cocoindex

ObjectiveAI integration for [cocoindex](https://github.com/cocoindex-io/cocoindex).

## Dependencies

- `cocoindex` — installed via pip into the venv.
- `../objectiveai-py` — sibling package, not pip-installable (no `pyproject.toml`).
  Imported via `PYTHONPATH` set up by `test.sh`. Its pinned runtime deps
  (`pydantic`, `httpx`, `httpx-sse`) are pulled in via
  `-r ../objectiveai-py/requirements.txt` in our `requirements.txt`.

## Virtual Environment

**CRITICAL: Never run bare `python` or `pip` commands.** Always use the venv:

```bash
# Windows
objectiveai-cocoindex/venv/Scripts/python.exe <args>
objectiveai-cocoindex/venv/Scripts/pip.exe <args>

# Linux/macOS
objectiveai-cocoindex/venv/bin/python <args>
objectiveai-cocoindex/venv/bin/pip <args>
```

## Build

```bash
bash objectiveai-cocoindex/build.sh
```

Creates the venv (if missing) and installs `requirements.txt` + `requirements-dev.txt`.

## Tests

```bash
bash objectiveai-cocoindex/test.sh
bash objectiveai-cocoindex/test.sh -- -k foo -vv
```
