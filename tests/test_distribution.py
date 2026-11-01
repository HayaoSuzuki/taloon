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
