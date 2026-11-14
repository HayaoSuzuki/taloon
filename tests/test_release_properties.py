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
    # Admissible candidates are the floor and patch successors from history.
    # Minimizing that set under the freshness constraint is independent of selecting
    # the largest historical version and incrementing it in production.
    candidates = [
        floor,
        *((major, minor, patch + 1) for major, minor, patch in history),
    ]
    expected = min(
        candidate
        for candidate in candidates
        if candidate >= floor and all(candidate > previous for previous in history)
    )
    # Act
    observed = next_version(text_floor, text_history)
    # Assert
    assert observed == expected


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
