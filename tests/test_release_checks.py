"""Only successful workflows for the released push may authorize upload."""
import pytest
from tools.release_checks import checks_passed, required_runs


def runs():
    return [dict(path=f'.github/workflows/{name}.yml', head_sha='a'*40,
                 event='push', status='completed', conclusion='success', run_number=1)
            for name in ['ci', 'documentation', 'dependency-audit']]


def test_complete_exact_main_checks_pass():
    assert checks_passed(runs(), 'a'*40)


@pytest.mark.parametrize('field,value', [('head_sha','b'*40), ('event','pull_request'),
    ('status','queued'), ('conclusion','failure'), ('path','.github/workflows/release.yml')])
def test_foreign_pending_failed_or_missing_checks_do_not_pass(field, value):
    data = runs(); data[0][field] = value
    assert not checks_passed(data, 'a'*40)


def test_newer_failure_supersedes_previous_success():
    data = runs(); data.append(dict(data[0], run_number=2, conclusion='failure'))
    assert not checks_passed(data, 'a'*40)


def test_failed_old_attempt_does_not_abort_current_retry():
    data = runs()
    data.append(dict(data[0], run_attempt=2,status='in_progress',conclusion=None))
    data[0]['conclusion'] = 'failure'
    selected = required_runs(data,'a'*40)
    assert not checks_passed(data,'a'*40)
    assert all(run['conclusion'] != 'failure' for run in selected)
