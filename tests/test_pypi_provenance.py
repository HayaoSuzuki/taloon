# SPDX-License-Identifier: Elastic-2.0
"""Distribution trust failures and bounded index propagation."""

import base64
import binascii
import hashlib
import json
import subprocess
import sys
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


@pytest.mark.parametrize("value", [None, 1, []])
def test_statement_requires_encoded_text(value: object) -> None:
    # Arrange
    envelope = {"envelope": {"statement": value}}
    # Act / Assert
    with pytest.raises(TypeError, match="Invalid encoded"):
        provenance.statement_of(envelope)


def test_predicate_must_be_text() -> None:
    # Arrange
    data = {"subject": [{"name": "example.whl", "digest": {"sha256": "a" * 64}}]}
    value: dict[str, object] = {
        "envelope": {"statement": base64.b64encode(json.dumps(data).encode()).decode()}
    }
    # Act / Assert
    with pytest.raises(TypeError, match="Missing predicate"):
        provenance.statement_types([value], "example.whl", "a" * 64)


@pytest.mark.parametrize("records", [None, {}, "invalid"])
def test_inventory_requires_list(records: object) -> None:
    # Arrange / Act / Assert
    with pytest.raises(TypeError, match="Missing index file inventory"):
        provenance.check_inventory({"urls": records}, [])


def test_missing_gh_is_reported(monkeypatch: pytest.MonkeyPatch) -> None:
    # Arrange
    monkeypatch.setattr(provenance.shutil, "which", lambda _name: None)
    # Act / Assert
    with pytest.raises(FileNotFoundError, match="GitHub CLI is required"):
        provenance.run_gh(["attestation", "verify"])


@pytest.mark.parametrize("commit", ["", "a" * 39, "g" * 40])
def test_build_refuses_invalid_commit(tmp_path: Path, commit: str) -> None:
    # Arrange / Act / Assert
    with pytest.raises(ValueError, match="Expected full release commit SHA"):
        provenance.verify_build(tmp_path / "wheel", tmp_path / "bundle", commit)


def test_stage_preserves_existing_output(tmp_path: Path) -> None:
    # Arrange
    output = tmp_path / "output"
    output.mkdir()
    (output / "keep").write_text("original")
    # Act / Assert
    with pytest.raises(ValueError, match="Output already exists"):
        provenance.stage("v0.1.0", "a" * 40, tmp_path, tmp_path / "release", output)
    assert (output / "keep").read_text() == "original"


@pytest.mark.parametrize("tag", ["0.1.0", "v0.1.0rc1"])
def test_index_refuses_unstable_tag(tmp_path: Path, tag: str) -> None:
    # Arrange / Act / Assert
    with pytest.raises(ValueError, match="Expected stable release tag"):
        provenance.verify_index("pypi", tag, "a" * 40, tmp_path)


@pytest.mark.parametrize("allow_missing", [True, False])
def test_missing_index_is_allowed_only_during_inspection(
    tmp_path: Path, *, allow_missing: bool
) -> None:
    # Arrange
    error = urllib.error.HTTPError("https://pypi.org/", 404, "missing", Message(), None)
    with patch.object(provenance, "fetch_json", side_effect=error):
        # Act / Assert
        if allow_missing:
            assert not provenance.verify_index(
                "pypi", "v0.1.0", "a" * 40, tmp_path, allow_missing=True
            )
        else:
            with pytest.raises(urllib.error.HTTPError) as observed:
                provenance.verify_index("pypi", "v0.1.0", "a" * 40, tmp_path)
            assert observed.value is error


def test_index_requires_complete_platform_set(tmp_path: Path) -> None:
    # Arrange
    with (
        patch.object(provenance, "fetch_json", return_value={"urls": []}),
        pytest.raises(ValueError, match="staged wheels"),
    ):
        # Act / Assert
        provenance.verify_index("pypi", "v0.1.0", "a" * 40, tmp_path)


