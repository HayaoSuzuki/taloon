mod common;
use common::{metadata, request, sdist, wheel};
use proptest::prelude::*;
use taloon::{
    evidence::{Evidence, EvidenceRecord, SidecarAssociation},
    model::{ArtifactFormat, Code, ContentHash, EvidenceRequirement},
};

#[test]
fn archive_proof_satisfies_archive_requirement() {
    // Arrange
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let req = request(&bytes, ArtifactFormat::Wheel);
    // Act
    let archive = Evidence::from_archive(&req, &bytes).unwrap();
    // Assert
    assert!(archive.requirement_satisfied(EvidenceRequirement::ArchiveVerified));
}

#[test]
fn sidecar_proof_satisfies_only_index_requirement() {
    // Arrange
    let meta = metadata("License-Expression: MIT");
    let bytes = wheel(&meta);
    let req = request(&bytes, ArtifactFormat::Wheel);
    let hash = ContentHash::digest(&bytes);
    let association = SidecarAssociation::new(hash, hash, ContentHash::digest(&meta)).unwrap();
    // Act
    let sidecar = Evidence::from_sidecar(&req, association, &meta).unwrap();
    // Assert
    assert!(sidecar.requirement_satisfied(EvidenceRequirement::IndexAssociated));
    assert!(!sidecar.requirement_satisfied(EvidenceRequirement::ArchiveVerified));
}

#[test]
fn sidecar_association_rejects_wrong_artifact_hash() {
    // Arrange
    let hash = ContentHash::digest(b"artifact");
    // Act
    let result = SidecarAssociation::new(hash, ContentHash::digest(b"wrong"), hash);
    // Assert
    assert!(result.is_err());
}

#[test]
fn sidecar_rejects_wrong_metadata_hash() {
    // Arrange
    let meta = metadata("License-Expression: MIT");
    let bytes = wheel(&meta);
    let req = request(&bytes, ArtifactFormat::Wheel);
    let hash = ContentHash::digest(&bytes);
    let association = SidecarAssociation::new(hash, hash, ContentHash::digest(b"wrong")).unwrap();
    // Act
    let result = Evidence::from_sidecar(&req, association, &meta);
    // Assert
    assert!(result.is_err());
}

#[rstest::rstest]
#[case::expression_precedes_legacy("License-Expression: MIT\nLicense: Apache-2.0", Some("MIT"))]
#[case::invalid_expression_does_not_fall_back("License-Expression: WRONG\nLicense: MIT", None)]
#[case::legacy_expression("License: MIT OR Apache-2.0", Some("MIT OR Apache-2.0"))]
#[case::classifier_is_not_expression("Classifier: License :: OSI Approved :: MIT License", None)]
#[case::prose_is_not_expression("License: This is MIT-ish", None)]
fn metadata_license_precedence(#[case] fields: &str, #[case] expected: Option<&str>) {
    // Arrange
    let bytes = wheel(&metadata(fields));
    let req = request(&bytes, ArtifactFormat::Wheel);
    // Act
    let evidence = Evidence::from_archive(&req, &bytes).unwrap();
    // Assert
    assert_eq!(
        evidence
            .license()
            .ok()
            .as_ref()
            .map(taloon::policy::LicenseExpression::text),
        expected
    );
}

#[test]
fn wrong_metadata_name_reports_identity_mismatch() {
    // Arrange
    let wrong = String::from_utf8(metadata("License-Expression: MIT"))
        .unwrap()
        .replace("Name: example", "Name: wrong");
    let bytes = wheel(wrong.as_bytes());
    let req = request(&bytes, ArtifactFormat::Wheel);
    // Act
    let result = Evidence::from_archive(&req, &bytes);
    // Assert
    assert_eq!(result.unwrap_err().code, Code::IdentityMismatch);
}

