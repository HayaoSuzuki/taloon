# SPDX-License-Identifier: Elastic-2.0
"""Distribution trust failures and bounded index propagation."""

import base64
import hashlib
import json
import tempfile
import urllib.error
from email.message import Message
from pathlib import Path
from unittest.mock import patch

import pytest
from hypothesis import given, settings
from hypothesis import strategies as st
from pypi_attestations import (
    Attestation,
    Distribution,
    GitHubPublisher,
    VerificationError,
)
from sigstore.models import Bundle
from sigstore.verify import policy

from tools import pypi_provenance as provenance
from tools.json_data import object_list, object_mapping


def statement(
    kind: str, *, name: str = "example.whl", digest: str = "a" * 64
) -> dict[str, object]:
    data = {
        "subject": [{"name": name, "digest": {"sha256": digest}}],
        "predicateType": kind,
    }
    return {
        "envelope": {"statement": base64.b64encode(json.dumps(data).encode()).decode()}
    }


def test_both_exact_subject_predicates_are_required() -> None:
    # Arrange
    values = [statement(provenance.SLSA), statement(provenance.PUBLISH)]
    # Act
    observed = provenance.statement_types(values, "example.whl", "a" * 64)
    # Assert
    assert observed == provenance.PREDICATES


@pytest.mark.parametrize(
    ("mutation", "message"),
    [
        ("missing", "Expected one SLSA"),
        ("duplicate", "Expected one SLSA"),
        ("subject", "Expected exactly one matching"),
        ("digest", "Expected exactly one matching"),
    ],
)
def test_wrong_predicates_or_subjects_rejected(mutation: str, message: str) -> None:
    # Arrange
    values = [statement(provenance.SLSA), statement(provenance.PUBLISH)]
    if mutation == "missing":
        values.pop()
    elif mutation == "duplicate":
        values[1] = values[0]
    elif mutation == "subject":
        values[1] = statement(provenance.PUBLISH, name="other.whl")
    else:
        values[1] = statement(provenance.PUBLISH, digest="b" * 64)
    # Act / Assert
    with pytest.raises(ValueError, match=message):
        provenance.statement_types(values, "example.whl", "a" * 64)


@pytest.fixture
def inventory_record(tmp_path: Path) -> tuple[Path, dict[str, object]]:
    wheel = tmp_path / "example.whl"
    wheel.write_bytes(b"bytes")
    return wheel, {
        "filename": wheel.name,
        "yanked": False,
        "digests": {"sha256": hashlib.sha256(b"bytes").hexdigest()},
    }


@pytest.mark.parametrize("mutation", ["missing", "duplicate", "yanked", "digest"])
def test_index_inventory_rejects_one_mutation(
    inventory_record: tuple[Path, dict[str, object]], mutation: str
) -> None:
    # Arrange
    wheel, record = inventory_record
    records = [record]
    if mutation == "missing":
        records = []
    elif mutation == "duplicate":
        records.append(record)
    elif mutation == "yanked":
        records = [{**record, "yanked": True}]
    else:
        records = [{**record, "digests": {"sha256": "a" * 64}}]
    message = (
        "Duplicate, invalid or yanked"
        if mutation in {"duplicate", "yanked"}
        else "Partial or mismatching"
    )
    # Act / Assert
    with pytest.raises(ValueError, match=message):
        provenance.check_inventory({"urls": records}, [wheel])


@settings(max_examples=128)
@given(
    payload=st.binary(max_size=256),
    mutation=st.sampled_from(["none", "missing", "duplicate", "yanked", "digest"]),
)
def test_generated_inventory_mutations_preserve_integrity(
    payload: bytes, mutation: str
) -> None:
    # Arrange: generalize the fixed inventory mutations over arbitrary wheel bytes.
    with tempfile.TemporaryDirectory() as directory:
        wheel = Path(directory) / "example.whl"
        wheel.write_bytes(payload)
        digest = hashlib.sha256(payload).hexdigest()
        record: dict[str, object] = {
            "filename": wheel.name,
            "yanked": mutation == "yanked",
            "digests": {"sha256": digest if mutation != "digest" else "invalid"},
        }
        records = (
            []
            if mutation == "missing"
            else [record] * (2 if mutation == "duplicate" else 1)
        )
        # Act
        try:
            provenance.check_inventory({"urls": records}, [wheel])
        except ValueError:
            accepted = False
        else:
            accepted = True
        # Assert
        assert accepted == (mutation == "none")


