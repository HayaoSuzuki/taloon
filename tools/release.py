"""Own stable release identities and content inventories.

Reservation protocol adapted from HayaoSuzuki/hoimin at
39d35f3980c428ab15258b519a233b86f1551ffa (Elastic-2.0).
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import tomllib

from tools.release_policy import SEMVER, RELEASE_PLATFORMS

RELEASE_STATE_REF = 'refs/heads/taloon-release-state'


def version_tuple(version: str) -> tuple[int, int, int]:
    if not re.fullmatch(SEMVER, version):
        raise ValueError('Expected stable MAJOR.MINOR.PATCH')
    return tuple(map(int, version.split('.')))


def next_version(floor: str, versions: list[str]) -> tuple[int, int, int]:
    minimum = version_tuple(floor)
    if not versions:
        return minimum
    major, minor, patch = max(map(version_tuple, versions))
    return max(minimum, (major, minor, patch + 1))


def git(root: Path, *args: str, check=True) -> subprocess.CompletedProcess:
    return subprocess.run(['git', '-C', str(root), *args], check=check,
                          text=True, capture_output=True, timeout=120)


def remote_release_state(root: Path) -> tuple[dict[str, str], str]:
    refs = {}
    for line in git(root, 'ls-remote', 'origin', 'refs/tags/v*', RELEASE_STATE_REF).stdout.splitlines():
        commit, ref = line.split()
        refs[ref] = commit
    tags = {ref.removeprefix('refs/tags/'): refs.get(ref + '^{}', commit)
            for ref, commit in refs.items() if re.fullmatch('refs/tags/v' + SEMVER, ref)}
    return tags, refs.get(RELEASE_STATE_REF, '')


def follows_commit(root: Path, commit: str, ancestor: str) -> bool:
    if git(root, 'rev-parse', '--is-shallow-repository').stdout.strip() == 'true':
        git(root, 'fetch', '--unshallow', '--no-tags', 'origin')
    git(root, 'fetch', '--no-tags', 'origin', ancestor)
    result = git(root, 'merge-base', '--is-ancestor', ancestor, commit, check=False)
    if result.returncode == 1:
        return False
    result.check_returncode()
    return True


def reserve_tag(root: Path, commit: str) -> str | None:
    if not re.fullmatch('[0-9a-f]{40}', commit):
        raise ValueError('Expected full source commit')
    git(root, 'cat-file', '-e', commit + '^{commit}')
    floor = tomllib.loads((root / 'Cargo.toml').read_text())['workspace']['package']['version']
    version_tuple(floor)
    for _ in range(5):
        tags, state = remote_release_state(root)
        existing = [tag for tag, sha in tags.items() if sha == commit]
        if existing:
            return max(existing, key=lambda tag: version_tuple(tag[1:]))
        ancestors = {state} if state else set()
        if tags:
            ancestors.add(tags[max(tags, key=lambda tag: version_tuple(tag[1:]))])
        if any(not follows_commit(root, commit, ancestor) for ancestor in ancestors):
            return None
        version = '.'.join(map(str, next_version(floor, [tag[1:] for tag in tags])))
        tag = 'v' + version
        # The state ref is a CAS lease shared by all versions. A tag and its
        # ordering marker must both update or neither updates. Tags never force.
        pushed = git(root, 'push', '--atomic',
                     f'--force-with-lease={RELEASE_STATE_REF}:{state}', 'origin',
                     f'{commit}:refs/tags/{tag}', f'{commit}:{RELEASE_STATE_REF}', check=False)
        if pushed.returncode == 0:
            return tag
    tags, _ = remote_release_state(root)
    existing = [tag for tag, sha in tags.items() if sha == commit]
    if existing:
        return max(existing, key=lambda tag: version_tuple(tag[1:]))
    raise RuntimeError('Could not reserve release tag after five attempts')


def set_version(root: Path, version: str) -> None:
    version_tuple(version)
    changes = []
    for name, pattern in [
        ('Cargo.toml', r'(\[workspace\.package\]\nversion = ")[^"]+("\n)'),
        ('Cargo.lock', r'(\[\[package\]\]\nname = "taloon"\nversion = ")[^"]+("\n)'),
    ]:
        path = root / name
        text, count = re.subn(pattern, lambda m: m[1] + version + m[2], path.read_text())
        if count != 1:
            raise ValueError(f'Expected one package version in {name}')
        changes.append((path, text))
    for path, text in changes:
        path.write_text(text)
    # Python metadata is dynamic; uv.lock records an editable dynamic project.


def assemble(root: Path, version: str, source: Path, output: Path) -> None:
    from tools.pypi_release import validate_wheel
    version_tuple(version)
    wheels = sorted(source.glob('*.whl'))
    expected = {f'taloon-{version}-py3-none-{p.published_wheel_tag}.whl' for p in RELEASE_PLATFORMS}
    if {p.name for p in wheels} != expected:
        raise ValueError('Expected exactly the three supported wheels')
    for wheel in wheels:
        validate_wheel(wheel, version)
    output.mkdir(parents=True, exist_ok=False)
    sums = []
    for wheel in wheels:
        shutil.copyfile(wheel, output / wheel.name)
        sums.append(hashlib.sha256(wheel.read_bytes()).hexdigest() + '  ' + wheel.name)
    (output / 'SHA256SUMS').write_text('\n'.join(sums) + '\n')


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    tag = sub.add_parser('tag'); tag.add_argument('--commit', required=True)
    version = sub.add_parser('set-version'); version.add_argument('version')
    pack = sub.add_parser('assemble')
    pack.add_argument('--version', required=True)
    pack.add_argument('--source', type=Path, required=True)
    pack.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    root = Path.cwd()
    if args.command == 'tag':
        result = reserve_tag(root, args.commit)
        if result is not None: print(result)
    elif args.command == 'set-version': set_version(root, args.version)
    elif args.command == 'assemble': assemble(root, args.version, args.source, args.output)


if __name__ == '__main__':
    main()