#[rstest::rstest]
#[case::folded("License-Expression: MIT OR\n Apache-2.0", true)]
#[case::duplicate("License-Expression: MIT\nLicense-Expression: Apache-2.0", false)]
fn metadata_expression_validity(#[case] fields: &str, #[case] expected: bool) {
    // Arrange
    let bytes = wheel(&metadata(fields));
    let req = request(&bytes, ArtifactFormat::Wheel);
    // Act
    let evidence = Evidence::from_archive(&req, &bytes).unwrap();
    // Assert
    assert_eq!(evidence.license().is_ok(), expected);
}

#[rstest::rstest]
#[case::historical("1.2", 0)]
#[case::current("2.6", 0)]
#[case::future_minor("2.7", 1)]
fn supported_metadata_version_warning_count(#[case] version: &str, #[case] warnings: usize) {
    // Arrange
    let meta = String::from_utf8(metadata("License: MIT"))
        .unwrap()
        .replace("2.4", version);
    let bytes = wheel(meta.as_bytes());
    let req = request(&bytes, ArtifactFormat::Wheel);
    // Act
    let evidence = Evidence::from_archive(&req, &bytes).unwrap();
    // Assert
    assert_eq!(evidence.summary().warnings.len(), warnings);
}

#[rstest::rstest]
#[case::invalid_historical_minor("1.3")]
#[case::invalid_historical_major("2.0")]
#[case::future_major("3.0")]
fn unsupported_metadata_version_is_rejected(#[case] version: &str) {
    // Arrange
    let meta = String::from_utf8(metadata("License: MIT"))
        .unwrap()
        .replace("2.4", version);
    let bytes = wheel(meta.as_bytes());
    let req = request(&bytes, ArtifactFormat::Wheel);
    // Act
    let result = Evidence::from_archive(&req, &bytes);
    // Assert
    assert!(result.is_err());
}

#[test]
fn duplicate_metadata_name_is_rejected() {
    // Arrange
    let bytes = wheel(&metadata("Name: repeated\nLicense-Expression: MIT"));
    let req = request(&bytes, ArtifactFormat::Wheel);
    // Act
    let result = Evidence::from_archive(&req, &bytes);
    // Assert
    assert!(result.is_err());
}

#[test]
fn source_distributions_do_not_claim_future_wheel_metadata() {
    // Arrange
    let bytes = sdist(&metadata(
        "License-Expression: MIT\nDynamic: License-Expression",
    ));
    let req = request(&bytes, ArtifactFormat::Sdist);
    // Act
    let evidence = Evidence::from_archive(&req, &bytes).unwrap();
    // Assert
    assert_eq!(evidence.license().unwrap().text(), "MIT");
}

#[test]
fn multiple_wheel_metadata_members_are_rejected() {
    // Arrange
    let bytes = common::wheel_with_two_metadata(&metadata("License-Expression: MIT"));
    let req = request(&bytes, ArtifactFormat::Wheel);
    // Act
    let result = Evidence::from_archive(&req, &bytes);
    // Assert
    assert!(result.is_err());
}

#[test]
fn cache_record_corruption_cannot_create_evidence() {
    // Arrange
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let req = request(&bytes, ArtifactFormat::Wheel);
    let evidence = Evidence::from_archive(&req, &bytes).unwrap();
    let mut encoded = serde_json::to_vec(&evidence.to_record()).unwrap();
    let at = encoded.windows(3).position(|s| s == b"MIT").unwrap();
    encoded[at..at + 3].copy_from_slice(b"ISC");
    let record: EvidenceRecord = serde_json::from_slice(&encoded).unwrap();
    // Act
    let result = Evidence::from_record(&req, record);
    // Assert
    assert!(result.is_err());
}

#[test]
fn cache_record_wrong_identity_cannot_create_evidence() {
    // Arrange
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let req = request(&bytes, ArtifactFormat::Wheel);
    let evidence = Evidence::from_archive(&req, &bytes).unwrap();
    let wrong = request(b"changed", ArtifactFormat::Wheel);
    // Act
    let result = Evidence::from_record(&wrong, evidence.to_record());
    // Assert
    assert!(result.is_err());
}

