# SPDX-License-Identifier: Elastic-2.0
"""Exercise an installed binary wheel through uv, without a Rust toolchain."""

import argparse
import json
import os
import shutil
import subprocess
import tempfile
import zipfile
from pathlib import Path

from tools.json_data import object_list, object_mapping


def require(*, condition: bool, detail: object = None) -> None:
    """Keep wheel verification active even when Python assertions are disabled."""
    if not condition:
        if detail is None:
            raise AssertionError
        raise AssertionError(detail)


def smoke(wheel: Path, python: str) -> None:
    """Exercise the installed wheel in isolated uv environments and a local project."""
    wheel = wheel.resolve(strict=True)
    with tempfile.TemporaryDirectory(prefix="taloon-wheel-") as temporary:
        root = Path(temporary)
        project = root / "project with spaces"
        project.mkdir()
        cache = root / "evidence"
        env = dict(os.environ, TALOON_CACHE_DIR=str(cache))
        env.pop("NETRC", None)
        # Every invocation ignores installed tools; only the supplied wheel may
        # provide taloon. Python selection affects installation, not the binary.
        executable = shutil.which("uv")
        if executable is None:
            msg = "uv is required for wheel verification"
            raise FileNotFoundError(msg)
        command = [
            executable,
            "tool",
            "run",
            "--isolated",
            "--python",
            python,
            "--from",
            str(wheel),
            "taloon",
        ]

        def run(*args: str) -> subprocess.CompletedProcess[str]:
            return subprocess.run(  # noqa: S603 -- resolved uv, selected interpreter and isolated wheel.
                command + list(args),
                check=False,
                cwd=project,
                env=env,
                capture_output=True,
                text=True,
                timeout=120,
            )

        version = wheel.name.split("-")[1]
        result = run("--version")
        require(condition=(result.returncode == 0), detail=result.stderr)
        require(
            condition=(result.stdout.strip() == "taloon " + version),
            detail=result.stdout,
        )
        require(condition=(run("--help").returncode == 0), detail=None)
        with zipfile.ZipFile(project / "example.whl", "w") as archive:
            archive.writestr(
                "example-1.0.dist-info/METADATA",
                "Metadata-Version: 2.4\nName: example\nVersion: 1.0\n"
                "License-Expression: MIT\n\n",
            )
        (project / "uv.lock").write_text(
            "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\n"
            "source={path='example.whl'}\n",
            encoding="utf-8",
        )

        def check(expected: int, *, hits: int) -> dict[str, object]:
            result = run("check", "--offline", "--format", "json")
            require(
                condition=(result.returncode == expected),
                detail=(result.stdout, result.stderr),
            )
            report = object_mapping(json.loads(result.stdout))
            require(condition=(report["exit_code"] == expected), detail=None)
            require(condition=(len(object_list(report["packages"])) == 1), detail=None)
            require(
                condition=(
                    object_mapping(report["acquisition"])["network_requests"] == 0
                ),
                detail=None,
            )
            require(
                condition=(object_mapping(report["acquisition"])["cache_hits"] == hits),
                detail=report,
            )
            return report

        policy = project / "pyproject.toml"
        policy.write_text("[tool.taloon]\nschema-version=1\nallow=['MIT']\n")
        cold = check(0, hits=0)
        require(
            condition=(object_mapping(cold["acquisition"])["archive_extractions"] == 1),
            detail=None,
        )
        warm = check(0, hits=1)
        require(
            condition=(object_mapping(warm["acquisition"])["archive_extractions"] == 0),
            detail=None,
        )
        policy.write_text("[tool.taloon]\nschema-version=1\nallow=[]\n")
        require(condition=(check(1, hits=1)["status"] == "violation"), detail=None)
        (project / "example.whl").unlink()
        require(condition=(check(2, hits=0)["status"] == "incomplete"), detail=None)
    print(f"Wheel smoke passed: {wheel.name}, Python {python}")  # noqa: T201 -- command-line status/output contract.


def main() -> None:
    """Probe one wheel with an explicitly supported Python interpreter."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("wheel", type=Path)
    parser.add_argument("--python", required=True, choices=["3.14", "3.15"])
    args = parser.parse_args()
    smoke(args.wheel, args.python)


if __name__ == "__main__":
    main()
