"""Shared helpers for push (streaming chunk accumulation)."""

from __future__ import annotations


def push_option_int(self_val: int | None, other_val: int | None) -> int | None:
    """Sum two optional ints (like Rust ``push_option_u64``)."""
    if self_val is not None and other_val is not None:
        return self_val + other_val
    if other_val is not None:
        return other_val
    return self_val


def push_option_string(self_val: str | None, other_val: str | None) -> str | None:
    """Concatenate two optional strings."""
    if self_val is not None and other_val is not None:
        return self_val + other_val
    if other_val is not None:
        return other_val
    return self_val


def push_option(self_val, other_val):
    """Merge two optional sub-objects by delegating to ``push()``.

    Both present → ``self_val.push(other_val)``, returns self_val.
    Only other → returns other_val (adopted).
    Only self / neither → returns self_val.
    """
    if self_val is not None and other_val is not None:
        self_val.push(other_val)
        return self_val
    if other_val is not None:
        return other_val
    return self_val


def push_by_index(self_list: list, other_list: list) -> None:
    """Merge *other_list* into *self_list* by ``index`` field.

    Items with a matching index are merged via ``push()``.
    New indices are appended.
    """
    from pydantic import RootModel

    index_map: dict[int, int] = {}
    for pos, item in enumerate(self_list):
        idx = _get_index(item)
        if idx is not None:
            index_map[idx] = pos

    for other_item in other_list:
        idx = _get_index(other_item)
        if idx is not None and idx in index_map:
            self_list[index_map[idx]].push(other_item)
        else:
            self_list.append(other_item)
            if idx is not None:
                index_map[idx] = len(self_list) - 1


def _get_index(item):
    """Extract an integer index from a model (BaseModel or RootModel)."""
    from pydantic import RootModel
    if isinstance(item, RootModel):
        inner = item.root
        if isinstance(inner, RootModel):
            inner = inner.root
        return getattr(inner, "index", None)
    return getattr(item, "index", None)
