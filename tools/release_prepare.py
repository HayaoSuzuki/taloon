# SPDX-License-Identifier: Elastic-2.0
"""Select one source identity; recovery consumes original assets, not rebuilds."""

import json
import os
import re
import shutil
import subprocess
import tomllib
from pathlib import Path

from tools.release import git, version_tuple

REPOSITORY = "HayaoSuzuki/taloon"


def _published_source(tag: str, event: str) -> tuple[str, bool]:
    git(Path.cwd(), "fetch", "--no-tags", "origin", "refs/tags/" + tag)
    commit = git(Path.cwd(), "rev-parse", "FETCH_HEAD^{commit}").stdout.strip()
    git(Path.cwd(), "merge-base", "--is-ancestor", commit, "HEAD")
    executable = shutil.which("gh")
    if executable is None:
        msg = "GitHub CLI is required"
        raise FileNotFoundError(msg)
    recovery = False
    result = subprocess.run(  # noqa: S603 -- resolved CLI and validated tag; no shell.
        [
            executable,
            "release",
            "view",
            tag,
            "--repo",
            REPOSITORY,
            "--json",
            "tagName,isDraft,isPrerelease",
        ],
        check=False,
        capture_output=True,
        text=True,
        timeout=60,
    )
    if result.returncode == 0:
        state = json.loads(result.stdout)
        if state != {"tagName": tag, "isDraft": False, "isPrerelease": False}:
            msg = "Recovery requires a published stable release"
            raise ValueError(msg)
        recovery = True
    elif event == "workflow_dispatch":
        raise RuntimeError("Recovery release is unavailable: " + result.stderr)
    elif "release not found" not in result.stderr.lower():
        raise RuntimeError("Cannot inspect release: " + result.stderr)
    return commit, recovery


def main() -> None:
    """Emit the single validated source identity for this workflow event."""
    event = os.environ["GITHUB_EVENT_NAME"]
    commit = os.environ["GITHUB_SHA"]
    if not re.fullmatch("[0-9a-f]{40}", commit):
        msg = "Invalid event source"
        raise ValueError(msg)
    floor = tomllib.loads(Path("Cargo.toml").read_text())["workspace"]["package"][
        "version"
    ]
    version_tuple(floor)
    publish = (
        event in ["push", "workflow_dispatch"]
        and os.environ["GITHUB_REF"] == "refs/heads/main"
    )
    tag = (
        os.environ.get("RECOVERY_TAG", "")
        if event == "workflow_dispatch"
        else os.environ.get("RESERVED_TAG", "")
    )
    if event == "workflow_dispatch" and (not publish or not tag):
        msg = "Recovery requires a stable tag and dispatch from main"
        raise ValueError(msg)
    if publish and not tag:
        # Another source overtook this unreserved commit. No publication follows.
        publish = False
    if tag:
        if not tag.startswith("v"):
            msg = "Expected stable vMAJOR.MINOR.PATCH"
            raise ValueError(msg)
        version_tuple(tag[1:])
    recovery = False
    if publish:
        commit, recovery = _published_source(tag, event)
    version = tag[1:] if tag else floor
    values = {
        "commit": commit,
        "tag": tag,
        "version": version,
        "publish": str(publish).lower(),
        "recovery": str(recovery).lower(),
    }
    with Path(os.environ["GITHUB_OUTPUT"]).open("a") as output:
        output.writelines(f"{key}={value}\n" for key, value in values.items())


if __name__ == "__main__":
    main()
