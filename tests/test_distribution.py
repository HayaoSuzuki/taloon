# SPDX-License-Identifier: Elastic-2.0
"""Packaging promises visible to installers."""

import tomllib
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]


@pytest.mark.parametrize(
    ("table", "field", "expected"),
    [
        ("project", "name", "taloon"),
        ("project", "requires-python", ">=3.14,<3.16"),
        ("project", "dependencies", []),
        ("project", "dynamic", ["version"]),
        ("project", "license", "Elastic-2.0"),
        ("project", "license-files", ["LICENSE"]),
        ("maturin", "bindings", "bin"),
    ],
)
def test_binary_distribution_metadata(table: str, field: str, expected: object) -> None:
    # Arrange
    metadata = ROOT / "pyproject.toml"
    # Act
    data = tomllib.loads(metadata.read_text())
    actual = data["project"] if table == "project" else data["tool"]["maturin"]
    # Assert
    assert actual[field] == expected


def test_release_platform_names_match_the_workflow_matrix() -> None:
    # Arrange
    import yaml  # noqa: PLC0415 -- workflow representation is a test boundary.

    from tools.release_policy import PLATFORM_NAMES  # noqa: PLC0415

    workflow = yaml.safe_load((ROOT / ".github/workflows/release.yml").read_text())
    matrix = workflow["jobs"]["wheels"]["strategy"]["matrix"]["include"]
    expected = tuple(row["platform"] for row in matrix)
    # Act
    observed = PLATFORM_NAMES
    # Assert
    assert observed == expected
    assert observed