@pytest.mark.parametrize("failures", range(12))
def test_bounded_propagation_wait(failures: int) -> None:
    # Arrange
    error = urllib.error.HTTPError(
        "https://pypi.org/", 404, "Not Found", Message(), None
    )
    clock = [0.0]

    def advance(delay: float) -> None:
        clock[0] += delay

    with (
        patch.object(
            provenance, "verify_index", side_effect=[error] * failures + [True]
        ),
        patch.object(provenance.time, "monotonic", side_effect=lambda: clock[0]),
        patch.object(provenance.time, "sleep", side_effect=advance),
    ):
        # Act
        observed = provenance.verify_with_propagation(
            "pypi", "v0.1.0", "a" * 40, Path("dist"), allow_missing=False
        )
    # Assert
    assert observed
    assert clock[0] <= 600


def test_invalid_publication_is_not_retried() -> None:
    # Arrange
    with (
        patch.object(
            provenance, "verify_index", side_effect=ValueError("digest")
        ) as call,
        pytest.raises(ValueError, match="digest"),
    ):
        # Act / Assert
        provenance.verify_with_propagation(
            "pypi", "v0.1.0", "a" * 40, Path("dist"), allow_missing=False
        )
    assert call.call_count == 1


def test_build_verification_pins_repository_workflow_and_source(tmp_path: Path) -> None:
    # Arrange
    wheel, bundle = tmp_path / "example.whl", tmp_path / "bundle.json"
    wheel.touch()
    bundle.touch()
    with patch.object(provenance, "run_gh") as call:
        # Act
        provenance.verify_build(wheel, bundle, "a" * 40)
    # Assert
    args = call.call_args.args[0]
    assert args[args.index("--repo") + 1] == "HayaoSuzuki/taloon"
    assert args[args.index("--source-digest") + 1] == "a" * 40
    assert args[args.index("--signer-digest") + 1] == "a" * 40
    assert (
        args[args.index("--signer-workflow") + 1]
        == "HayaoSuzuki/taloon/.github/workflows/release.yml"
    )
    assert "--deny-self-hosted-runners" in args


def test_recovery_publish_identity_is_main_without_old_build_digest(
    tmp_path: Path,
) -> None:
    # Arrange
    wheel, bundle = tmp_path / "example.whl", tmp_path / "bundle.json"
    wheel.touch()
    bundle.touch()
    with patch.object(provenance, "run_gh") as call:
        # Act
        provenance.verify_publish(wheel, bundle)
    # Assert
    args = call.call_args.args[0]
    assert args[args.index("--source-ref") + 1] == "refs/heads/main"
    assert args[args.index("--predicate-type") + 1] == provenance.PUBLISH
    assert "--signer-digest" not in args


def signature_case(
    mutation: str,
) -> tuple[Attestation, Distribution, GitHubPublisher | policy.VerificationPolicy]:
    raw = json.loads(
        (
            Path(__file__).parent / "fixtures/release/reference-build.sigstore.json"
        ).read_text()
    )
    if mutation == "signature":
        raw["dsseEnvelope"]["signatures"][0]["sig"] = base64.b64encode(
            b"invalid signature"
        ).decode()
    value = Attestation.from_bundle(Bundle.from_json(json.dumps(raw)))
    subjects = object_list(
        provenance.statement_of(value.model_dump(mode="json"))["subject"]
    )
    subject = object_mapping(subjects[0])
    name = subject["name"]
    digest = object_mapping(subject["digest"])["sha256"]
    assert isinstance(name, str)
    assert isinstance(digest, str)
    distribution = Distribution(
        name=name, digest=digest if mutation != "digest" else "0" * 64
    )
    publisher: GitHubPublisher | policy.VerificationPolicy = GitHubPublisher(
        repository="HayaoSuzuki/hoimin", workflow="release.yml"
    )
    if mutation == "publisher":
        publisher = GitHubPublisher(
            repository="HayaoSuzuki/taloon", workflow="release.yml"
        )
    elif mutation == "source":
        publisher = policy.OIDCSourceRepositoryDigest("0" * 40)
    return value, distribution, publisher


def test_real_single_subject_signature_is_valid() -> None:
    # Arrange
    value, distribution, publisher = signature_case("none")
    # Act
    observed = value.verify(publisher, distribution, offline=True)
    # Assert
    assert observed[0] == provenance.SLSA


@pytest.mark.parametrize("mutation", ["publisher", "source", "digest", "signature"])
def test_real_single_subject_signature_rejects_one_mutation(mutation: str) -> None:
    # Arrange
    value, distribution, publisher = signature_case(mutation)
    # Act / Assert
    with pytest.raises(VerificationError):
        value.verify(publisher, distribution, offline=True)
