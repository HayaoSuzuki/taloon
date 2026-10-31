"""Exercise an installed binary wheel through uv, without a Rust toolchain."""

import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile
import zipfile


def smoke(wheel: Path, python: str) -> None:
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
        command = ["uv", "tool", "run", "--isolated", "--python", python,
                   "--from", str(wheel), "taloon"]

        def run(*args: str) -> subprocess.CompletedProcess:
            return subprocess.run(command + list(args), cwd=project, env=env,
                                  capture_output=True, text=True, timeout=120)

        version = wheel.name.split("-")[1]
        result = run("--version")
        assert result.returncode == 0, result.stderr
        assert result.stdout.strip() == "taloon " + version, result.stdout
        assert run("--help").returncode == 0
        with zipfile.ZipFile(project / "example.whl", "w") as archive:
            archive.writestr("example-1.0.dist-info/METADATA",
                            "Metadata-Version: 2.4\nName: example\nVersion: 1.0\n"
                            "License-Expression: MIT\n\n")
        (project / "uv.lock").write_text(
            "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\n"
            "source={path='example.whl'}\n", encoding="utf-8")

        def check(expected: int, *, hits: int) -> dict:
            result = run("check", "--offline", "--format", "json")
            assert result.returncode == expected, (result.stdout, result.stderr)
            report = json.loads(result.stdout)
            assert report["exit_code"] == expected
            assert len(report["packages"]) == 1
            assert report["acquisition"]["network_requests"] == 0
            assert report["acquisition"]["cache_hits"] == hits, report
            return report

        policy = project / "pyproject.toml"
        policy.write_text("[tool.taloon]\nschema-version=1\nallow=['MIT']\n")
        cold = check(0, hits=0)
        assert cold["acquisition"]["archive_extractions"] == 1
        warm = check(0, hits=1)
        assert warm["acquisition"]["archive_extractions"] == 0
        policy.write_text("[tool.taloon]\nschema-version=1\nallow=[]\n")
        assert check(1, hits=1)["status"] == "violation"
        (project / "example.whl").unlink()
        assert check(2, hits=0)["status"] == "incomplete"
    print(f"Wheel smoke passed: {wheel.name}, Python {python}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("wheel", type=Path)
    parser.add_argument("--python", required=True, choices=["3.14", "3.15"])
    args = parser.parse_args()
    smoke(args.wheel, args.python)


if __name__ == "__main__":
    main()