#[test]
fn cached_declarations_do_not_retain_description_bodies() {
    // Arrange
    let small = metadata("License-Expression: MIT");
    let mut large = small.clone();
    large.extend_from_slice("description".repeat(10_000).as_bytes());
    let small_archive = wheel(&small);
    let large_archive = wheel(&large);
    // Act
    let small = Evidence::from_archive(
        &request(&small_archive, ArtifactFormat::Wheel),
        &small_archive,
    )
    .unwrap();
    let large_evidence = Evidence::from_archive(
        &request(&large_archive, ArtifactFormat::Wheel),
        &large_archive,
    )
    .unwrap();
    // Assert
    assert_eq!(
        serde_json::to_vec(&small.to_record()).unwrap().len(),
        serde_json::to_vec(&large_evidence.to_record())
            .unwrap()
            .len()
    );
    assert_eq!(
        serde_json::to_value(large_evidence.summary()).unwrap()["metadata_hash"],
        ContentHash::digest(&large).to_string()
    );
}

#[test]
fn duplicate_zip_names_are_rejected_even_when_the_reader_deduplicates_them() {
    // Arrange
    let mut bytes = common::wheel_with_two_metadata(&metadata("License-Expression: MIT"));
    let old = b"another-1.0.dist-info/METADATA";
    let new = b"example-1.0.dist-info/METADATA";
    assert_eq!(old.len(), new.len());
    let positions = bytes
        .windows(old.len())
        .enumerate()
        .filter_map(|(i, s)| (s == old).then_some(i))
        .collect::<Vec<_>>();
    for i in positions {
        bytes[i..i + old.len()].copy_from_slice(new);
    }
    // Act
    let result = Evidence::from_archive(&request(&bytes, ArtifactFormat::Wheel), &bytes);
    // Assert
    assert!(result.is_err());
}

#[test]
fn cached_sidecars_cannot_change_registry_associations() {
    // Arrange
    let meta = metadata("License-Expression: MIT");
    let bytes = wheel(&meta);
    let req = request(&bytes, ArtifactFormat::Wheel);
    let hash = ContentHash::digest(&bytes);
    let e = Evidence::from_sidecar(
        &req,
        SidecarAssociation::new(hash, hash, ContentHash::digest(&meta)).unwrap(),
        &meta,
    )
    .unwrap();
    let other = taloon::model::ArtifactRequest::new(
        taloon::model::PackageId::parse(
            "example",
            "1.0",
            taloon::model::SourceId::parse("registry", "https://other.test/simple").unwrap(),
        )
        .unwrap(),
        taloon::model::ArtifactLocator::registry(
            "https://other.test/simple",
            "https://files.example.test/example.whl",
        )
        .unwrap(),
        ArtifactFormat::Wheel,
        Some(hash),
    )
    .unwrap();
    // Act
    let result = Evidence::from_record(&other, e.to_record());
    // Assert
    assert!(result.is_err());
}

#[rstest::rstest]
#[case::version("Version: 2.0\nLicense-Expression: MIT")]
#[case::name("Name: second\nLicense-Expression: MIT")]
fn metadata_identity_failure_cannot_be_license_override(#[case] fields: &str) {
    // Arrange
    let bytes = wheel(&metadata(fields));
    let req = request(&bytes, ArtifactFormat::Wheel);
    // Act
    let result = Evidence::from_archive(&req, &bytes);
    // Assert
    assert!(result.is_err());
}

#[test]
fn metadata_encoding_failure_cannot_be_license_override() {
    // Arrange
    let mut meta = metadata("License-Expression: MIT");
    meta[0] = 255;
    let bytes = wheel(&meta);
    let req = request(&bytes, ArtifactFormat::Wheel);
    // Act
    let result = Evidence::from_archive(&req, &bytes);
    // Assert
    assert!(result.is_err());
}

