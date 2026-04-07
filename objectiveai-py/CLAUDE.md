# objectiveai-py

Python SDK for ObjectiveAI. Generated from `objectiveai-json-schema/`.

## Virtual Environment

**CRITICAL: Never run bare `python` or `pip` commands.** Always use the venv:

```bash
# Windows
objectiveai-py/venv/Scripts/python.exe <args>
objectiveai-py/venv/Scripts/pip.exe <args>

# Running scripts
objectiveai-py/venv/Scripts/python.exe objectiveai-py/scripts/install_pydantic.py

# Running tests
objectiveai-py/venv/Scripts/python.exe -m pytest objectiveai-py/tests/ <args>
```

## Code Generation

Types are auto-generated from JSON schemas by `scripts/install_pydantic.py`. Do not edit files under `objectiveai/` that contain the `THIS FILE IS AUTO-GENERATED` header.

```bash
objectiveai-py/venv/Scripts/python.exe objectiveai-py/scripts/install_pydantic.py
```

## Tests

```bash
# All tests
objectiveai-py/venv/Scripts/python.exe -m pytest objectiveai-py/tests/ -x --tb=short

# Roundtrip test only
objectiveai-py/venv/Scripts/python.exe -m pytest objectiveai-py/tests/test_pydantic_roundtrip.py -x --tb=short
```