@pytest.mark.parametrize(
    "mutation", ["inventory", "host", "bytes", "bundles", "publisher", "attestations"]
)
def test_index_wheel_rejects_untrusted_evidence(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, mutation: str
) -> None:
    # Arrange
    import io  # noqa: PLC0415 -- in-memory HTTP boundary.

    wheel = tmp_path / "example.whl"
    wheel.write_bytes(b"original wheel")
    root = tmp_path / "download"
    root.mkdir()
    record = {
        "filename": wheel.name,
        "url": "https://files.pythonhosted.org/packages/example.whl",
    }
    data: dict[str, object] = {"urls": [record]}
    container: dict[str, object] = {
        "publisher": {
            "kind": "GitHub",
            "repository": provenance.REPOSITORY,
            "workflow": provenance.WORKFLOW,
        },
        "attestations": [],
    }
    response: dict[str, object] = {"attestation_bundles": [container]}
    errors = {
        "inventory": "Missing index file inventory",
        "host": "Unexpected index download host",
        "bytes": "Downloaded index wheel differs",
        "bundles": "Expected exactly one attestation bundle",
        "publisher": "Unexpected index publisher identity",
        "attestations": "Missing attestations",
    }
    if mutation == "inventory":
        data["urls"] = None
    elif mutation == "host":
        record["url"] = "https://attacker.example/packages/example.whl"
    elif mutation == "bundles":
        response["attestation_bundles"] = []
    elif mutation == "publisher":
        container["publisher"] = {
            "kind": "GitHub",
            "repository": "attacker/repo",
            "workflow": provenance.WORKFLOW,
        }
    elif mutation == "attestations":
        container["attestations"] = None
    payload = b"substituted" if mutation == "bytes" else wheel.read_bytes()
    monkeypatch.setattr(
        provenance.urllib.request, "urlopen", lambda *_a, **_k: io.BytesIO(payload)
    )
    monkeypatch.setattr(provenance, "fetch_json", lambda _url: response)
    # Act / Assert
    with pytest.raises((TypeError, ValueError), match=errors[mutation]):
        provenance.verify_index_wheel(
            "https://pypi.org/integrity/example", "a" * 40, wheel, data, root=root
        )
    assert wheel.read_bytes() == b"original wheel"


def test_fetch_json_decodes_response_object(monkeypatch: pytest.MonkeyPatch) -> None:
    # Arrange
    import io  # noqa: PLC0415 -- in-memory HTTP boundary.

    monkeypatch.setattr(
        provenance.urllib.request,
        "urlopen",
        lambda *_a, **_k: io.BytesIO(b'{"urls": []}'),
    )
    # Act
    observed = provenance.fetch_json("https://pypi.org/pypi/taloon/0.1.0/json")
    # Assert
    assert observed == {"urls": []}


@pytest.mark.parametrize("mode", ["inspect", "verify"])
def test_index_cli_reports_inspection_only(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, mode: str
) -> None:
    # Arrange
    output = tmp_path / "output"
    monkeypatch.setenv("GITHUB_OUTPUT", str(output))
    monkeypatch.setattr(
        "sys.argv",
        [
            "provenance",
            mode,
            "--tag",
            "v0.1.0",
            "--commit",
            "a" * 40,
            "--directory",
            str(tmp_path),
            "--index",
            "pypi",
        ],
    )
    monkeypatch.setattr(provenance, "verify_with_propagation", lambda *_a, **_k: True)
    # Act
    provenance.main()
    # Assert
    if mode == "inspect":
        assert output.read_text() == "published=true\n"
    else:
        assert not output.exists()


@pytest.mark.parametrize("mode", ["stage", "inspect"])
def test_cli_requires_mode_specific_arguments(
    monkeypatch: pytest.MonkeyPatch, mode: str
) -> None:
    # Arrange
    monkeypatch.setattr(
        "sys.argv",
        [
            "provenance",
            mode,
            "--tag",
            "v0.1.0",
            "--commit",
            "a" * 40,
            "--directory",
            "dist",
        ],
    )
    # Act / Assert
    with pytest.raises(SystemExit) as observed:
        provenance.main()
    assert observed.value.code == 2


