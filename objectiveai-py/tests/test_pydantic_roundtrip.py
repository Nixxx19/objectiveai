"""
Roundtrip test: Pydantic model → JSON Schema must exactly match the original
objectiveai-json-schema/ files, proving no information is lost during the
Pydantic code generation.

RULES FOR THIS FILE
===================

1. This test code is FORBIDDEN from reading or deserializing the original
   JSON schema files. Doing so would amount to cheating — the whole point
   is that schemas must be reconstructible entirely from the generated
   Pydantic types.

2. The only things imported from the harness are:
   - ALL_TITLES: the set of schema title strings (metadata, not content)
   - assert_schema_matches(title, dict): the strict equality check

3. To make tests pass, the assistant is allowed to modify:
   - This test file (conversion / normalization logic)
   - The auto-generation script (scripts/install_pydantic.py)

4. The assistant is FORBIDDEN from modifying:
   - The harness (test_pydantic_roundtrip_harness.py)
   - The original JSON schemas (objectiveai-json-schema/*.json)

5. This test MUST be entirely generic. It must not contain any
   schema-specific logic, hardcoded titles, special cases, or
   conditional branches for particular types. It must work unchanged
   even if all existing JSON schemas are removed and replaced with
   entirely new ones. The only schema-aware code lives in the
   auto-generation script.

This is an information-loss and reconstructibility test.
"""

import ast
import importlib
import inspect
import sys
import typing
from pathlib import Path
from typing import Any, Union, get_args, get_origin

import pytest
from pydantic import BaseModel, RootModel
from pydantic.fields import FieldInfo
from pydantic_core import PydanticUndefined

from objectiveai.json_value import JsonValue
from test_pydantic_roundtrip_harness import ALL_TITLES, assert_schema_matches

# Import helpers from the generator so the test stays in sync automatically.
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "scripts"))
import install_pydantic  # noqa: E402
from install_pydantic import (  # noqa: E402
    compute_global_class_names,
    detect_generic_prefixes,
    title_to_class_name,
    title_to_pascal,
)
from install_pydantic import title_to_path as _title_to_path  # noqa: E402


# ---------------------------------------------------------------------------
# Setup
# ---------------------------------------------------------------------------

# Populate GENERIC_PREFIXES so title_to_path works correctly.
install_pydantic.GENERIC_PREFIXES = detect_generic_prefixes(ALL_TITLES)

# Compute global class names (handles within-file collisions).
GLOBAL_CLASS_NAMES = compute_global_class_names(ALL_TITLES)


def title_to_module_and_name(title: str) -> tuple[str, str]:
    """Map a schema title to (module_path, class_name)."""
    dir_path, file_name = _title_to_path(title)
    module_path = "objectiveai." + (dir_path + "." if dir_path else "") + file_name
    module_path = module_path.replace("/", ".")
    class_name = GLOBAL_CLASS_NAMES.get(title, title_to_class_name(title))
    return module_path, class_name


# ---------------------------------------------------------------------------
# Load all Pydantic types
# ---------------------------------------------------------------------------


def _extract_type_checking_imports(mod: Any) -> dict[str, type]:
    """Extract TYPE_CHECKING imports from a module, resolving them at runtime.

    This is needed because TYPE_CHECKING imports aren't available at runtime,
    but model_rebuild needs the actual types. We parse the source AST to find
    what each module imports under TYPE_CHECKING and resolve those to real classes.
    """
    try:
        source = inspect.getsource(mod)
    except (OSError, TypeError):
        return {}

    tree = ast.parse(source)
    imports: dict[str, type] = {}

    for node in ast.walk(tree):
        if isinstance(node, ast.If):
            if isinstance(node.test, ast.Name) and node.test.id == "TYPE_CHECKING":
                for stmt in node.body:
                    if isinstance(stmt, ast.ImportFrom) and stmt.module:
                        try:
                            imported_mod = importlib.import_module(stmt.module)
                            for alias in stmt.names:
                                local_name = alias.asname or alias.name
                                attr = getattr(imported_mod, alias.name, None)
                                if attr is not None:
                                    imports[local_name] = attr
                        except ImportError:
                            pass
    return imports


