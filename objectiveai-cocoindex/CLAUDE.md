# objectiveai-cocoindex

ObjectiveAI integration for [cocoindex](https://github.com/cocoindex-io/cocoindex).

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