def test_stage_cli_selects_requested_destination(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    # Arrange
    output = tmp_path / "staged"
    state = tmp_path / "release.json"
    monkeypatch.setattr(
        "sys.argv",
        [
            "provenance",
            "stage",
            "--tag",
            "v0.1.0",
            "--commit",
            "a" * 40,
            "--directory",
            str(tmp_path),
            "--release-json",
            str(state),
            "--output",
            str(output),
        ],
    )

    def stage(
        tag: str, commit: str, directory: Path, release_json: Path, destination: Path
    ) -> None:
        assert (tag, commit, directory, release_json) == (
            "v0.1.0",
            "a" * 40,
            tmp_path,
            state,
        )
        destination.mkdir()

    monkeypatch.setattr(provenance, "stage", stage)
    # Act
    provenance.main()
    # Assert
    assert output.is_dir()


@pytest.mark.parametrize("error_code", [403, 404])
def test_inspection_never_waits_for_errors(error_code: int) -> None:
    # Arrange
    error = urllib.error.HTTPError(
        "https://pypi.org/", error_code, "failure", Message(), None
    )
    with (
        patch.object(provenance, "verify_index", side_effect=error),
        patch.object(provenance.time, "sleep") as sleep,
        pytest.raises(urllib.error.HTTPError) as observed,
    ):
        # Act / Assert
        provenance.verify_with_propagation(
            "pypi", "v0.1.0", "a" * 40, Path("dist"), allow_missing=True
        )
    assert observed.value is error
    sleep.assert_not_called()


def test_deadline_after_sleep_stops_retry() -> None:
    # Arrange
    error = urllib.error.HTTPError("https://pypi.org/", 404, "failure", Message(), None)
    with (
        patch.object(provenance, "verify_index", side_effect=error),
        patch.object(provenance.time, "monotonic", side_effect=[0, 1, 601]),
        patch.object(provenance.time, "sleep"),
        pytest.raises(urllib.error.HTTPError) as observed,
    ):
        # Act / Assert
        provenance.verify_with_propagation(
            "pypi", "v0.1.0", "a" * 40, Path("dist"), allow_missing=False
        )
    assert observed.value is error


@pytest.mark.parametrize(
    "evidence",
    [
        "matching",
        "matching-after-other",
        "multiple-wheels",
        "later-invalid-signature",
        "missing-file",
        "wrong-subject",
        "invalid-signature",
    ],
)
def test_stage_publishes_only_verified_matching_build(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, evidence: str
) -> None:
    # Arrange: use the real checked-in signature envelope, substituting only its
    # cryptographic verification boundary because the fixture belongs to hoimin.
    value, _, _ = signature_case("none")
    raw = value.to_bundle().to_json()
    output = tmp_path / "staged"

    def prepare(_tag: str, _state: Path, _assets: Path, destination: Path) -> None:
        destination.mkdir()
        (destination / "taloon-0.1.0-py3-none-any.whl").write_bytes(b"original")
        if evidence == "multiple-wheels":
            (destination / "taloon-0.1.0-py3-none-other.whl").write_bytes(b"original")

    def gh(_args: list[str], *, cwd: Path | None = None) -> None:
        assert cwd is not None
        if evidence != "missing-file":
            (cwd / "bundle.jsonl").write_text(
                (raw + "\n")
                * (
                    2
                    if evidence in {"matching-after-other", "later-invalid-signature"}
                    else 1
                )
            )

    verification_calls: list[str] = []

    def verify(
        _self: Attestation, _publisher: GitHubPublisher, _distribution: Distribution
    ) -> tuple[str, dict[str, object]]:
        verification_calls.append(_distribution.name)
        if evidence == "invalid-signature" or (
            evidence == "later-invalid-signature" and len(verification_calls) == 2
        ):
            message = "signature rejected"
            raise ValueError(message)
        return provenance.SLSA, {}

    monkeypatch.setattr(provenance, "prepare_wheels", prepare)
    monkeypatch.setattr(provenance, "run_gh", gh)
    monkeypatch.setattr(provenance, "verify_build", lambda *_a: None)
    monkeypatch.setattr(Attestation, "verify", verify)
    monkeypatch.setattr(
        Distribution,
        "from_file",
        lambda _path: Distribution(
            name="taloon-0.1.0-py3-none-any.whl", digest="a" * 64
        ),
    )
    subjects = iter(
        ["taloon-0.1.0-py3-none-any.whl", "taloon-0.1.0-py3-none-other.whl"]
        if evidence == "multiple-wheels"
        else ["taloon-0.1.0-py3-none-any.whl"] * 2
        if evidence == "later-invalid-signature"
        else ["other.whl", "taloon-0.1.0-py3-none-any.whl"]
        if evidence == "matching-after-other"
        else [
            "other.whl"
            if evidence == "wrong-subject"
            else "taloon-0.1.0-py3-none-any.whl"
        ]
    )
    monkeypatch.setattr(
        provenance,
        "statement_of",
        lambda _value: {
            "subject": [
                {
                    "name": next(subjects),
                    "digest": {"sha256": hashlib.sha256(b"original").hexdigest()},
                }
            ]
        },
    )
    # Act / Assert
    if evidence in {"matching", "matching-after-other", "multiple-wheels"}:
        provenance.stage("v0.1.0", "a" * 40, tmp_path, tmp_path / "release", output)
        assert (output / "taloon-0.1.0-py3-none-any.whl").read_bytes() == b"original"
        staged = Attestation.model_validate_json(
            (output / "taloon-0.1.0-py3-none-any.whl.slsa.attestation").read_text()
        )
        assert staged.model_dump_json() == value.model_dump_json()
        if evidence == "multiple-wheels":
            assert (
                output / "taloon-0.1.0-py3-none-other.whl"
            ).read_bytes() == b"original"
            assert (
                output / "taloon-0.1.0-py3-none-other.whl.slsa.attestation"
            ).is_file()
    else:
        with pytest.raises(ValueError, match=r"Missing|signature rejected"):
            provenance.stage("v0.1.0", "a" * 40, tmp_path, tmp_path / "release", output)
        assert not output.exists()


def test_complete_index_verifies_each_platform(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    # Arrange
    wheels = [
        tmp_path / f"taloon-0.1.0-py3-none-{p.published_wheel_tag}.whl"
        for p in provenance.RELEASE_PLATFORMS
    ]
    for wheel in wheels:
        wheel.write_bytes(wheel.name.encode())
    records = [
        {
            "filename": w.name,
            "yanked": False,
            "digests": {"sha256": provenance.sha256(w)},
        }
        for w in wheels
    ]
    verified: list[str] = []
    monkeypatch.setattr(provenance, "fetch_json", lambda _url: {"urls": records})

    def verify(
        _url: str, _commit: str, wheel: Path, _data: dict[str, object], *, root: Path
    ) -> None:
        assert root.is_dir()
        verified.append(wheel.name)

    monkeypatch.setattr(provenance, "verify_index_wheel", verify)
    # Act
    observed = provenance.verify_index("pypi", "v0.1.0", "a" * 40, tmp_path)
    # Assert
    assert observed
    assert verified == sorted(w.name for w in wheels)


def test_index_verifies_both_signed_predicates(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    # Arrange
    import io  # noqa: PLC0415 -- in-memory HTTP boundary.

    wheel = tmp_path / "taloon-0.1.0-py3-none-any.whl"
    wheel.write_bytes(b"original")
    root = tmp_path / "download"
    root.mkdir()
    original, _, _ = signature_case("none")
    values = [
        statement(kind, name=wheel.name, digest=provenance.sha256(wheel))
        for kind in [provenance.SLSA, provenance.PUBLISH]
    ]
    container = {
        "publisher": {
            "kind": "GitHub",
            "repository": provenance.REPOSITORY,
            "workflow": provenance.WORKFLOW,
        },
        "attestations": values,
    }
    monkeypatch.setattr(
        provenance, "fetch_json", lambda _url: {"attestation_bundles": [container]}
    )
    monkeypatch.setattr(
        provenance.urllib.request, "urlopen", lambda *_a, **_k: io.BytesIO(b"original")
    )
    monkeypatch.setattr(Attestation, "model_validate", lambda _value: original)
    predicates = iter([provenance.SLSA, provenance.PUBLISH])
    monkeypatch.setattr(Attestation, "verify", lambda *_a: (next(predicates), {}))
    verified: list[str] = []

    def build(downloaded: Path, bundle: Path, commit: str) -> None:
        assert downloaded.read_bytes() == b"original"
        assert commit == "a" * 40
        Bundle.from_json(bundle.read_text())
        verified.append("build")

    def publish(downloaded: Path, bundle: Path) -> None:
        assert downloaded.read_bytes() == b"original"
        Bundle.from_json(bundle.read_text())
        verified.append("publish")

    monkeypatch.setattr(provenance, "verify_build", build)
    monkeypatch.setattr(provenance, "verify_publish", publish)
    # Act
    provenance.verify_index_wheel(
        "https://pypi.org/integrity/example",
        "a" * 40,
        wheel,
        {
            "urls": [
                {
                    "filename": wheel.name,
                    "url": "https://files.pythonhosted.org/packages/example.whl",
                }
            ]
        },
        root=root,
    )
    # Assert
    assert verified == ["build", "publish"]
    assert (root / wheel.name).read_bytes() == wheel.read_bytes()


def test_resolved_cli_runs_with_separate_arguments(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    # Arrange: execute an actual child, so dropped/combined arguments fail visibly.
    import sys  # noqa: PLC0415 -- fixed current interpreter for child process.

    output = tmp_path / "arguments.json"
    monkeypatch.setattr(provenance.shutil, "which", lambda _name: sys.executable)
    program = (
        "import json, pathlib, sys; "
        "pathlib.Path(sys.argv[1]).write_text(json.dumps(sys.argv[2:]))"
    )
    # Act
    provenance.run_gh(
        ["-c", program, str(output), "argument with spaces", "--literal"], cwd=tmp_path
    )
    # Assert
    assert json.loads(output.read_text()) == ["argument with spaces", "--literal"]


@pytest.mark.parametrize("module", ["tools.pypi_provenance", "tools.pypi_release"])
def test_module_cli_exposes_required_release_arguments(module: str) -> None:
    # Arrange
    import subprocess  # noqa: PLC0415 -- explicit CLI process boundary.
    import sys  # noqa: PLC0415 -- fixed current interpreter for child process.

    # Act
    observed = subprocess.run(  # noqa: S603 -- fixed interpreter, local module, no shell.
        [sys.executable, "-m", module, "--help"],
        capture_output=True,
        text=True,
        check=True,
    )
    # Assert
    assert "--tag" in observed.stdout
    assert "--release-json" in observed.stdout
    assert "--output" in observed.stdout


@pytest.mark.parametrize("encoded", ["!e30=", "e3\n0=", "e30=@@"])
def test_statement_rejects_non_base64_characters(encoded: str) -> None:
    # Arrange
    envelope = {"envelope": {"statement": encoded}}
    # Act / Assert
    with pytest.raises(binascii.Error):
        provenance.statement_of(envelope)


def test_failed_github_cli_cannot_be_treated_as_success(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    # Arrange
    monkeypatch.setattr(provenance.shutil, "which", lambda _name: sys.executable)
    # Act / Assert
    with pytest.raises(subprocess.CalledProcessError) as observed:
        provenance.run_gh(["-c", "raise SystemExit(7)"])
    assert observed.value.returncode == 7