def load_pydantic_types() -> dict[str, Any]:
    """Import all generated Pydantic types."""
    types: dict[str, Any] = {}
    for title in ALL_TITLES:
        module_path, class_name = title_to_module_and_name(title)
        try:
            mod = importlib.import_module(module_path)
            cls = getattr(mod, class_name)
            types[title] = cls
        except (ImportError, AttributeError) as e:
            types[title] = e

    # Build global namespace for resolving forward references (fallback)
    namespace = {}
    for title, cls in types.items():
        if not isinstance(cls, Exception):
            # Use the actual class name (which may be long for within-file collisions)
            actual_name = GLOBAL_CLASS_NAMES.get(title, title_to_class_name(title))
            namespace[actual_name] = cls

    # Collect all classes (including variant types) from loaded modules
    all_classes: list[type] = []
    for title, cls in types.items():
        if isinstance(cls, type) and issubclass(cls, (BaseModel, RootModel)):
            all_classes.append(cls)
            mod = sys.modules.get(cls.__module__)
            if mod:
                for attr_name in dir(mod):
                    if "Variant" in attr_name:
                        attr = getattr(mod, attr_name, None)
                        if isinstance(attr, type) and issubclass(attr, (BaseModel, RootModel)):
                            all_classes.append(attr)

    # Build per-module namespaces: global namespace + module's own TYPE_CHECKING
    # imports (which correctly resolve colliding short names for that module)
    module_namespaces: dict[str, dict] = {}
    for cls in all_classes:
        mod = sys.modules.get(cls.__module__)
        if mod and mod.__name__ not in module_namespaces:
            module_ns = dict(namespace)
            # Add module's own defined classes (runtime imports + local defs)
            for attr_name in dir(mod):
                attr = getattr(mod, attr_name, None)
                if isinstance(attr, type):
                    module_ns[attr_name] = attr
            # Override with correctly-resolved TYPE_CHECKING imports
            tc_imports = _extract_type_checking_imports(mod)
            module_ns.update(tc_imports)
            module_namespaces[mod.__name__] = module_ns

    # Rebuild all models with per-module namespaces
    for cls in all_classes:
        mod = sys.modules.get(cls.__module__)
        if mod and mod.__name__ in module_namespaces:
            try:
                cls.model_rebuild(_types_namespace=module_namespaces[mod.__name__])
            except Exception:
                pass

    return types


pydantic_types = load_pydantic_types()

# Build reverse mapping: PascalCase name → schema title
# With short class names, multiple titles may share the same class name
# (e.g., AgentCompletionChunk in agent, vector, functions.inventions).
# We keep the old long-name mapping as fallback and also use model_config.title.
_pascal_to_title: dict[str, str] = {}
for _t in ALL_TITLES:
    _pascal_to_title[title_to_pascal(_t)] = _t
    _pascal_to_title[GLOBAL_CLASS_NAMES.get(_t, title_to_class_name(_t))] = _t


# ---------------------------------------------------------------------------
# Custom Pydantic → JSON Schema converter
# ---------------------------------------------------------------------------


def _get_extra_setting(cls: type) -> str | None:
    """Get the 'extra' setting from model_config."""
    config = getattr(cls, "model_config", None)
    if config and isinstance(config, dict):
        return config.get("extra")
    return None


def _is_known_type(tp: Any) -> str | None:
    """If tp is a known Pydantic type (in ALL_TITLES), return its title."""
    if isinstance(tp, type):
        # Prefer model_config.title (unambiguous, set by codegen)
        config = getattr(tp, "model_config", None)
        if config and isinstance(config, dict):
            title = config.get("title")
            if title and title in ALL_TITLES:
                return title
        # Fallback to class name lookup
        name = tp.__name__
        return _pascal_to_title.get(name)
    return None


def _is_none_type(tp: Any) -> bool:
    return tp is type(None)


