"""Reject incomplete or substituted distribution inventories before upload."""
import hashlib
import json
import zipfile
from pathlib import Path

import pytest

from tools.pypi_release import prepare_wheels, validate_wheel
from tools.release_policy import RELEASE_PLATFORMS

ROOT = Path(__file__).resolve().parents[1]


def wheel(path, *, license='Elastic-2.0', requirement='>=3.14,<3.16'):
    with zipfile.ZipFile(path, 'w') as z:
        z.writestr('taloon-0.1.0.dist-info/METADATA',
                   f'Metadata-Version: 2.4\nName: taloon\nVersion: 0.1.0\nRequires-Python: {requirement}\nLicense-Expression: {license}\nLicense-File: LICENSE\n\n')
        z.writestr('taloon-0.1.0.dist-info/licenses/LICENSE', (ROOT / 'LICENSE').read_bytes())
        executable = 'taloon.exe' if 'win_amd64' in path.name else 'taloon'
        z.writestr('taloon-0.1.0.data/scripts/' + executable, b'synthetic executable')
        tags = path.name.removesuffix('.whl').split('-')[-1].split('.')
        z.writestr('taloon-0.1.0.dist-info/WHEEL', 'Wheel-Version: 1.0\nRoot-Is-Purelib: false\n' + ''.join('Tag: py3-none-' + tag + '\n' for tag in tags))


def assets(tmp_path):
    directory = tmp_path / 'assets'
    directory.mkdir()
    for platform in RELEASE_PLATFORMS:
        wheel(directory / f'taloon-0.1.0-py3-none-{platform.published_wheel_tag}.whl')
    (directory / 'SHA256SUMS').write_text(''.join(
        hashlib.sha256(p.read_bytes()).hexdigest() + '  ' + p.name + '\n'
        for p in sorted(directory.glob('*.whl'))))
    state = tmp_path / 'release.json'
    state.write_text(json.dumps(dict(tagName='v0.1.0', isDraft=False, isPrerelease=False)))
    return directory, state


def test_prepare_complete_inventory(tmp_path):
    directory, state = assets(tmp_path)
    output = tmp_path / 'staged'
    prepare_wheels('v0.1.0', state, directory, output)
    assert sorted(p.name for p in output.iterdir()) == sorted(p.name for p in directory.glob('*.whl'))
    with pytest.raises(FileExistsError):
        prepare_wheels('v0.1.0', state, directory, output)


@pytest.mark.parametrize('mutation', ['missing', 'extra', 'bytes', 'license', 'python', 'duplicate', 'draft', 'sum'])
def test_inventory_failure_stages_nothing(tmp_path, mutation):
    directory, state = assets(tmp_path)
    first = sorted(directory.glob('*.whl'))[0]
    if mutation == 'missing': first.unlink()
    elif mutation == 'extra': wheel(directory / 'taloon-0.1.0-py3-none-any.whl')
    elif mutation == 'bytes': first.write_bytes(first.read_bytes() + b'changed')
    elif mutation in ['license', 'python']:
        wheel(first, **({'license': 'MIT'} if mutation == 'license' else {'requirement': '>=3.10'}))
    elif mutation == 'duplicate':
        with pytest.warns(UserWarning), zipfile.ZipFile(first, 'a') as z:
            z.writestr('taloon-0.1.0.dist-info/METADATA', 'duplicate')
    elif mutation == 'draft': state.write_text(json.dumps(dict(tagName='v0.1.0', isDraft=True, isPrerelease=False)))
    elif mutation == 'sum':
        p = directory / 'SHA256SUMS'; p.write_text(p.read_text() + p.read_text())
    output = tmp_path / 'staged'
    with pytest.raises(ValueError): prepare_wheels('v0.1.0', state, directory, output)
    assert not output.exists()


@pytest.mark.parametrize('field', ['license', 'requirement'])
def test_wheel_metadata_is_checked_independently_of_checksums(tmp_path, field):
    path = tmp_path / 'taloon-0.1.0-py3-none-win_amd64.whl'
    wheel(path, **{field: 'MIT' if field == 'license' else '>=3.10'})
    with pytest.raises(ValueError): validate_wheel(path, '0.1.0')


@pytest.mark.parametrize('mutation', ['script', 'wheel-tag', 'dependency', 'license-text'])
def test_internal_wheel_structure_is_required(tmp_path, mutation):
    path = tmp_path / 'taloon-0.1.0-py3-none-win_amd64.whl'
    wheel(path)
    with zipfile.ZipFile(path) as z:
        contents = {name: z.read(name) for name in z.namelist()}
    if mutation == 'script': del contents['taloon-0.1.0.data/scripts/taloon.exe']
    elif mutation == 'wheel-tag': contents['taloon-0.1.0.dist-info/WHEEL'] = b'Tag: py3-none-any\n'
    elif mutation == 'dependency': contents['taloon-0.1.0.dist-info/METADATA'] = contents['taloon-0.1.0.dist-info/METADATA'].replace(b'\n\n', b'\nRequires-Dist: evil\n\n')
    else: contents['taloon-0.1.0.dist-info/licenses/LICENSE'] = b'wrong license'
    with zipfile.ZipFile(path, 'w') as z:
        for name, value in contents.items(): z.writestr(name, value)
    with pytest.raises(ValueError): validate_wheel(path, '0.1.0')
