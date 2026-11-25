# SPDX-License-Identifier: Elastic-2.0
"""Packaging promises visible to installers."""

import json
import re
import tomllib
from pathlib import Path
from typing import TypedDict, cast

import pytest
import yaml
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
            "manylinux2014_x86_64.manylinux_2_17_x86_64",
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
        ("linux-x86_64", "manylinux2014_x86_64.manylinux_2_17_x86_64"),
        ("macos-aarch64", "macosx_11_0_arm64"),
    ],
    ids=["windows", "linux", "macos"],
)
def test_release_pattern_accepts_supported_build_tag(name: str, tag: str) -> None:
    # Arrange

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

    from tools.release_policy import RELEASE_PLATFORMS  # noqa: PLC0415

    name, template = platform_and_template
    tag = template.format(version)
    platforms = {platform.name: platform for platform in RELEASE_PLATFORMS}
    # Act
    matched = re.fullmatch(platforms[name].release_wheel_pattern, tag)
    # Assert
    assert matched is not None


PROMOTION_JOBS = (
    "testpypi-inspect",
    "testpypi-publish",
    "testpypi-verify",
    "pypi-inspect",
    "pypi-publish",
    "pypi-verify",
)


class PromotionStep(TypedDict, total=False):
    """Workflow step fields used by the promotion replay."""

    id: str
    env: dict[str, str]
    run: str


def promotion_enabled(
    condition: str, needs: dict[str, dict[str, str]], *, cancelled: bool
) -> bool:
    """Evaluate the conjunction and equality subset used by the promotion jobs."""
    for term in condition.split(" && "):
        if term == "!cancelled()":
            if cancelled:
                return False
        else:
            match = re.fullmatch(
                r"needs\.([a-z0-9-]+)\.(result|outputs\.[a-z]+) == '([a-z]+)'", term
            )
            assert match is not None, term
            job, field, expected = match.groups()
            if needs[job][field] != expected:
                return False
    return True


class PromotionScenario(TypedDict):
    """Workflow outcomes supplied as inputs to the Quint model."""

    testPresent: bool
    prodPresent: bool
    fault: int
    cancelAt: int


class PromotionCase(TypedDict):
    """Terminal job and upload observations from Quint."""

    scenario: PromotionScenario
    jobs: list[int]
    uploads: list[int]
    verified: bool


@pytest.fixture(scope="module")
def release_jobs() -> dict[str, dict[str, object]]:
    """Load the checked-in workflow that owns the actual gate expressions."""
    workflow = yaml.safe_load(
        (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
    )
    return cast("dict[str, dict[str, object]]", workflow["jobs"])


def publishing_step(job: dict[str, object]) -> dict[str, object]:
    """Find the pinned publisher step in a workflow job."""
    steps = cast("list[dict[str, object]]", job["steps"])
    [upload] = [
        step
        for step in steps
        if str(step.get("uses", "")).startswith("pypa/gh-action-pypi-publish@")
    ]
    return upload


def replay_release_promotion(
    jobs: dict[str, dict[str, object]], scenario: PromotionScenario
) -> tuple[list[int], list[int], bool]:
    """Apply the workflow's gates to scripted upstream and step outcomes."""
    fault = scenario["fault"]
    outcomes = {
        "prepare": {"result": "success", "outputs.publish": str(fault != 8).lower()},
        "stage": {"result": "failure" if fault == 7 else "success"},
        **{name: {"result": "skipped"} for name in PROMOTION_JOBS},
    }
    present = {"testpypi": scenario["testPresent"], "pypi": scenario["prodPresent"]}
    started: list[int] = []
    uploads: list[int] = []
    for phase, name in enumerate(PROMOTION_JOBS, start=1):
        job = jobs[name]
        dependencies = cast("list[str]", job["needs"])
        needs = {dependency: outcomes[dependency] for dependency in dependencies}
        cancelled = 0 < scenario["cancelAt"] <= phase
        if not promotion_enabled(cast("str", job["if"]), needs, cancelled=cancelled):
            continue

        started.append(phase)
        index, role = name.split("-", 1)
        upload = role == "publish" and promotion_enabled(
            cast("str", publishing_step(job)["if"]), needs, cancelled=False
        )
        if upload:
            uploads.append(phase)
        failed = fault == phase and (role != "publish" or upload)
        outcomes[name]["result"] = "failure" if failed else "success"
        if role == "inspect":
            outcomes[name]["outputs.published"] = str(present[index]).lower()

    return started, uploads, outcomes["pypi-verify"]["result"] == "success"


@pytest.mark.parametrize(
    "case",
    [
        json.loads(line)
        for line in (ROOT / "tests/fixtures/oracle/release-promotion.jsonl")
        .read_text(encoding="utf-8")
        .splitlines()
    ],
)
def test_quint_replays_release_promotion(
    case: PromotionCase, release_jobs: dict[str, dict[str, object]]
) -> None:
    # Arrange
    scenario = case["scenario"]

    # Act
    observed = replay_release_promotion(release_jobs, scenario)

    # Assert
    assert observed == (case["jobs"], case["uploads"], case["verified"])


@pytest.mark.parametrize("index", ["testpypi", "pypi"])
@pytest.mark.parametrize("mode", ["inspect", "verify"])
def test_release_promotion_checks_the_corresponding_index(
    index: str, mode: str, release_jobs: dict[str, dict[str, object]]
) -> None:
    # Arrange
    job = release_jobs[f"{index}-{mode}"]
    steps = cast("list[PromotionStep]", job["steps"])

    # Act
    [step] = [step for step in steps if step.get("id") == mode]

    # Assert
    assert step["env"]["INDEX"] == index
    assert f"tools.pypi_provenance {mode} " in step["run"]
    assert '--index "$INDEX"' in step["run"]
    if mode == "inspect":
        assert job["outputs"] == {"published": "${{ steps.inspect.outputs.published }}"}


@pytest.mark.parametrize(
    ("index", "repository_url"),
    [
        ("testpypi", "https://test.pypi.org/legacy/"),
        ("pypi", "https://upload.pypi.org/legacy/"),
    ],
)
def test_release_promotion_targets_the_corresponding_index(
    index: str, repository_url: str, release_jobs: dict[str, dict[str, object]]
) -> None:
    # Arrange
    job = release_jobs[f"{index}-publish"]

    # Act
    step = publishing_step(job)

    # Assert
    assert step["with"] == {
        "packages-dir": "dist/",
        "repository-url": repository_url,
        "attestations": True,
        "print-hash": True,
    }
