"""Select one source identity; recovery consumes original assets, not rebuilds."""
import json
import os
from pathlib import Path
import re
import subprocess
import tomllib

from tools.release import git, version_tuple

REPOSITORY = 'HayaoSuzuki/taloon'


def main() -> None:
    event = os.environ['GITHUB_EVENT_NAME']
    commit = os.environ['GITHUB_SHA']
    if not re.fullmatch('[0-9a-f]{40}', commit):
        raise ValueError('Invalid event source')
    floor = tomllib.loads(Path('Cargo.toml').read_text())['workspace']['package']['version']
    version_tuple(floor)
    publish = event in ['push','workflow_dispatch'] and os.environ['GITHUB_REF'] == 'refs/heads/main'
    tag = os.environ.get('RECOVERY_TAG','') if event == 'workflow_dispatch' else os.environ.get('RESERVED_TAG','')
    if event == 'workflow_dispatch' and (not publish or not tag):
        raise ValueError('Recovery requires a stable tag and dispatch from main')
    if publish and not tag:
        # Another source overtook this unreserved commit. No publication follows.
        publish = False
    if tag:
        if not tag.startswith('v'): raise ValueError('Expected stable vMAJOR.MINOR.PATCH')
        version_tuple(tag[1:])
    recovery = False
    if publish:
        git(Path.cwd(), 'fetch', '--no-tags', 'origin', 'refs/tags/' + tag)
        commit = git(Path.cwd(), 'rev-parse', 'FETCH_HEAD^{commit}').stdout.strip()
        git(Path.cwd(), 'merge-base', '--is-ancestor', commit, 'HEAD')
        result = subprocess.run(['gh','release','view',tag,'--repo',REPOSITORY,
                                 '--json','tagName,isDraft,isPrerelease'],
                                capture_output=True,text=True,timeout=60)
        if result.returncode == 0:
            state = json.loads(result.stdout)
            if state != dict(tagName=tag,isDraft=False,isPrerelease=False):
                raise ValueError('Recovery requires a published stable release')
            recovery = True
        elif event == 'workflow_dispatch':
            raise RuntimeError('Recovery release is unavailable: ' + result.stderr)
        elif 'release not found' not in result.stderr.lower():
            raise RuntimeError('Cannot inspect release: ' + result.stderr)
    version = tag[1:] if tag else floor
    values = dict(commit=commit,tag=tag,version=version,
                  publish=str(publish).lower(),recovery=str(recovery).lower())
    with Path(os.environ['GITHUB_OUTPUT']).open('a') as output:
        for key,value in values.items(): output.write(f'{key}={value}\n')


if __name__ == '__main__': main()
