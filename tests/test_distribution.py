# SPDX-License-Identifier: Elastic-2.0
"""Packaging promises visible to installers."""

import tomllib
from pathlib import Path

import pytest
from hypothesis import given
from hypothesis import strategies as st

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


@pytest.mark.parametrize(
    ("name", "rust_target", "wheel_tag"),
    [
        ("windows-x86_64", "x86_64-pc-windows-msvc", "win_amd64"),
        (
            "linux-x86_64",
            "x86_64-unknown-linux-gnu",
            "manylinux_2_17_x86_64.manylinux2014_x86_64",
        ),
        ("macos-aarch64", "aarch64-apple-darwin", "macosx_11_0_arm64"),
    ],
    ids=["windows", "linux", "macos"],
)
def test_release_platform_installer_identities(
    name: str, rust_target: str, wheel_tag: str
) -> None:
    # Arrange: explicit external compiler and installer identifiers.
    from tools.release_policy import RELEASE_PLATFORMS  # noqa: PLC0415

    platforms = {platform.name: platform for platform in RELEASE_PLATFORMS}
    # Act
    platform = platforms[name]
    # Assert
    assert platform.rust_target == rust_target
    assert platform.published_wheel_tag == wheel_tag


@pytest.mark.parametrize(
    ("name", "tag"),
    [
        ("windows-x86_64", "win_amd64"),
        ("linux-x86_64", "manylinux_2_17_x86_64.manylinux2014_x86_64"),
        ("macos-aarch64", "macosx_11_0_arm64"),
    ],
    ids=["windows", "linux", "macos"],
)
def test_release_pattern_accepts_supported_build_tag(name: str, tag: str) -> None:
    # Arrange
    import re  # noqa: PLC0415 -- protocol regex is the observed boundary.

    from tools.release_policy import RELEASE_PLATFORMS  # noqa: PLC0415

    platforms = {platform.name: platform for platform in RELEASE_PLATFORMS}
    # Act
    matched = re.fullmatch(platforms[name].release_wheel_pattern, tag)
    # Assert
    assert matched is not None


@given(
    platform_and_template=st.sampled_from(
        [
            ("linux-x86_64", "manylinux_2_{}_x86_64"),
            ("macos-aarch64", "macosx_{}_0_arm64"),
        ]
    ),
    version=st.integers(min_value=11, max_value=99),
)
def test_release_patterns_accept_generated_compatible_build_tags(
    platform_and_template: tuple[str, str], version: int
) -> None:
    # Arrange: platform tag grammars are independent of the policy regexes.
    import re  # noqa: PLC0415 -- observe compatibility at the regex boundary.

    from tools.release_policy import RELEASE_PLATFORMS  # noqa: PLC0415

    name, template = platform_and_template
    tag = template.format(version)
    platforms = {platform.name: platform for platform in RELEASE_PLATFORMS}
    # Act
    matched = re.fullmatch(platforms[name].release_wheel_pattern, tag)
    # Assert
    assert matched is not None
