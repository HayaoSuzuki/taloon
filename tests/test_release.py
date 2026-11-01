# SPDX-License-Identifier: Elastic-2.0
"""Exercise one reservation scenario per case against real local Git repositories."""

import shutil
import subprocess
import tomllib
from concurrent.futures import ThreadPoolExecutor
from typing import TYPE_CHECKING

import pytest

from tools import release

if TYPE_CHECKING:
    from pathlib import Path

GIT = shutil.which("git") or pytest.fail("Git is required for reservation tests")


def git(root: Path, *args: str) -> str:
    return subprocess.check_output(  # noqa: S603 -- resolved Git; local fixture arguments.
        [GIT, "-C", str(root), *args], text=True, timeout=120
    ).strip()


@pytest.fixture
def repository(tmp_path: Path) -> Path:
    remote = tmp_path / "remote.git"
    git(tmp_path, "init", "--bare", str(remote))
    root = tmp_path / "checkout"
    git(tmp_path, "clone", str(remote), str(root))
    git(root, "config", "user.email", "fixture@example.invalid")
    git(root, "config", "user.name", "Fixture")
    git(root, "config", "commit.gpgsign", "false")
    (root / "Cargo.toml").write_text('[workspace.package]\nversion = "0.1.0"\n')
    git(root, "add", ".")
    git(root, "commit", "-m", "first")
    return root


def advance(root: Path) -> str:
    git(root, "commit", "--allow-empty", "-m", "next")
    return git(root, "rev-parse", "HEAD")


def test_initial_reservation_uses_manifest_floor(repository: Path) -> None:
    # Arrange
    commit = git(repository, "rev-parse", "HEAD")
    # Act
    observed = release.reserve_tag(repository, commit)
    # Assert
    assert observed == "v0.1.0"


def test_repeated_reservation_reuses_commit(repository: Path) -> None:
    # Arrange
    commit = git(repository, "rev-parse", "HEAD")
    original = release.reserve_tag(repository, commit)
    # Act
    observed = release.reserve_tag(repository, commit)
    # Assert
    assert observed == original
    assert (
        git(repository, "ls-remote", "origin", "refs/tags/v0.1.0").split()[0] == commit
    )


def test_stale_unreserved_commit_is_rejected(repository: Path) -> None:
    # Arrange
    release.reserve_tag(repository, git(repository, "rev-parse", "HEAD"))
    stale = advance(repository)
    newer = advance(repository)
    release.reserve_tag(repository, newer)
    # Act
    observed = release.reserve_tag(repository, stale)
    # Assert
    assert observed is None


def test_reserved_commit_survives_newer_reservation(repository: Path) -> None:
    # Arrange
    first = git(repository, "rev-parse", "HEAD")
    original = release.reserve_tag(repository, first)
    release.reserve_tag(repository, advance(repository))
    # Act
    observed = release.reserve_tag(repository, first)
    # Assert
    assert observed == original


@pytest.mark.parametrize(
    ("fault", "expected_attempts"),
    [("collision", 2), ("lost-response", 1), ("collision-then-loss", 2)],
)
def test_reservation_recovers_publication_fault(
    repository: Path,
    monkeypatch: pytest.MonkeyPatch,
    fault: str,
    expected_attempts: int,
) -> None:
    # Arrange
    original = release.git
    commit = git(repository, "rev-parse", "HEAD")
    attempts: list[tuple[str, ...]] = []

    def faulty_push(
        root: Path, *args: str, check: bool = True
    ) -> subprocess.CompletedProcess[str]:
        if args[0] != "push":
            return original(root, *args, check=check)
        attempts.append(args)
        if "collision" in fault and len(attempts) == 1:
            return subprocess.CompletedProcess(args, 1, "", "collision")
        result = original(root, *args, check=check)
        if fault != "collision":
            return subprocess.CompletedProcess(args, 1, result.stdout, "response lost")
        return result

    monkeypatch.setattr(release, "git", faulty_push)
    # Act
    observed = release.reserve_tag(repository, commit)
    # Assert
    assert observed == "v0.1.0"
    assert len(attempts) == expected_attempts


