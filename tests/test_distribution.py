"""Packaging promises visible to installers."""
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def test_binary_distribution_metadata():
    data = tomllib.loads((ROOT / 'pyproject.toml').read_text())
    assert data['project']['name'] == 'taloon'
    assert data['project']['requires-python'] == '>=3.14,<3.16'
    assert data['project']['dependencies'] == []
    assert data['tool']['maturin']['bindings'] == 'bin'
    assert data['project']['dynamic'] == ['version']
    assert data['project']['license'] == 'Elastic-2.0'
    assert data['project']['license-files'] == ['LICENSE']
