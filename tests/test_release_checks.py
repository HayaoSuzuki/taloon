# SPDX-License-Identifier: Elastic-2.0
"""Only successful workflows for the released push may authorize upload."""

import pytest

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
