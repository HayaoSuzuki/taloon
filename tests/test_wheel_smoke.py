# SPDX-License-Identifier: Elastic-2.0
"""Wheel validation must not disappear under optimized Python execution."""

import subprocess
import sys


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
