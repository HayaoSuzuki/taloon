"""Event identity and recovery selection at the GitHub boundary."""
import json
import subprocess
from pathlib import Path

import pytest
from tools import release_prepare


@pytest.fixture
def event(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    (tmp_path/'Cargo.toml').write_text('[workspace.package]\nversion="0.1.0"\n')
    output=tmp_path/'output'
    for key,value in dict(GITHUB_OUTPUT=str(output),GITHUB_EVENT_NAME='pull_request',
            GITHUB_SHA='a'*40,GITHUB_REF='refs/pull/1/merge',RESERVED_TAG='',RECOVERY_TAG='').items():
        monkeypatch.setenv(key,value)
    return output


def test_preview_does_not_contact_release_service(event, monkeypatch):
    def forbidden(*args,**kwargs): pytest.fail('preview attempted release access')
    monkeypatch.setattr(release_prepare.subprocess,'run',forbidden)
    release_prepare.main()
    data=dict(line.split('=',1) for line in event.read_text().splitlines())
    assert data == dict(commit='a'*40,tag='',version='0.1.0',publish='false',recovery='false')


def test_recovery_must_run_from_main(event, monkeypatch):
    monkeypatch.setenv('GITHUB_EVENT_NAME','workflow_dispatch')
    monkeypatch.setenv('RECOVERY_TAG','v0.1.0')
    with pytest.raises(ValueError): release_prepare.main()
    assert not event.exists()


@pytest.mark.parametrize('draft', [False,True])
def test_recovery_uses_original_tag_commit_without_building(event, monkeypatch, draft):
    monkeypatch.setenv('GITHUB_EVENT_NAME','workflow_dispatch')
    monkeypatch.setenv('GITHUB_REF','refs/heads/main')
    monkeypatch.setenv('RECOVERY_TAG','v0.1.0')
    calls=[]
    def git(root,*args):
        calls.append(args)
        return subprocess.CompletedProcess(args,0,'b'*40+'\n','')
    monkeypatch.setattr(release_prepare,'git',git)
    monkeypatch.setattr(release_prepare.subprocess,'run',lambda *a,**k:
        subprocess.CompletedProcess(a,0,json.dumps(dict(tagName='v0.1.0',isDraft=draft,isPrerelease=False)),''))
    if draft:
        with pytest.raises(ValueError): release_prepare.main()
        assert not event.exists()
    else:
        release_prepare.main()
        assert 'commit='+'b'*40 in event.read_text()
        assert 'recovery=true' in event.read_text()
        assert ('merge-base','--is-ancestor','b'*40,'HEAD') in calls
