# SPDX-License-Identifier: Elastic-2.0
"""Exercise one reservation scenario per case against real local Git repositories."""

import hashlib
import runpy
import shutil
import subprocess
import sys
import tomllib
import zipfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import pytest

from tools import release
from tools.release_policy import RELEASE_PLATFORMS

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
    [
        ("collision", 2),
        ("lost-response", 1),
        ("collision-then-loss", 2),
        ("final-loss", 5),
    ],
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
        if ("collision" in fault and len(attempts) == 1) or (
            fault == "final-loss" and len(attempts) < 5
        ):
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


@pytest.mark.parametrize("version", ["0.1.1", "0.2.0", "1.0.0"])
def test_release_version_keeps_internal_decisions_locked(
    tmp_path: Path, version: str
) -> None:
    # Arrange: exercise real repository manifests with the release preparation.
    root = Path(__file__).resolve().parents[1]
    for name in ("Cargo.toml", "Cargo.lock"):
        shutil.copyfile(root / name, tmp_path / name)
    internal = tomllib.loads((root / "crates/taloon-decisions/Cargo.toml").read_text())

    # Act
    release.set_version(tmp_path, version)

    # Assert: Cargo's inherited version must still agree with the locked package.
    declared = internal["package"]["version"]
    actual = version if isinstance(declared, dict) else declared
    packages = tomllib.loads((tmp_path / "Cargo.lock").read_text())["package"]
    locked = next(p for p in packages if p["name"] == "taloon-decisions")
    assert locked["version"] == actual


