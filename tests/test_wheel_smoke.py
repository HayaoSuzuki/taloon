# SPDX-License-Identifier: Elastic-2.0
"""Wheel validation must not disappear under optimized Python execution."""

import runpy
import subprocess
import sys
from pathlib import Path
from typing import Literal, TypedDict, Unpack

import pytest

from tools import wheel_smoke


class SmokeInvocation(TypedDict):
    """Typed process boundary shared by the smoke probe and local binary adapter."""

    check: bool
    cwd: Path
    env: dict[str, str]
    capture_output: bool
    text: Literal[True]
    timeout: int


def test_wheel_probe_checks_remain_active_with_optimization() -> None:
    # Arrange
    command = [
        sys.executable,
        "-O",
        "-c",
        (
            "from tools.wheel_smoke import require; "
            "require(condition=False, detail='probe rejected')"
        ),
    ]

    # Act
    observed = subprocess.run(  # noqa: S603 -- fixed interpreter and local source.
        command,
        check=False,
        capture_output=True,
        text=True,
    )

    # Assert
    assert observed.returncode != 0
    assert "AssertionError: probe rejected" in observed.stderr


@pytest.mark.parametrize(
    ("detail", "expected"),
    [(None, ()), ("invalid report", ("invalid report",))],
    ids=["no-detail", "with-detail"],
)
def test_require_rejects_failed_verification(
    detail: object, expected: tuple[object, ...]
) -> None:
    # Arrange / Act / Assert
    with pytest.raises(AssertionError) as error:
        wheel_smoke.require(condition=False, detail=detail)
    assert error.value.args == expected


def test_smoke_requires_installer(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    # Arrange
    wheel = tmp_path / "taloon-0.1.0-py3-none-any.whl"
    wheel.touch()
    monkeypatch.setattr(wheel_smoke.shutil, "which", lambda _name: None)
    # Act / Assert
    with pytest.raises(FileNotFoundError, match="uv is required"):
        wheel_smoke.smoke(wheel, "3.14")


@pytest.fixture
def installed_binary(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    wheel = tmp_path / "taloon-0.1.0-py3-none-any.whl"
    wheel.touch()
    binary = Path(__file__).resolve().parents[1] / "target/debug/taloon"
    assert binary.is_file(), "Build taloon before running installed binary smoke tests"
    original = subprocess.run
    monkeypatch.setattr(wheel_smoke.shutil, "which", lambda _name: "/fixture/uv")

    def invoke(
        command: list[str], **kwargs: Unpack[SmokeInvocation]
    ) -> subprocess.CompletedProcess[str]:
        return original([str(binary), *command[9:]], **kwargs)

    monkeypatch.setattr(wheel_smoke.subprocess, "run", invoke)
    return wheel


def test_smoke_checks_real_binary_cache_and_policy_transitions(
    installed_binary: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    # Arrange
    python_version = "3.14"

    # Act
    wheel_smoke.smoke(installed_binary, python_version)

    # Assert
    assert capsys.readouterr().out == (
        "Wheel smoke passed: taloon-0.1.0-py3-none-any.whl, Python 3.14\n"
    )


def test_wheel_probe_cli_runs_selected_binary(
    installed_binary: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    # Arrange
    monkeypatch.setattr(
        sys, "argv", ["wheel_smoke", str(installed_binary), "--python", "3.14"]
    )
    # Act
    wheel_smoke.main()
    # Assert
    assert "Wheel smoke passed:" in capsys.readouterr().out


def test_wheel_probe_module_requires_wheel(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    # Arrange
    monkeypatch.setattr(sys, "argv", ["wheel_smoke"])
    # Act / Assert
    with pytest.raises(SystemExit) as error:
        runpy.run_path(str(wheel_smoke.__file__), run_name="__main__")
    assert error.value.code == 2
    assert "wheel" in capsys.readouterr().err


def test_missing_wheel_is_rejected_before_installer_access(tmp_path: Path) -> None:
    # Arrange
    from unittest.mock import patch  # noqa: PLC0415 -- isolate installer boundary.

    missing = tmp_path / "missing.whl"
    # Act / Assert
    with (
        patch.object(wheel_smoke.subprocess, "run") as installer,
        pytest.raises(FileNotFoundError),
    ):
        wheel_smoke.smoke(missing, "3.14")
    installer.assert_not_called()
