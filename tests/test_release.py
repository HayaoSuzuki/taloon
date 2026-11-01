"""Exercise reservations against real local Git repositories."""
import subprocess
from pathlib import Path

import pytest

from tools import release


def git(root, *args):
    return subprocess.check_output(['git', '-C', str(root), *args], text=True).strip()


@pytest.fixture
def repository(tmp_path):
    remote = tmp_path / 'remote.git'
    subprocess.run(['git', 'init', '--bare', str(remote)], check=True, capture_output=True)
    root = tmp_path / 'checkout'
    subprocess.run(['git', 'clone', str(remote), str(root)], check=True, capture_output=True)
    git(root, 'config', 'user.email', 'fixture@example.invalid')
    git(root, 'config', 'user.name', 'Fixture')
    (root / 'Cargo.toml').write_text('[workspace.package]\nversion = "0.1.0"\n')
    git(root, 'add', '.')
    git(root, 'commit', '-m', 'first')
    return root


def advance(root):
    git(root, 'commit', '--allow-empty', '-m', 'next')
    return git(root, 'rev-parse', 'HEAD')


def test_reservation_reuses_commit_and_rejects_overtaken(repository):
    root = repository
    first = git(root, 'rev-parse', 'HEAD')
    assert release.reserve_tag(root, first) == 'v0.1.0'
    assert release.reserve_tag(root, first) == 'v0.1.0'
    second = advance(root)
    third = advance(root)
    assert release.reserve_tag(root, third) == 'v0.1.1'
    assert release.reserve_tag(root, second) is None
    assert release.reserve_tag(root, first) == 'v0.1.0'
    assert git(root, 'ls-remote', 'origin', 'refs/tags/v0.1.0').split()[0] == first


def test_reservation_retries_collision_and_lost_response(repository, monkeypatch):
    root = repository
    original = release.git
    commit = git(root, 'rev-parse', 'HEAD')
    attempts = []

    def collide(root, *args, **kwargs):
        if args[0] == 'push':
            attempts.append(args)
            if len(attempts) == 1:
                return subprocess.CompletedProcess(args, 1, '', 'collision')
            result = original(root, *args, **kwargs)
            return subprocess.CompletedProcess(args, 1, result.stdout, 'response lost')
        return original(root, *args, **kwargs)

    monkeypatch.setattr(release, 'git', collide)
    assert release.reserve_tag(root, commit) == 'v0.1.0'
    assert len(attempts) == 2


def test_reservation_manifest_floor_and_unrelated_source(repository):
    root = repository
    first = git(root, 'rev-parse', 'HEAD')
    assert release.reserve_tag(root, first) == 'v0.1.0'
    (root / 'Cargo.toml').write_text('[workspace.package]\nversion = "1.0.0"\n')
    git(root, 'add', '.')
    git(root, 'commit', '-m', 'major')
    assert release.reserve_tag(root, git(root, 'rev-parse', 'HEAD')) == 'v1.0.0'
    git(root, 'checkout', '--orphan', 'unrelated')
    git(root, 'commit', '-m', 'unrelated')
    assert release.reserve_tag(root, git(root, 'rev-parse', 'HEAD')) is None


def test_competing_checkouts_do_not_overwrite_reservations(repository, tmp_path):
    from concurrent.futures import ThreadPoolExecutor
    root = repository
    first = git(root, 'rev-parse', 'HEAD')
    second = advance(root)
    other = tmp_path / 'other'
    git(root, 'clone', str(root), str(other))
    # Both clients see the same commits and remote, but reserve independently.
    git(other, 'remote', 'set-url', 'origin', git(root, 'remote', 'get-url', 'origin'))
    with ThreadPoolExecutor(max_workers=2) as pool:
        a = pool.submit(release.reserve_tag, root, first)
        b = pool.submit(release.reserve_tag, other, second)
        older, newer = a.result(), b.result()
    tags, state = release.remote_release_state(root)
    assert newer in tags and tags[newer] == second
    assert state == second
    assert older is None or tags[older] == first
    assert len(set(tags.values())) == len(tags)


@pytest.mark.parametrize('bad', ['01.2.3', '1.2', '1.2.3\n', '-1.2.3', '1.2.3-dev.1', 'v1.2.3'])
def test_invalid_stable_versions(bad):
    with pytest.raises(ValueError):
        release.version_tuple(bad)


def test_generated_next_versions_are_ordered_and_respect_floor():
    # Independent integer tuple expectations cover carry-free patch increments
    # and manifest floors across major/minor boundaries.
    for major in range(4):
        for minor in range(4):
            for patch in range(32):
                latest = f'{major}.{minor}.{patch}'
                assert release.next_version('0.1.0', [latest]) == max(
                    (0, 1, 0), (major, minor, patch + 1))
                assert release.next_version('9.0.0', [latest]) == (9, 0, 0)


def test_set_version_preserves_other_packages_and_dynamic_metadata(tmp_path):
    (tmp_path / 'Cargo.toml').write_text('[workspace.package]\nversion = "0.1.0"\n')
    (tmp_path / 'Cargo.lock').write_text('[[package]]\nname = "taloon"\nversion = "0.1.0"\n\n[[package]]\nname = "other"\nversion = "0.1.0"\n')
    release.set_version(tmp_path, '0.1.1')
    import tomllib
    assert tomllib.loads((tmp_path / 'Cargo.toml').read_text())['workspace']['package']['version'] == '0.1.1'
    packages = tomllib.loads((tmp_path / 'Cargo.lock').read_text())['package']
    assert [p['version'] for p in packages] == ['0.1.1', '0.1.0']
