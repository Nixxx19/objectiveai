# objectiveai-cocoindex

ObjectiveAI integration for [cocoindex](https://github.com/cocoindex-io/cocoindex).

## Dependencies

- `cocoindex` — installed from PyPI into the venv.
- `objectiveai==X.Y.Z` — pinned in `requirements.txt` so this package
  declares a clean PyPI dependency when published. Version is bumped by
  `bash version.sh <new-version>` (entry in `REQUIREMENTS_TXTS`).
- For local dev, `build.sh` filters the `objectiveai==` line out of the
  install and editable-installs `../objectiveai-py` instead. This means
  changes in the sibling SDK are picked up immediately, and the cocoindex
  build doesn't depend on a published PyPI wheel existing yet. Maturin
  compiles `objectiveai._pyo3` into the cocoindex venv on first install,
  so a Rust toolchain must be available locally.

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
