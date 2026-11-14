# SPDX-License-Identifier: Elastic-2.0
"""Keep JSON boundary types honest without changing opaque field values."""

import pytest
from hypothesis import given
from hypothesis import strategies as st

from tools.json_data import object_list, object_mapping

json_values = st.recursive(
    st.none() | st.booleans() | st.integers() | st.text(),
    lambda children: (
        st.lists(children, max_size=8)
        | st.dictionaries(st.text(), children, max_size=8)
    ),
    max_leaves=32,
)


@pytest.mark.parametrize(
    "value",
    [None, [], "object", {1: "not a JSON key"}],
    ids=["null", "array", "text", "nonstring-key"],
)
def test_object_mapping_rejects_invalid_shape(value: object) -> None:
    # Arrange: each parametrized input violates the mapping/key boundary.
    # Act / Assert: reject that single invalid value before field access.
    with pytest.raises(TypeError, match="Expected JSON object"):
        object_mapping(value)


@given(st.dictionaries(st.text(), json_values))
def test_object_mapping_preserves_valid_fields(value: dict[str, object]) -> None:
    # Arrange: generated mappings have JSON string keys and opaque field values.
    # Act
    observed = object_mapping(value)
    # Assert
    assert observed is value


@pytest.mark.parametrize(
    "value", [None, {}, "array", (1, 2)], ids=["null", "object", "text", "tuple"]
)
def test_object_list_rejects_invalid_shape(value: object) -> None:
    # Arrange: each parametrized value is not an array.
    # Act / Assert
    with pytest.raises(TypeError, match="Expected JSON array"):
        object_list(value)


@given(st.lists(json_values))
def test_object_list_preserves_valid_items(value: list[object]) -> None:
    # Arrange: item values are deliberately opaque to this shape boundary.
    # Act
    observed = object_list(value)
    # Assert
    assert observed is value
