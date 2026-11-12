# SPDX-License-Identifier: Elastic-2.0
"""Only successful workflows for the released push may authorize upload."""

import json
import runpy
import subprocess
import sys
from itertools import count
from types import SimpleNamespace

import pytest

from tools import release_checks
from tools.release_checks import checks_passed, required_runs


def runs() -> list[dict[str, object]]:
    return [
        {
            "path": f".github/workflows/{name}.yml",
            "head_sha": "a" * 40,
            "event": "push",
            "status": "completed",
            "conclusion": "success",
            "run_number": 1,
        }
        for name in ["ci", "documentation", "dependency-audit"]
    ]


def test_complete_exact_main_checks_pass() -> None:
    # Arrange
    data = runs()
    # Act
    observed = checks_passed(data, "a" * 40)
    # Assert
    assert observed


@pytest.mark.parametrize(
    ("field", "value"),
    [
        ("head_sha", "b" * 40),
        ("event", "pull_request"),
        ("status", "queued"),
        ("conclusion", "failure"),
        ("path", ".github/workflows/release.yml"),
    ],
)
def test_foreign_pending_failed_or_missing_checks_do_not_pass(
    field: str, value: str
) -> None:
    # Arrange
    data = runs()
    data[0][field] = value
    # Act
    observed = checks_passed(data, "a" * 40)
    # Assert
    assert not observed


def test_newer_failure_supersedes_previous_success() -> None:
    # Arrange
    data = runs()
    data.append({**data[0], "run_number": 2, "conclusion": "failure"})
    # Act
    observed = checks_passed(data, "a" * 40)
    # Assert
    assert not observed


def test_failed_old_attempt_does_not_abort_current_retry() -> None:
    # Arrange
    data = runs()
    data.append(
        {**data[0], "run_attempt": 2, "status": "in_progress", "conclusion": None}
    )
    data[0]["conclusion"] = "failure"
    # Act
    selected = required_runs(data, "a" * 40)
    observed = checks_passed(data, "a" * 40)
    # Assert
    assert not observed
    assert all(run["conclusion"] != "failure" for run in selected)


@pytest.mark.parametrize(
    ("field", "value"), [("run_number", "2"), ("run_attempt", True)]
)
def test_invalid_workflow_counters_are_rejected(field: str, value: object) -> None:
    # Arrange
    data = runs()
    data[0][field] = value
    # Act / Assert
    with pytest.raises(TypeError, match="Invalid workflow run counter"):
        required_runs(data, "a" * 40)


@pytest.mark.parametrize("commit", ["", "a" * 39, "g" * 40])
def test_wait_rejects_invalid_source_before_polling(commit: str) -> None:
    # Arrange / Act / Assert
    with pytest.raises(ValueError, match="Expected full source commit"):
        release_checks.wait(commit)


def test_wait_requires_github_cli(monkeypatch: pytest.MonkeyPatch) -> None:
    # Arrange
    monkeypatch.setattr(release_checks.shutil, "which", lambda _name: None)
    # Act / Assert
    with pytest.raises(FileNotFoundError, match="GitHub CLI is required"):
        release_checks.wait("a" * 40)


@pytest.fixture
def workflow_service(monkeypatch: pytest.MonkeyPatch) -> list[dict[str, object]]:
    data = runs()
    # Finite virtual time prevents broken success checks from sleeping in real time.
    monkeypatch.setattr(
        release_checks,
        "time",
        SimpleNamespace(monotonic=count(0, 600).__next__, sleep=lambda _seconds: None),
    )
    monkeypatch.setattr(release_checks.shutil, "which", lambda _name: "/fixture/gh")

    def query(
        command: list[str], **_kwargs: object
    ) -> subprocess.CompletedProcess[str]:
        workflow = command[2].split("/workflows/")[1].split("/runs?")[0]
        matching = [r for r in data if r["path"] == ".github/workflows/" + workflow]
        return subprocess.CompletedProcess(
            command, 0, json.dumps({"workflow_runs": matching}), ""
        )

    monkeypatch.setattr(release_checks.subprocess, "run", query)
    return data


def test_wait_reports_exact_source_success(
    workflow_service: list[dict[str, object]], capsys: pytest.CaptureFixture[str]
) -> None:
    # Arrange
    assert len(workflow_service) == 3
    # Act
    release_checks.wait("a" * 40)
    # Assert
    assert (
        capsys.readouterr().out == "Required main checks passed for " + "a" * 40 + "\n"
    )


@pytest.mark.parametrize(
    "conclusion", ["failure", "cancelled", "timed_out", "action_required"]
)
def test_wait_rejects_terminal_failed_checks(
    workflow_service: list[dict[str, object]], conclusion: str
) -> None:
    # Arrange
    workflow_service[0].update(
        conclusion=conclusion, html_url="https://example.invalid/run/7"
    )
    # Act / Assert
    with pytest.raises(
        RuntimeError,
        match=r"Required main check failed: https://example\.invalid/run/7",
    ):
        release_checks.wait("a" * 40)


def test_wait_polls_pending_checks_before_authorizing(
    workflow_service: list[dict[str, object]],
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    # Arrange
    workflow_service[0]["status"] = "in_progress"

    def complete(_seconds: float) -> None:
        workflow_service[0]["status"] = "completed"

    monkeypatch.setattr(release_checks.time, "sleep", complete)
    # Act
    release_checks.wait("a" * 40)
    # Assert
    assert capsys.readouterr().out.splitlines() == [
        "Waiting for required main checks",
        "Required main checks passed for " + "a" * 40,
    ]


def test_wait_deadline_cannot_authorize_pending_checks(
    workflow_service: list[dict[str, object]], monkeypatch: pytest.MonkeyPatch
) -> None:
    # Arrange
    workflow_service[0]["status"] = "queued"
    clock = iter([0.0, 0.0, 1800.0])
    monkeypatch.setattr(release_checks.time, "monotonic", lambda: next(clock))
    monkeypatch.setattr(release_checks.time, "sleep", lambda _seconds: None)
    # Act / Assert
    with pytest.raises(
        TimeoutError, match="Main checks did not complete within 30 minutes"
    ):
        release_checks.wait("a" * 40)


def test_checks_module_requires_source_argument(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    # Arrange
    monkeypatch.setattr(sys, "argv", ["release_checks"])
    # Act / Assert
    with pytest.raises(SystemExit) as error:
        runpy.run_path(str(release_checks.__file__), run_name="__main__")
    assert error.value.code == 2
    assert "--commit" in capsys.readouterr().err
