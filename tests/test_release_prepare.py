# SPDX-License-Identifier: Elastic-2.0
"""Event identity and recovery selection at the GitHub boundary."""

import json
import subprocess
from typing import TYPE_CHECKING, Never

import pytest

from tools import release_prepare

if TYPE_CHECKING:
    from pathlib import Path


@pytest.fixture
def event(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    monkeypatch.chdir(tmp_path)
    (tmp_path / "Cargo.toml").write_text('[workspace.package]\nversion="0.1.0"\n')
    output = tmp_path / "output"
    for key, value in {
        "GITHUB_OUTPUT": str(output),
        "GITHUB_EVENT_NAME": "pull_request",
        "GITHUB_SHA": "a" * 40,
        "GITHUB_REF": "refs/pull/1/merge",
        "RESERVED_TAG": "",
        "RECOVERY_TAG": "",
    }.items():
        monkeypatch.setenv(key, value)
    return output


def test_preview_does_not_contact_release_service(
    event: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    # Arrange
    def forbidden(*_args: object, **_kwargs: object) -> Never:
        pytest.fail("preview attempted release access")

    monkeypatch.setattr(release_prepare.subprocess, "run", forbidden)
    # Act
    release_prepare.main()
    observed = dict(line.split("=", 1) for line in event.read_text().splitlines())
    # Assert
    assert observed == {
        "commit": "a" * 40,
        "tag": "",
        "version": "0.1.0",
        "publish": "false",
        "recovery": "false",
    }


def test_recovery_must_run_from_main(
    event: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    # Arrange
    monkeypatch.setenv("GITHUB_EVENT_NAME", "workflow_dispatch")
    monkeypatch.setenv("RECOVERY_TAG", "v0.1.0")
    # Act / Assert: refusal must precede output publication.
    with pytest.raises(
        ValueError, match="Recovery requires a stable tag and dispatch from main"
    ):
        release_prepare.main()
    assert not event.exists()


@pytest.fixture
def recovery(event: Path, monkeypatch: pytest.MonkeyPatch) -> list[tuple[str, ...]]:
    assert not event.exists()
    monkeypatch.setenv("GITHUB_EVENT_NAME", "workflow_dispatch")
    monkeypatch.setenv("GITHUB_REF", "refs/heads/main")
    monkeypatch.setenv("RECOVERY_TAG", "v0.1.0")
    monkeypatch.setattr(release_prepare.shutil, "which", lambda _name: "/fixture/gh")
    calls: list[tuple[str, ...]] = []

    def git(_root: Path, *args: str) -> subprocess.CompletedProcess[str]:
        calls.append(args)
        return subprocess.CompletedProcess(args, 0, "b" * 40 + "\n", "")

    monkeypatch.setattr(release_prepare, "git", git)
    return calls


def release_result(*, draft: bool) -> subprocess.CompletedProcess[str]:
    return subprocess.CompletedProcess(
        ["/fixture/gh"],
        0,
        json.dumps({"tagName": "v0.1.0", "isDraft": draft, "isPrerelease": False}),
        "",
    )


def test_recovery_uses_original_tag_commit_without_building(
    event: Path, recovery: list[tuple[str, ...]], monkeypatch: pytest.MonkeyPatch
) -> None:
    # Arrange
    monkeypatch.setattr(
        release_prepare.subprocess, "run", lambda *_a, **_k: release_result(draft=False)
    )
    # Act
    release_prepare.main()
    # Assert
    assert "commit=" + "b" * 40 in event.read_text()
    assert "recovery=true" in event.read_text()
    assert ("merge-base", "--is-ancestor", "b" * 40, "HEAD") in recovery


def test_draft_release_cannot_be_recovered(
    event: Path, recovery: list[tuple[str, ...]], monkeypatch: pytest.MonkeyPatch
) -> None:
    # Arrange
    monkeypatch.setattr(
        release_prepare.subprocess, "run", lambda *_a, **_k: release_result(draft=True)
    )
    # Act / Assert
    with pytest.raises(
        ValueError, match="Recovery requires a published stable release"
    ):
        release_prepare.main()
    assert not event.exists()
    assert recovery
