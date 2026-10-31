"""Distribution trust failures and bounded index propagation."""
import base64
import hashlib
import json
from pathlib import Path
from unittest.mock import patch
import urllib.error

import pytest
from tools import pypi_provenance as provenance


def statement(kind, *, name='example.whl', digest='a'*64):
    data = dict(subject=[dict(name=name, digest={'sha256':digest})], predicateType=kind)
    return {'envelope': {'statement': base64.b64encode(json.dumps(data).encode()).decode()}}


def test_both_exact_subject_predicates_are_required():
    values = [statement(provenance.SLSA), statement(provenance.PUBLISH)]
    assert provenance.statement_types(values, 'example.whl', 'a'*64) == provenance.PREDICATES


@pytest.mark.parametrize('mutation', ['missing', 'duplicate', 'subject', 'digest'])
def test_wrong_predicates_or_subjects_rejected(mutation):
    values = [statement(provenance.SLSA), statement(provenance.PUBLISH)]
    if mutation == 'missing': values.pop()
    elif mutation == 'duplicate': values[1] = values[0]
    elif mutation == 'subject': values[1] = statement(provenance.PUBLISH, name='other.whl')
    else: values[1] = statement(provenance.PUBLISH, digest='b'*64)
    with pytest.raises(ValueError): provenance.statement_types(values, 'example.whl', 'a'*64)


def test_index_inventory_rejects_partial_yanked_and_substitution(tmp_path):
    wheel = tmp_path / 'example.whl'; wheel.write_bytes(b'bytes')
    record = dict(filename=wheel.name, yanked=False, digests={'sha256':hashlib.sha256(b'bytes').hexdigest()})
    provenance.check_inventory({'urls':[record]}, [wheel])
    for records in [[], [record, record], [dict(record,yanked=True)], [dict(record,digests={'sha256':'a'*64})]]:
        with pytest.raises(ValueError): provenance.check_inventory({'urls':records}, [wheel])


@pytest.mark.parametrize('failures', range(12))
def test_bounded_propagation_wait(failures):
    error = urllib.error.HTTPError('https://pypi.org/',404,'Not Found',{},None)
    clock = [0.0]
    def advance(delay): clock[0] += delay
    with patch.object(provenance,'verify_index',side_effect=[error]*failures+[True]), \
         patch.object(provenance.time,'monotonic',side_effect=lambda:clock[0]), \
         patch.object(provenance.time,'sleep',side_effect=advance):
        assert provenance.verify_with_propagation('pypi','v0.1.0','a'*40,Path('dist'),allow_missing=False)
    assert clock[0] <= 600


def test_invalid_publication_is_not_retried():
    with patch.object(provenance,'verify_index',side_effect=ValueError('digest')) as call, \
         pytest.raises(ValueError):
        provenance.verify_with_propagation('pypi','v0.1.0','a'*40,Path('dist'),allow_missing=False)
    assert call.call_count == 1


def test_build_verification_pins_repository_workflow_and_source(tmp_path):
    wheel = tmp_path/'example.whl'; wheel.touch()
    bundle = tmp_path/'bundle.json'; bundle.touch()
    with patch.object(provenance,'run_gh') as call:
        provenance.verify_build(wheel,bundle,'a'*40)
    args = call.call_args.args[0]
    assert args[args.index('--repo')+1] == 'HayaoSuzuki/taloon'
    assert args[args.index('--source-digest')+1] == 'a'*40
    assert args[args.index('--signer-digest')+1] == 'a'*40
    assert args[args.index('--signer-workflow')+1] == 'HayaoSuzuki/taloon/.github/workflows/release.yml'
    assert '--deny-self-hosted-runners' in args


def test_recovery_publish_identity_is_main_without_old_build_digest(tmp_path):
    wheel = tmp_path/'example.whl'; wheel.touch()
    bundle = tmp_path/'bundle.json'; bundle.touch()
    with patch.object(provenance,'run_gh') as call:
        provenance.verify_publish(wheel,bundle)
    args = call.call_args.args[0]
    assert args[args.index('--source-ref')+1] == 'refs/heads/main'
    assert args[args.index('--predicate-type')+1] == provenance.PUBLISH
    assert '--signer-digest' not in args


@pytest.mark.parametrize('mutation', ['none','publisher','source','digest','signature'])
def test_real_single_subject_signature_and_rejection(mutation):
    from pypi_attestations import Attestation, Distribution, GitHubPublisher, VerificationError
    from sigstore.models import Bundle
    from sigstore.verify import policy
    raw=json.loads((Path(__file__).parent/'fixtures/release/reference-build.sigstore.json').read_text())
    if mutation == 'signature':
        raw['dsseEnvelope']['signatures'][0]['sig'] = base64.b64encode(b'invalid signature').decode()
    value=Attestation.from_bundle(Bundle.from_json(json.dumps(raw)))
    subject=provenance.statement_of(value.model_dump(mode='json'))['subject'][0]
    distribution=Distribution(name=subject['name'],digest=subject['digest']['sha256'] if mutation!='digest' else '0'*64)
    publisher=GitHubPublisher(repository='HayaoSuzuki/hoimin',workflow='release.yml')
    if mutation == 'publisher': publisher=GitHubPublisher(repository='HayaoSuzuki/taloon',workflow='release.yml')
    elif mutation == 'source': publisher=policy.OIDCSourceRepositoryDigest('0'*40)
    if mutation=='none':
        assert value.verify(publisher,distribution,offline=True)[0] == provenance.SLSA
    else:
        with pytest.raises(VerificationError): value.verify(publisher,distribution,offline=True)
