# SPDX-License-Identifier: Elastic-2.0
"""Event identity and recovery selection at the GitHub boundary."""

import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Never

import pytest

from tools import release_prepare


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
        pytest.fail(
            "preview attempted release access"
        )  # pragma: no cover -- failure-only guard; execution fails this test.

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
    commands: list[list[str]] = []

    def run(
        arguments: list[str], **_kwargs: object
    ) -> subprocess.CompletedProcess[str]:
        commands.append(arguments)
        return release_result(draft=False)

    monkeypatch.setattr(release_prepare.subprocess, "run", run)
    # Act
    release_prepare.main()
    # Assert
    assert "commit=" + "b" * 40 in event.read_text()
    assert "recovery=true" in event.read_text()
    assert len(commands) == 1
    assert commands[0][commands[0].index("--repo") + 1] == "HayaoSuzuki/taloon"
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


@pytest.mark.parametrize(
    "commit",
    ["", "a" * 39, "g" * 40, "A" * 40],
    ids=["empty", "short", "nonhex", "uppercase"],
)
def test_invalid_event_identity_never_emits_output(
    event: Path, monkeypatch: pytest.MonkeyPatch, commit: str
) -> None:
    # Arrange
    monkeypatch.setenv("GITHUB_SHA", commit)
    # Act / Assert
    with pytest.raises(ValueError, match="Invalid event source"):
        release_prepare.main()
    assert not event.exists()


def test_missing_github_cli_prevents_recovery_output(
    event: Path, recovery: list[tuple[str, ...]], monkeypatch: pytest.MonkeyPatch
) -> None:
    # Arrange
    monkeypatch.setattr(release_prepare.shutil, "which", lambda _name: None)
    # Act / Assert
    with pytest.raises(FileNotFoundError, match="GitHub CLI is required"):
        release_prepare.main()
    assert not event.exists()
    assert recovery


@pytest.mark.parametrize(
    ("workflow_event", "error"),
    [
        ("workflow_dispatch", "release not found"),
        ("push", "authentication failed"),
    ],
    ids=["workflow_dispatch", "push"],
)
def test_release_lookup_failure_never_authorizes_publication(
    event: Path,
    recovery: list[tuple[str, ...]],
    monkeypatch: pytest.MonkeyPatch,
    *,
    workflow_event: str,
    error: str,
) -> None:
    # Arrange
    monkeypatch.setenv("GITHUB_EVENT_NAME", workflow_event)
    monkeypatch.setenv("RESERVED_TAG", "v0.1.0")
    monkeypatch.setattr(
        release_prepare.subprocess,
        "run",
        lambda *_a, **_k: subprocess.CompletedProcess([], 1, "", error),
    )
    # Act / Assert
    with pytest.raises(RuntimeError, match=error):
        release_prepare.main()
    assert not event.exists()
    assert recovery


def test_unreserved_main_push_cannot_publish(
    event: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    # Arrange
    monkeypatch.setenv("GITHUB_EVENT_NAME", "push")
    monkeypatch.setenv("GITHUB_REF", "refs/heads/main")
    # Act
    release_prepare.main()
    observed = dict(line.split("=", 1) for line in event.read_text().splitlines())
    # Assert
    assert observed["publish"] == "false"
    assert observed["tag"] == ""
    assert observed["commit"] == "a" * 40


@pytest.mark.parametrize(
    "tag", ["0.1.0", "release-0.1.0"], ids=["no-prefix", "wrong-prefix"]
)
def test_non_version_tag_never_emits_output(
    event: Path, monkeypatch: pytest.MonkeyPatch, tag: str
) -> None:
    # Arrange
    monkeypatch.setenv("RESERVED_TAG", tag)
    # Act / Assert
    with pytest.raises(ValueError, match=r"Expected stable vMAJOR\.MINOR\.PATCH"):
        release_prepare.main()
    assert not event.exists()


def test_reserved_push_without_existing_release_selects_new_publication(
    event: Path, recovery: list[tuple[str, ...]], monkeypatch: pytest.MonkeyPatch
) -> None:
    # Arrange
    monkeypatch.setenv("GITHUB_EVENT_NAME", "push")
    monkeypatch.setenv("RESERVED_TAG", "v0.1.0")
    monkeypatch.setattr(
        release_prepare.subprocess,
        "run",
        lambda *_a, **_k: subprocess.CompletedProcess([], 1, "", "release not found"),
    )
    # Act
    release_prepare.main()
    observed = dict(line.split("=", 1) for line in event.read_text().splitlines())
    # Assert
    assert observed == {
        "commit": "b" * 40,
        "tag": "v0.1.0",
        "version": "0.1.0",
        "publish": "true",
        "recovery": "false",
    }
    assert recovery


def test_preview_module_emits_identity_without_release_access(
    event: Path,
) -> None:
    # Arrange
    environment = {
        **os.environ,
        "PYTHONPATH": str(Path(__file__).resolve().parents[1]),
        "PATH": "",
    }
    # Act
    observed = subprocess.run(
        [sys.executable, "-m", "tools.release_prepare"],
        env=environment,
        capture_output=True,
        text=True,
        check=False,
    )
    # Assert
    assert observed.returncode == 0, observed.stderr
    assert dict(line.split("=", 1) for line in event.read_text().splitlines()) == {
        "commit": "a" * 40,
        "tag": "",
        "version": "0.1.0",
        "publish": "false",
        "recovery": "false",
    }
