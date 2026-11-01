# SPDX-License-Identifier: Elastic-2.0
"""Generalize fixed version-boundary cases with independent integer expectations."""

from hypothesis import given, settings
from hypothesis import strategies as st

from tools.release import next_version, version_tuple

versions = st.one_of(
    st.sampled_from([(0, 1, 0), (0, 1, 99), (2, 3, 4), (9, 0, 0)]),
    st.tuples(st.integers(0, 1000), st.integers(0, 1000), st.integers(0, 1000)),
)


def render(version: tuple[int, int, int]) -> str:
    return ".".join(map(str, version))


@settings(max_examples=256)
@given(floor=versions, history=st.lists(versions, max_size=32))
def test_next_version_respects_floor_and_minimal_patch_freshness(
    floor: tuple[int, int, int], history: list[tuple[int, int, int]]
) -> None:
    # Arrange: numeric ordering does not call production parsing or selection.
    text_floor = render(floor)
    text_history = list(map(render, history))
    # Act
    observed = next_version(text_floor, text_history)
    # Assert: above the floor, one patch decrement must reach an existing version.
    assert observed >= floor
    assert all(observed > previous for previous in history)
    if observed > floor:
        assert observed[2] > 0
        predecessor = (*observed[:2], observed[2] - 1)
        assert predecessor in history


@settings(max_examples=256)
@given(version=versions)
def test_version_parser_roundtrips_numeric_versions(
    version: tuple[int, int, int],
) -> None:
    # Arrange
    text = render(version)
    # Act
    observed = version_tuple(text)
    # Assert
    assert observed == version
