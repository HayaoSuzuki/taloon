# SPDX-License-Identifier: Elastic-2.0
"""Validate JSON container shapes while preserving opaque provider fields."""

from typing import cast


def object_mapping(value: object) -> dict[str, object]:
    """Require string object keys before exposing provider field values."""
    if not isinstance(value, dict) or any(not isinstance(key, str) for key in value):
        msg = "Expected JSON object"
        raise TypeError(msg)
    return cast("dict[str, object]", value)


def object_list(value: object) -> list[object]:
    """Require an array before inspecting individual provider records."""
    if not isinstance(value, list):
        msg = "Expected JSON array"
        raise TypeError(msg)
    return cast("list[object]", value)