def test_reservation_respects_increased_floor(repository: Path) -> None:
    # Arrange
    release.reserve_tag(repository, git(repository, "rev-parse", "HEAD"))
    (repository / "Cargo.toml").write_text('[workspace.package]\nversion = "1.0.0"\n')
    git(repository, "add", ".")
    git(repository, "commit", "-m", "major")
    commit = git(repository, "rev-parse", "HEAD")
    # Act
    observed = release.reserve_tag(repository, commit)
    # Assert
    assert observed == "v1.0.0"


def test_unrelated_source_cannot_reserve(repository: Path) -> None:
    # Arrange
    release.reserve_tag(repository, git(repository, "rev-parse", "HEAD"))
    git(repository, "checkout", "--orphan", "unrelated")
    git(repository, "commit", "-m", "unrelated")
    commit = git(repository, "rev-parse", "HEAD")
    # Act
    observed = release.reserve_tag(repository, commit)
    # Assert
    assert observed is None


def test_competing_checkouts_do_not_overwrite_reservations(
    repository: Path, tmp_path: Path
) -> None:
    # Arrange
    first = git(repository, "rev-parse", "HEAD")
    second = advance(repository)
    other = tmp_path / "other"
    git(repository, "clone", str(repository), str(other))
    git(
        other,
        "remote",
        "set-url",
        "origin",
        git(repository, "remote", "get-url", "origin"),
    )
    # Act
    with ThreadPoolExecutor(max_workers=2) as pool:
        older_job = pool.submit(release.reserve_tag, repository, first)
        newer_job = pool.submit(release.reserve_tag, other, second)
        older, newer = older_job.result(), newer_job.result()
    tags, state = release.remote_release_state(repository)
    # Assert
    assert newer is not None
    assert newer in tags
    assert tags[newer] == second
    assert state == second
    assert older is None or tags[older] == first
    assert len(set(tags.values())) == len(tags)


@pytest.mark.parametrize(
    "bad", ["01.2.3", "1.2", "1.2.3\n", "-1.2.3", "1.2.3-dev.1", "v1.2.3"]
)
def test_invalid_stable_versions(bad: str) -> None:
    # Arrange: each independently named input violates stable-version syntax.
    # Act / Assert
    with pytest.raises(ValueError, match=r"Expected stable MAJOR.MINOR.PATCH"):
        release.version_tuple(bad)


@pytest.mark.parametrize(
    ("floor", "history", "expected"),
    [
        ("0.1.0", [], (0, 1, 0)),
        ("0.1.0", ["0.1.99"], (0, 1, 100)),
        ("9.0.0", ["2.3.4"], (9, 0, 0)),
        ("0.1.0", ["2.3.4"], (2, 3, 5)),
    ],
)
def test_next_version_named_boundaries(
    floor: str, history: list[str], expected: tuple[int, int, int]
) -> None:
    # Arrange: explicit boundary inputs are independent cases.
    # Act
    observed = release.next_version(floor, history)
    # Assert
    assert observed == expected


def test_set_version_preserves_other_packages_and_dynamic_metadata(
    tmp_path: Path,
) -> None:
    # Arrange
    (tmp_path / "Cargo.toml").write_text('[workspace.package]\nversion = "0.1.0"\n')
    (tmp_path / "Cargo.lock").write_text(
        '[[package]]\nname = "taloon"\nversion = "0.1.0"\n\n'
        '[[package]]\nname = "other"\nversion = "0.1.0"\n'
    )
    # Act
    release.set_version(tmp_path, "0.1.1")
    # Assert
    assert (
        tomllib.loads((tmp_path / "Cargo.toml").read_text())["workspace"]["package"][
            "version"
        ]
        == "0.1.1"
    )
    packages = tomllib.loads((tmp_path / "Cargo.lock").read_text())["package"]
    assert [p["version"] for p in packages] == ["0.1.1", "0.1.0"]
