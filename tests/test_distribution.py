# SPDX-License-Identifier: Elastic-2.0
"""Packaging promises visible to installers."""

import tomllib
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]


def distribution_section(
    data: dict[str, object], section: tuple[str, ...]
) -> dict[str, object]:
    from tools.json_data import object_mapping  # noqa: PLC0415 -- typed metadata boundary.

    for key in section:
        data = object_mapping(data[key])
    return data


@pytest.mark.parametrize(
    ("section", "field", "expected"),
    [
        (("project",), "name", "taloon"),
        (("project",), "requires-python", ">=3.14,<3.16"),
        (("project",), "dependencies", []),
        (("project",), "dynamic", ["version"]),
        (("project",), "license", "Elastic-2.0"),
        (("project",), "license-files", ["LICENSE"]),
        (("tool", "maturin"), "bindings", "bin"),
    ],
    ids=[
        "name",
        "requires-python",
        "dependencies",
        "dynamic",
        "license",
        "license-files",
        "bindings",
    ],
)
def test_binary_distribution_metadata(
    section: tuple[str, ...], field: str, expected: object
) -> None:
    # Arrange
    metadata = ROOT / "pyproject.toml"
    # Act
    data = tomllib.loads(metadata.read_text())
    actual = distribution_section(data, section)
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