def _is_nullable_type(tp: Any) -> bool:
    """Check if tp is Optional[X] (Union[X, None])."""
    origin = get_origin(tp)
    if origin is Union:
        return any(_is_none_type(a) for a in get_args(tp))
    return False


def _unwrap_annotated(tp: Any) -> tuple[Any, list[Any]]:
    """Unwrap Annotated[X, ...] → (X, [metadata...])."""
    origin = get_origin(tp)
    if origin is typing.Annotated:
        args = get_args(tp)
        return args[0], list(args[1:])
    return tp, []


def _extract_annotated_constraints(metadata: list[Any]) -> dict:
    """Extract JSON Schema constraints from Annotated metadata (FieldInfo, Ge, Le, etc.)."""
    result: dict = {}
    for m in metadata:
        if isinstance(m, FieldInfo):
            # Check FieldInfo's own metadata list
            for mm in (m.metadata or []):
                if hasattr(mm, "ge") and mm.ge is not None:
                    result["minimum"] = mm.ge
                if hasattr(mm, "le") and mm.le is not None:
                    result["maximum"] = mm.le
                if hasattr(mm, "pattern") and mm.pattern is not None:
                    result["pattern"] = mm.pattern
            # Check json_schema_extra
            extra = m.json_schema_extra
            if isinstance(extra, dict):
                if "format" in extra:
                    result["format"] = extra["format"]
                if "pattern" in extra:
                    result["pattern"] = extra["pattern"]
        else:
            # Direct constraint objects (Ge, Le, etc.)
            if hasattr(m, "ge") and m.ge is not None:
                result["minimum"] = m.ge
            if hasattr(m, "le") and m.le is not None:
                result["maximum"] = m.le
            if hasattr(m, "pattern") and m.pattern is not None:
                result["pattern"] = m.pattern
    return result


def convert_type(tp: Any, root_title: str) -> dict:
    """Convert a Python type annotation to JSON Schema.

    Handles Annotated wrappers, extracting constraints from Field metadata.
    """
    # Unwrap Annotated[T, Field(...)]
    base_tp, metadata = _unwrap_annotated(tp)
    constraints = _extract_annotated_constraints(metadata)

    result = _convert_type_inner(base_tp, root_title)
    result.update(constraints)
    return result


def _convert_type_inner(tp: Any, root_title: str) -> dict:
    """Inner conversion without Annotated unwrapping."""
    if _is_none_type(tp):
        return {"type": "null"}

    # Check if it's a known type → emit $ref
    known = _is_known_type(tp)
    if known:
        return {"$ref": known}

    # RootModel subclass → check for metadata, then unwrap
    if isinstance(tp, type) and issubclass(tp, RootModel) and tp is not RootModel:
        return _convert_root_model(tp, root_title)

    # BaseModel subclass → object with properties
    if isinstance(tp, type) and issubclass(tp, BaseModel) and tp is not BaseModel:
        return _convert_base_model(tp, root_title)

    # Primitive types
    if tp is str:
        return {"type": "string"}
    if tp is int:
        return {"type": "integer"}
    if tp is float:
        return {"type": "number"}
    if tp is bool:
        return {"type": "boolean"}

    # Native types with JSON Schema format
    from datetime import datetime as _datetime
    from uuid import UUID as _UUID
    if tp is _datetime:
        return {"type": "string", "format": "date-time"}
    if tp is _UUID:
        return {"type": "string", "format": "uuid"}

    # object / JsonValue (bare schema — any JSON value)
    if tp is object or tp is JsonValue:
        return {}

    origin = get_origin(tp)
    args = get_args(tp)

    # Union
    if origin is Union:
        return _convert_union(list(args), root_title)

    # list
    if origin is list:
        result: dict = {"type": "array"}
        if args:
            result["items"] = convert_type(args[0], root_title)
        return result

    # dict
    if origin is dict:
        if args and len(args) == 2:
            val_type = args[1]
            if val_type is object or val_type is JsonValue:
                return {"type": "object"}
            val_schema = convert_type(val_type, root_title)
            return {"type": "object", "additionalProperties": val_schema}
        return {"type": "object"}

    # Literal
    if origin is typing.Literal:
        values = list(args)
        result: dict = {}
        # Infer type from literal values
        if values and all(isinstance(v, str) for v in values):
            result["type"] = "string"
        elif values and all(isinstance(v, int) for v in values):
            result["type"] = "integer"
        result["enum"] = values
        return result

    return {}


