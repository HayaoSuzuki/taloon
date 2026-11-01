# SPDX-License-Identifier: Elastic-2.0
"""Require completed checks for the exact released main push, never a PR."""

import argparse
import json
import re
import shutil
import subprocess
import time

from tools.json_data import object_list, object_mapping

REPOSITORY = "HayaoSuzuki/taloon"
WORKFLOWS = ("ci.yml", "documentation.yml", "dependency-audit.yml")


def _counter(value: object) -> int:
    if not isinstance(value, int) or isinstance(value, bool):
        msg = "Invalid workflow run counter"
        raise TypeError(msg)
    return value


def _run_order(run: dict[str, object]) -> tuple[int, int]:
    return _counter(run["run_number"]), _counter(run.get("run_attempt", 1))


def required_runs(
    runs: list[dict[str, object]], commit: str
) -> list[dict[str, object]]:
    """Select the newest main-push attempt for every required workflow."""
    selected = []
    for workflow in WORKFLOWS:
        matching = [
            run
            for run in runs
            if run.get("path") == ".github/workflows/" + workflow
            and run.get("head_sha") == commit
            and run.get("event") == "push"
        ]
        if not matching:
            continue
        latest = max(matching, key=_run_order)
        selected.append(latest)
    return selected


def checks_passed(runs: list[dict[str, object]], commit: str) -> bool:
    """Require successful completed checks for every release workflow."""
    selected = required_runs(runs, commit)
    return len(selected) == len(WORKFLOWS) and all(
        run.get("status") == "completed" and run.get("conclusion") == "success"
        for run in selected
    )


def wait(commit: str) -> None:
    """Poll exact-source workflow results until success, failure or the deadline."""
    if not re.fullmatch("[0-9a-f]{40}", commit):
        msg = "Expected full source commit"
        raise ValueError(msg)
    gh = shutil.which("gh")
    if gh is None:
        msg = "GitHub CLI is required"
        raise FileNotFoundError(msg)
    deadline = time.monotonic() + 1800
    while time.monotonic() < deadline:
        runs = []
        for workflow in WORKFLOWS:
            result = subprocess.run(  # noqa: S603 -- resolved CLI, fixed API path and validated commit.
                [
                    gh,
                    "api",
                    f"repos/{REPOSITORY}/actions/workflows/{workflow}/runs?head_sha={commit}&event=push&branch=main&per_page=100",
                ],
                check=True,
                capture_output=True,
                text=True,
                timeout=60,
            )
            payload = object_mapping(json.loads(result.stdout))
            runs.extend(
                object_mapping(value) for value in object_list(payload["workflow_runs"])
            )
        if checks_passed(runs, commit):
            print("Required main checks passed for " + commit)  # noqa: T201 -- command-line status/output contract.
            return
        for run in required_runs(runs, commit):
            if run.get("status") == "completed" and run.get("conclusion") in [
                "failure",
                "cancelled",
                "timed_out",
                "action_required",
            ]:
                msg = f"Required main check failed: {run['html_url']}"
                raise RuntimeError(msg)
        print("Waiting for required main checks", flush=True)  # noqa: T201 -- command-line status/output contract.
        time.sleep(20)
    msg = "Main checks did not complete within 30 minutes"
    raise TimeoutError(msg)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--commit", required=True)
    wait(parser.parse_args().commit)