fn generated_license_expression() -> impl Strategy<Value = String> {
    let leaf = prop::sample::select(vec!["MIT", "Apache-2.0", "BSD-3-Clause", "ISC"]);
    prop_oneof![
        leaf.clone().prop_map(str::to_owned),
        (leaf.clone(), prop::sample::select(vec!["AND", "OR"]), leaf)
            .prop_map(|(left, operator, right)| format!("{left} {operator} {right}")),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig{cases:256,failure_persistence:Some(Box::new(proptest::test_runner::FileFailurePersistence::Direct("tests/evidence.proptest-regressions"))),..ProptestConfig::default()})]
    #[test]
    fn changing_archive_bytes_rejects_the_locked_digest(position in any::<usize>(),bit in 0_u8..8){
        // Arrange
        let bytes = wheel(&metadata("License-Expression: MIT"));
        let req = request(&bytes, ArtifactFormat::Wheel);
        let mut changed = bytes.clone();
        let length = changed.len();
        changed[position % length] ^= 1 << bit;
        // Act
        let result = Evidence::from_archive(&req, &changed);
        // Assert
        prop_assert_eq!(result.unwrap_err().code, Code::HashMismatch);
    }
    #[test]
    fn archive_cache_roundtrip_preserves_declaration_and_strength(
        license in generated_license_expression()
    ) {
        // Arrange
        let bytes = wheel(&metadata(&format!("License-Expression: {license}")));
        let req = request(&bytes, ArtifactFormat::Wheel);
        let evidence = Evidence::from_archive(&req, &bytes).unwrap();
        // Act
        let restored = Evidence::from_record(&req, evidence.to_record()).unwrap();
        // Assert
        prop_assert_eq!(restored.license().unwrap().text().to_owned(), license);
        prop_assert!(restored.requirement_satisfied(EvidenceRequirement::ArchiveVerified));
    }

    #[test]
    fn sidecar_cache_roundtrip_preserves_declaration_and_strength(
        license in generated_license_expression()
    ) {
        // Arrange
        let meta = metadata(&format!("License-Expression: {license}"));
        let bytes = wheel(&meta);
        let req = request(&bytes, ArtifactFormat::Wheel);
        let hash = ContentHash::digest(&bytes);
        let association = SidecarAssociation::new(hash, hash, ContentHash::digest(&meta)).unwrap();
        let evidence = Evidence::from_sidecar(&req, association, &meta).unwrap();
        // Act
        let restored = Evidence::from_record(&req, evidence.to_record()).unwrap();
        // Assert
        prop_assert_eq!(restored.license().unwrap().text().to_owned(), license);
        prop_assert!(restored.requirement_satisfied(EvidenceRequirement::IndexAssociated));
        prop_assert!(!restored.requirement_satisfied(EvidenceRequirement::ArchiveVerified));
    }

    #[test]
    fn generated_foreign_metadata_names_are_rejected(suffix in "[a-z]{1,24}") {
        // Arrange: prefix guarantees a different identity without filtering generated inputs.
        let meta = String::from_utf8(metadata("License-Expression: MIT")).unwrap()
            .replace("Name: example", &format!("Name: other-{suffix}"));
        let bytes = wheel(meta.as_bytes());
        let req = request(&bytes, ArtifactFormat::Wheel);
        // Act
        let result = Evidence::from_archive(&req, &bytes);
        // Assert
        prop_assert_eq!(result.unwrap_err().code, Code::IdentityMismatch);
    }

}

#[test]
fn archive_records_cannot_change_distribution_format() {
    // Arrange
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let req = request(&bytes, ArtifactFormat::Wheel);
    let evidence = Evidence::from_archive(&req, &bytes).unwrap();
    let wrong = request(&bytes, ArtifactFormat::Sdist);
    // Act
    let result = Evidence::from_record(&wrong, evidence.to_record());
    // Assert
    assert!(result.is_err());
}