def _convert_union(args: list[Any], root_title: str) -> dict:
    """Convert a Union type to anyOf schema."""
    none_args = [a for a in args if _is_none_type(a)]
    non_none_args = [a for a in args if not _is_none_type(a)]

    if none_args and len(non_none_args) == 1:
        inner = _convert_union_member(non_none_args[0], root_title)
        return {"anyOf": [inner, {"type": "null"}]}

    variants = [_convert_union_member(a, root_title) for a in args]
    return {"anyOf": variants}


def _convert_union_member(tp: Any, root_title: str) -> dict:
    """Convert a single Union member to a JSON Schema dict.

    For inline variant types (not in ALL_TITLES), includes description
    from docstring and converts the type inline.
    """
    if _is_none_type(tp):
        return {"type": "null"}

    known = _is_known_type(tp)
    if known:
        return {"$ref": known}

    # Inline variant type — include description from docstring
    if isinstance(tp, type) and issubclass(tp, (BaseModel, RootModel)):
        result: dict = {}
        doc = getattr(tp, "__doc__", None)
        if doc:
            result["description"] = doc
        if issubclass(tp, RootModel):
            inner = _convert_root_model(tp, root_title)
        else:
            inner = _convert_base_model(tp, root_title)
        result.update(inner)
        return result

    return convert_type(tp, root_title)


def _convert_root_model(cls: type, root_title: str) -> dict:
    """Convert a RootModel subclass to JSON Schema."""
    # Plain RootModel — unwrap root type and extract field constraints
    fields = cls.model_fields
    if "root" not in fields:
        return {}
    field_info = fields["root"]
    root_type = field_info.annotation
    result = convert_type(root_type, root_title)

    # Extract constraints from field_info.metadata (Pydantic unwraps Annotated)
    for m in (field_info.metadata or []):
        if hasattr(m, "ge") and m.ge is not None:
            result["minimum"] = m.ge
        if hasattr(m, "le") and m.le is not None:
            result["maximum"] = m.le
        if hasattr(m, "pattern") and m.pattern is not None:
            result["pattern"] = m.pattern
    fi_extra = field_info.json_schema_extra
    if isinstance(fi_extra, dict):
        if "format" in fi_extra:
            result["format"] = fi_extra["format"]
        if "pattern" in fi_extra:
            result["pattern"] = fi_extra["pattern"]

    return result


def _get_root_annotation(cls: type) -> Any:
    """Get the root field type annotation from a RootModel."""
    fields = cls.model_fields
    if "root" in fields:
        return fields["root"].annotation
    return object


def _find_variant_types(cls: type) -> list[type]:
    """Find variant types for a class by scanning its module.

    Looks for {ClassName}Variant1, Variant2, etc. in the same module.
    """
    mod = sys.modules.get(cls.__module__)
    if not mod:
        return []
    base_name = cls.__name__
    variants: list[type] = []
    i = 1
    while True:
        variant_cls = getattr(mod, f"{base_name}Variant{i}", None)
        if variant_cls is None:
            break
        variants.append(variant_cls)
        i += 1
    return variants


def _convert_base_model(cls: type, root_title: str) -> dict:
    """Convert a BaseModel subclass to a JSON Schema object."""
    result: dict = {"type": "object"}

    # Discover variant types by naming convention (flatten pattern)
    variants = _find_variant_types(cls)
    if len(variants) == 1:
        # Single variant → emit its schema directly (e.g. $ref)
        variant_schema = _convert_union_member(variants[0], root_title)
        result.update(variant_schema)
    elif len(variants) > 1:
        # Multiple variants → emit anyOf
        result["anyOf"] = [_convert_union_member(v, root_title) for v in variants]

    properties = _convert_properties(cls, root_title)
    if properties:
        result["properties"] = properties

    # additionalProperties: false
    extra_setting = _get_extra_setting(cls)
    if extra_setting == "forbid":
        result["additionalProperties"] = False

    return result


