//! Immutable record sharing preserves proof, association and wire semantics.
mod common;
use proptest::prelude::*;
use rstest::rstest;
use taloon::{
    evidence::{Evidence, SidecarAssociation},
    model::{
        ArtifactFormat, ArtifactLocator, ArtifactRequest, Code, ContentHash, PackageId, SourceId,
    },
};
#[derive(Clone, Copy, Debug)]
enum ProofKind {
    Archive,
    Hashless,
    Sidecar,
}
#[derive(Clone, Copy, Debug)]
enum Change {
    Same,
    Artifact,
    Package,
    Hash,
    Format,
    Registry,
    Pin,
}
fn fixture(kind: ProofKind, change: Change) -> (Evidence, ArtifactRequest) {
    let meta = common::metadata("License-Expression: MIT");
    let bytes = common::wheel(&meta);
    let hash = ContentHash::digest(&bytes);
    let original = if matches!(kind, ProofKind::Hashless) {
        ArtifactRequest::new(
            PackageId::parse(
                "example",
                "1.0",
                SourceId::parse("path", "example.whl").unwrap(),
            )
            .unwrap(),
            ArtifactLocator::local("example.whl").unwrap(),
            ArtifactFormat::Wheel,
            None,
        )
        .unwrap()
    } else {
        common::request(&bytes, ArtifactFormat::Wheel)
    };
    let evidence = if matches!(kind, ProofKind::Sidecar) {
        Evidence::from_sidecar(
            &original,
            SidecarAssociation::new(hash, hash, ContentHash::digest(&meta)).unwrap(),
            &meta,
        )
        .unwrap()
    } else {
        Evidence::from_archive(&original, &bytes).unwrap()
    };
    let source = if matches!(change, Change::Registry) {
        SourceId::parse("registry", "https://other.test/simple").unwrap()
    } else {
        original.package().source().clone()
    };
    let name = if matches!(change, Change::Package) {
        "other"
    } else {
        "example"
    };
    let locator = if matches!(change, Change::Registry) {
        ArtifactLocator::registry(source.location(), "https://files.example.test/another.whl")
            .unwrap()
    } else if matches!(change, Change::Artifact) {
        if matches!(kind, ProofKind::Hashless) {
            ArtifactLocator::local("another.whl").unwrap()
        } else {
            ArtifactLocator::registry(source.location(), "https://files.example.test/another.whl")
                .unwrap()
        }
    } else {
        original.locator().clone()
    };
    let expected_hash = if matches!(change, Change::Hash) {
        Some(ContentHash::digest(b"wrong"))
    } else if matches!(change, Change::Pin) {
        Some(hash)
    } else {
        original.hash()
    };
    let format = if matches!(change, Change::Format) {
        ArtifactFormat::Sdist
    } else {
        original.format()
    };
    let target = ArtifactRequest::new(
        PackageId::parse(name, "1.0", source).unwrap(),
        locator,
        format,
        expected_hash,
    )
    .unwrap();
    (evidence, target)
}
#[rstest]
#[case::archive_same(ProofKind::Archive, Change::Same, None)]
#[case::archive_artifact(ProofKind::Archive, Change::Artifact, None)]
#[case::archive_registry(ProofKind::Archive, Change::Registry, None)]
#[case::archive_package(ProofKind::Archive, Change::Package, Some(Code::IdentityMismatch))]
#[case::archive_hash(ProofKind::Archive, Change::Hash, Some(Code::HashMismatch))]
#[case::archive_format(ProofKind::Archive, Change::Format, Some(Code::Cache))]
#[case::sidecar_same(ProofKind::Sidecar, Change::Same, None)]
#[case::sidecar_artifact(ProofKind::Sidecar, Change::Artifact, Some(Code::IdentityMismatch))]
#[case::sidecar_registry(ProofKind::Sidecar, Change::Registry, Some(Code::IdentityMismatch))]
#[case::sidecar_package(ProofKind::Sidecar, Change::Package, Some(Code::IdentityMismatch))]
#[case::sidecar_hash(ProofKind::Sidecar, Change::Hash, Some(Code::HashMismatch))]
#[case::sidecar_format(ProofKind::Sidecar, Change::Format, Some(Code::Cache))]
#[case::hashless_same(ProofKind::Hashless, Change::Same, None)]
#[case::hashless_artifact(ProofKind::Hashless, Change::Artifact, Some(Code::InvalidResult))]
#[case::hashless_package(ProofKind::Hashless, Change::Package, Some(Code::InvalidResult))]
#[case::hashless_format(ProofKind::Hashless, Change::Format, Some(Code::InvalidResult))]
#[case::hashless_pinned(ProofKind::Hashless, Change::Pin, None)]
fn shared_records_preserve_exact_proof_binding(
    #[case] kind: ProofKind,
    #[case] change: Change,
    #[case] expected: Option<Code>,
) {
    // Arrange
    let (evidence, target) = fixture(kind, change);
    let original = serde_json::to_vec(&evidence.to_record()).unwrap();
    // Act
    let result = evidence
        .rebind(&target)
        .map(|bound| serde_json::to_vec(&bound.to_record()).unwrap());
    // Assert
    if let Some(code) = expected {
        assert_eq!(result.err().unwrap().code, code);
    } else {
        assert_eq!(result.unwrap(), original);
    }
}
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn cloned_observations_keep_declarations_and_strength(
        sidecar in any::<bool>(),
        future in any::<bool>(),
        padding in "[a-zA-Z0-9]{0,128}",
    ) {
        // Arrange: independently selected proof and header version survive cloning.
        let version = if future { "2.7" } else { "2.4" };
        let meta = format!(
            "Metadata-Version: {version}\nName: example\nVersion: 1.0\nLicense-Expression: MIT\nX-Padding: {padding}\n\n"
        );
        let bytes = common::wheel(meta.as_bytes());
        let hash = ContentHash::digest(&bytes);
        let request = common::request(&bytes, ArtifactFormat::Wheel);
        let evidence = if sidecar {
            Evidence::from_sidecar(
                &request,
                SidecarAssociation::new(hash, hash, ContentHash::digest(meta.as_bytes())).unwrap(),
                meta.as_bytes(),
            ).unwrap()
        } else {
            Evidence::from_archive(&request, &bytes).unwrap()
        };
        let expected = serde_json::to_value(evidence.summary()).unwrap();
        let record_json = serde_json::to_value(evidence.to_record()).unwrap();

        // Act
        let rebound = evidence.clone().rebind(&request).unwrap();

        // Assert
        prop_assert_eq!(serde_json::to_value(rebound.summary()).unwrap(), expected);
        prop_assert_eq!(serde_json::to_value(rebound.to_record()).unwrap(), record_json);
        prop_assert_eq!(
            rebound.requirement_satisfied(taloon::model::EvidenceRequirement::ArchiveVerified),
            !sidecar,
        );
        prop_assert_eq!(rebound.summary().warnings.len(), usize::from(future));
    }
}