#[test]
fn cached_sidecars_require_the_original_artifact_association() {
    // Arrange
    let meta = metadata("License-Expression: MIT");
    let bytes = wheel(&meta);
    let first = request(&bytes, ArtifactFormat::Wheel);
    let hash = ContentHash::digest(&bytes);
    let evidence = Evidence::from_sidecar(
        &first,
        SidecarAssociation::new(hash, hash, ContentHash::digest(&meta)).unwrap(),
        &meta,
    )
    .unwrap();
    let second = taloon::model::ArtifactRequest::new(
        first.package().clone(),
        taloon::model::ArtifactLocator::registry(
            "https://pypi.org/simple",
            "https://files.example.test/another.whl",
        )
        .unwrap(),
        first.format(),
        Some(hash),
    )
    .unwrap();
    // Act
    let result = Evidence::from_record(&second, evidence.to_record());
    // Assert
    assert!(result.is_err());
}

#[test]
fn wheel_metadata_must_be_a_regular_member() {
    // Arrange
    let mut bytes = wheel(&metadata("License-Expression: MIT"));
    let directory = bytes.windows(4).position(|s| s == b"PK\x01\x02").unwrap();
    bytes[directory + 5] = 3;
    bytes[directory + 38..directory + 42].copy_from_slice(&(0o010_644_u32 << 16).to_le_bytes());
    // Act
    let result = Evidence::from_archive(&request(&bytes, ArtifactFormat::Wheel), &bytes);
    // Assert
    assert!(result.is_err());
}

#[rstest::rstest]
#[case::email("Copyright authors <maintainer@example.org>")]
#[case::question("Redistribution permitted? See the conditions below.")]
#[case::url("See https://example.org/license for the full license.")]
#[case::folded("Copyright authors\n Contact: maintainer@example.org")]
fn declaration_prose_is_not_a_locator(#[case] declaration: &str) {
    // Arrange
    let bytes = wheel(&metadata(&format!("License: {declaration}")));
    let evidence = Evidence::from_archive(&request(&bytes, ArtifactFormat::Wheel), &bytes).unwrap();
    let expected = declaration.replace("\n ", " ");
    // Act
    let original = evidence.summary().original_license;
    // Assert
    assert_eq!(original, vec![expected]);
}

#[test]
fn archive_cache_roundtrip_retains_invalid_declaration() {
    // Arrange
    let bytes = wheel(&metadata("License-Expression: WRONG"));
    let req = request(&bytes, ArtifactFormat::Wheel);
    let evidence = Evidence::from_archive(&req, &bytes).unwrap();
    // Act
    let restored = Evidence::from_record(&req, evidence.to_record()).unwrap();
    // Assert
    assert!(restored.license().is_err());
    assert_eq!(
        restored.license().unwrap_err(),
        evidence.license().unwrap_err()
    );
    assert!(restored.requirement_satisfied(EvidenceRequirement::ArchiveVerified));
}

#[test]
fn sidecar_cache_roundtrip_retains_invalid_declaration() {
    // Arrange
    let meta = metadata("License-Expression: WRONG");
    let bytes = wheel(&meta);
    let req = request(&bytes, ArtifactFormat::Wheel);
    let hash = ContentHash::digest(&bytes);
    let association = SidecarAssociation::new(hash, hash, ContentHash::digest(&meta)).unwrap();
    let evidence = Evidence::from_sidecar(&req, association, &meta).unwrap();
    // Act
    let restored = Evidence::from_record(&req, evidence.to_record()).unwrap();
    // Assert
    assert!(restored.license().is_err());
    assert_eq!(
        restored.license().unwrap_err(),
        evidence.license().unwrap_err()
    );
    assert!(restored.requirement_satisfied(EvidenceRequirement::IndexAssociated));
    assert!(!restored.requirement_satisfied(EvidenceRequirement::ArchiveVerified));
}