def test_missing_git_cannot_reserve(
    repository: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    # Arrange
    monkeypatch.setattr(release.shutil, "which", lambda _name: None)
    # Act / Assert
    with pytest.raises(FileNotFoundError, match="Git is required"):
        release.git(repository, "status")


@pytest.mark.parametrize("commit", ["", "g" * 40, "a" * 39])
def test_invalid_reservation_commit_is_rejected(repository: Path, commit: str) -> None:
    # Arrange / Act / Assert
    with pytest.raises(ValueError, match="Expected full source commit"):
        release.reserve_tag(repository, commit)


@pytest.mark.parametrize(
    "manifest", ["", '[workspace.package]\nversion = "0.1.0"\n' * 2]
)
def test_version_update_does_not_partially_write_invalid_manifest(
    tmp_path: Path, manifest: str
) -> None:
    # Arrange
    (tmp_path / "Cargo.toml").write_text(manifest)
    # Act / Assert
    with pytest.raises(
        ValueError, match=r"Expected one package version in Cargo\.toml"
    ):
        release.set_version(tmp_path, "0.2.0")
    assert (tmp_path / "Cargo.toml").read_text() == manifest


def test_version_update_does_not_write_manifest_when_lock_is_invalid(
    tmp_path: Path,
) -> None:
    # Arrange
    original = '[workspace.package]\nversion = "0.1.0"\n'
    (tmp_path / "Cargo.toml").write_text(original)
    (tmp_path / "Cargo.lock").write_text("")
    # Act / Assert
    with pytest.raises(
        ValueError, match=r"Expected one package version in Cargo\.lock"
    ):
        release.set_version(tmp_path, "0.2.0")
    assert (tmp_path / "Cargo.toml").read_text() == original


def test_reservation_reports_exhausted_collisions(
    repository: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    # Arrange
    commit = git(repository, "rev-parse", "HEAD")
    original = release.git

    def reject(
        root: Path, *args: str, check: bool = True
    ) -> subprocess.CompletedProcess[str]:
        if args[0] == "push":
            return subprocess.CompletedProcess(args, 1, "", "collision")
        return original(root, *args, check=check)

    monkeypatch.setattr(release, "git", reject)
    # Act / Assert
    with pytest.raises(
        RuntimeError, match="Could not reserve release tag after five attempts"
    ):
        release.reserve_tag(repository, commit)
    assert release.remote_release_state(repository) == ({}, "")


def test_assembly_rejects_incomplete_inventory_before_creating_output(
    tmp_path: Path,
) -> None:
    # Arrange
    output = tmp_path / "output"
    # Act / Assert
    with pytest.raises(ValueError, match="Expected exactly the three supported wheels"):
        release.assemble(tmp_path, "0.1.0", tmp_path, output)
    assert not output.exists()


def test_assembly_copies_validated_wheels_and_hash_inventory(tmp_path: Path) -> None:
    # Arrange
    source = tmp_path / "source"
    source.mkdir()
    for platform in RELEASE_PLATFORMS:
        path = source / f"taloon-0.1.0-py3-none-{platform.published_wheel_tag}.whl"
        with zipfile.ZipFile(path, "w") as archive:
            archive.writestr(
                "taloon-0.1.0.dist-info/METADATA",
                "Metadata-Version: 2.4\nName: taloon\nVersion: 0.1.0\n"
                "Requires-Python: >=3.14,<3.16\nLicense-Expression: Elastic-2.0\n"
                "License-File: LICENSE\n\n",
            )
            archive.writestr(
                "taloon-0.1.0.dist-info/licenses/LICENSE",
                (Path(__file__).resolve().parents[1] / "LICENSE").read_bytes(),
            )
            executable = "taloon.exe" if "win_amd64" in path.name else "taloon"
            archive.writestr("taloon-0.1.0.data/scripts/" + executable, b"fixture")
            archive.writestr(
                "taloon-0.1.0.dist-info/WHEEL",
                "Wheel-Version: 1.0\nRoot-Is-Purelib: false\n"
                + "".join(
                    "Tag: py3-none-" + tag + "\n"
                    for tag in platform.published_wheel_tag.split(".")
                ),
            )
    output = tmp_path / "output"
    # Act
    release.assemble(tmp_path, "0.1.0", source, output)
    # Assert
    assert (output / "SHA256SUMS").read_text() == "".join(
        hashlib.sha256(path.read_bytes()).hexdigest() + "  " + path.name + "\n"
        for path in sorted(source.glob("*.whl"))
    )
    assert {path.name: path.read_bytes() for path in output.glob("*.whl")} == {
        path.name: path.read_bytes() for path in source.glob("*.whl")
    }


def test_cli_dispatches_version_update(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    # Arrange
    (tmp_path / "Cargo.toml").write_text('[workspace.package]\nversion = "0.1.0"\n')
    (tmp_path / "Cargo.lock").write_text(
        '[[package]]\nname = "taloon"\nversion = "0.1.0"\n'
    )
    monkeypatch.chdir(tmp_path)
    monkeypatch.setattr(sys, "argv", ["release", "set-version", "0.2.0"])
    # Act
    release.main()
    # Assert
    assert (
        tomllib.loads((tmp_path / "Cargo.toml").read_text())["workspace"]["package"][
            "version"
        ]
        == "0.2.0"
    )


def test_cli_prints_reserved_tag(
    repository: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    # Arrange
    monkeypatch.chdir(repository)
    monkeypatch.setattr(
        sys,
        "argv",
        ["release", "tag", "--commit", git(repository, "rev-parse", "HEAD")],
    )
    # Act
    release.main()
    # Assert
    assert capsys.readouterr().out == "v0.1.0\n"


def test_cli_assembly_rejects_missing_wheels(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    # Arrange
    output = tmp_path / "output"
    monkeypatch.setattr(
        sys,
        "argv",
        [
            "release",
            "assemble",
            "--version",
            "0.1.0",
            "--source",
            str(tmp_path),
            "--output",
            str(output),
        ],
    )
    # Act / Assert
    with pytest.raises(ValueError, match="Expected exactly the three supported wheels"):
        release.main()
    assert not output.exists()


def test_shallow_checkout_fetches_ancestry_before_accepting_source(
    repository: Path, tmp_path: Path
) -> None:
    # Arrange
    first = git(repository, "rev-parse", "HEAD")
    second = advance(repository)
    git(repository, "push", "origin", "HEAD:refs/heads/main")
    remote = git(repository, "remote", "get-url", "origin")
    shallow = tmp_path / "shallow"
    git(
        tmp_path,
        "clone",
        "--depth=1",
        "--branch=main",
        Path(remote).as_uri(),
        str(shallow),
    )
    # Act
    follows = release.follows_commit(shallow, second, first)
    # Assert
    assert follows
    assert git(shallow, "rev-parse", "--is-shallow-repository") == "false"


def test_release_module_exposes_help(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    # Arrange
    monkeypatch.setattr(sys, "argv", ["release", "--help"])
    # Act / Assert
    with pytest.raises(SystemExit) as error:
        runpy.run_path(str(release.__file__), run_name="__main__")
    assert error.value.code == 0
    assert "set-version" in capsys.readouterr().out