def _convert_properties(cls: type, root_title: str) -> dict:
    """Convert BaseModel fields to JSON Schema properties."""
    properties: dict = {}
    fields = cls.model_fields

    for field_name, field_info in fields.items():
        prop_name = field_info.alias if field_info.alias else field_name
        tp = field_info.annotation
        prop_schema = _convert_property(tp, field_info, root_title)
        properties[prop_name] = prop_schema

    return properties


def _convert_property(tp: Any, field_info: FieldInfo, root_title: str) -> dict:
    """Convert a single property (type + field info) to JSON Schema."""
    result: dict = {}

    # Description from Field
    if field_info.description:
        result["description"] = field_info.description

    # Convert the type annotation to JSON Schema
    type_schema = convert_type(tp, root_title)

    # Extract constraints from field_info.metadata (for non-nullable props
    # where Pydantic merges Annotated Field into field_info.metadata)
    fi_constraints: dict = {}
    for m in (field_info.metadata or []):
        if hasattr(m, "ge") and m.ge is not None:
            fi_constraints["minimum"] = m.ge
        if hasattr(m, "le") and m.le is not None:
            fi_constraints["maximum"] = m.le
        if hasattr(m, "pattern") and m.pattern is not None:
            fi_constraints["pattern"] = m.pattern
    fi_extra = field_info.json_schema_extra
    if isinstance(fi_extra, dict):
        if "format" in fi_extra:
            fi_constraints["format"] = fi_extra["format"]
        if "pattern" in fi_extra:
            fi_constraints["pattern"] = fi_extra["pattern"]
        if "additionalProperties" in fi_extra:
            fi_constraints["additionalProperties"] = fi_extra["additionalProperties"]

    # Place constraints correctly:
    # - For nullable types: constraints should go inside the non-null anyOf variant
    # - For non-nullable types: constraints go directly on the property
    if fi_constraints:
        if "anyOf" in type_schema:
            # Nullable: overlay constraints on the non-null variant
            for variant in type_schema["anyOf"]:
                if variant.get("type") != "null":
                    variant.update(fi_constraints)
                    break
        else:
            type_schema.update(fi_constraints)

    result.update(type_schema)

    # Default value — but don't emit "default: null" for nullable fields
    # since that's just the implicit Optional default, not an explicit schema default
    if field_info.default is not PydanticUndefined:
        if field_info.default is None and _is_nullable_type(tp):
            pass  # Suppress implicit default: null for nullable fields
        else:
            result["default"] = field_info.default

    return result


def convert_top_level(cls: Any, title: str) -> dict:
    """Convert a Pydantic type to a complete JSON Schema with title and description."""
    result: dict = {"title": title}

    # Get description from docstring
    doc = getattr(cls, "__doc__", None)
    if doc:
        result["description"] = doc

    # Convert the type itself
    if isinstance(cls, type) and issubclass(cls, BaseModel) and not issubclass(cls, RootModel):
        inner = _convert_base_model(cls, title)
        result.update(inner)
    elif isinstance(cls, type) and issubclass(cls, RootModel):
        inner = _convert_root_model(cls, title)
        result.update(inner)
    else:
        inner = convert_type(cls, title)
        result.update(inner)

    return result


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------


@pytest.mark.parametrize("title", sorted(ALL_TITLES))
def test_roundtrip(title: str) -> None:
    """Verify Pydantic model → JSON Schema exactly matches the original."""
    pydantic_type = pydantic_types[title]

    if isinstance(pydantic_type, Exception):
        pytest.fail(f"Failed to import Pydantic type for '{title}': {pydantic_type}")

    converted = convert_top_level(pydantic_type, title)
    assert_schema_matches(title, converted)
