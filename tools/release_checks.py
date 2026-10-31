"""Require completed checks for the exact released main push, never a PR."""
import argparse
import json
import re
import subprocess
import time

REPOSITORY = 'HayaoSuzuki/taloon'
WORKFLOWS = ('ci.yml', 'documentation.yml', 'dependency-audit.yml')


def required_runs(runs: list[dict], commit: str) -> list[dict]:
    selected = []
    for workflow in WORKFLOWS:
        matching = [run for run in runs if
                    run.get('path') == '.github/workflows/' + workflow
                    and run.get('head_sha') == commit and run.get('event') == 'push']
        if not matching:
            continue
        latest = max(matching, key=lambda r: (r['run_number'], r.get('run_attempt', 1)))
        selected.append(latest)
    return selected


def checks_passed(runs: list[dict], commit: str) -> bool:
    selected = required_runs(runs, commit)
    return len(selected) == len(WORKFLOWS) and all(
        run.get('status') == 'completed' and run.get('conclusion') == 'success'
        for run in selected)


def wait(commit: str) -> None:
    if not re.fullmatch('[0-9a-f]{40}', commit):
        raise ValueError('Expected full source commit')
    deadline = time.monotonic() + 1800
    while time.monotonic() < deadline:
        runs = []
        for workflow in WORKFLOWS:
            result = subprocess.run(['gh','api',
                f'repos/{REPOSITORY}/actions/workflows/{workflow}/runs?head_sha={commit}&event=push&branch=main&per_page=100'],
                check=True, capture_output=True, text=True, timeout=60)
            runs.extend(json.loads(result.stdout)['workflow_runs'])
        if checks_passed(runs, commit):
            print('Required main checks passed for ' + commit)
            return
        for run in required_runs(runs, commit):
            if run.get('status') == 'completed' and run.get('conclusion') in ['failure','cancelled','timed_out','action_required']:
                raise RuntimeError('Required main check failed: ' + run['html_url'])
        print('Waiting for required main checks', flush=True)
        time.sleep(20)
    raise TimeoutError('Main checks did not complete within 30 minutes')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--commit', required=True)
    wait(parser.parse_args().commit)
