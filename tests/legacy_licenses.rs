mod common;
use common::{metadata, request, wheel};
use proptest::prelude::*;
use rstest::rstest;
use taloon::{
    evidence::{Evidence, SidecarAssociation},
    model::{ArtifactFormat, Code, ContentHash, EvidenceRequirement},
};
fn legacy(value: &str) -> Evidence {
    let meta = metadata(&format!("License: {}", value.replace('\n', "\n ")));
    let bytes = wheel(&meta);
    let request = request(&bytes, ArtifactFormat::Wheel);
    let hash = request.hash().unwrap();
    Evidence::from_sidecar(
        &request,
        SidecarAssociation::new(hash, hash, ContentHash::digest(&meta)).unwrap(),
        &meta,
    )
    .unwrap()
}
#[rstest]
#[case::mit_title("MIT License", "MIT")]
#[case::apache_title("Apache License 2.0", "Apache-2.0")]
#[case::apache_punctuation("Apache License, Version 2.0", "Apache-2.0")]
#[case::apache_short("Apache 2.0", "Apache-2.0")]
#[case::bsd2("BSD 2-Clause License", "BSD-2-Clause")]
#[case::bsd3("BSD 3-Clause License", "BSD-3-Clause")]
#[case::bsd2_reverse("2-Clause BSD License", "BSD-2-Clause")]
#[case::bsd3_reverse("3-Clause BSD License", "BSD-3-Clause")]
#[case::mit_wrapped("The MIT License (MIT)", "MIT")]
#[case::case_insensitive("mit license", "MIT")]
fn precise_legacy_names_are_interpreted_without_archive_proof(
    #[case] value: &str,
    #[case] expected: &str,
) {
    // Arrange
    let evidence = legacy(value);
    // Act
    let expression = evidence.license().unwrap();
    // Assert
    assert_eq!(expression.text(), expected);
    assert!(evidence.requirement_satisfied(EvidenceRequirement::IndexAssociated));
    assert!(!evidence.requirement_satisfied(EvidenceRequirement::ArchiveVerified));
    assert!(!evidence.needs_license_files());
}
#[rstest]
#[case::bsd("BSD")]
#[case::bsd_generic("BSD License")]
#[case::apache_unspecified("Apache Software License")]
#[case::added_terms("MIT License, non-commercial use only")]
#[case::missing_conjunction("MIT License Apache License 2.0")]
#[case::unknown("UNKNOWN")]
#[case::locator("https://example.test/MIT-License")]
fn ambiguous_or_modified_names_remain_unknown(#[case] value: &str) {
    // Arrange
    let evidence = legacy(value);
    // Act
    let error = evidence.license().unwrap_err();
    // Assert
    assert_eq!(error.code, Code::UnknownLicense);
}
proptest! {
    #[test]
    fn extra_words_cannot_be_discarded_as_name_decoration(extra in "[a-z]{1,24}") {
        // Arrange
        let evidence=legacy(&format!("MIT License {extra}"));
        // Act
        let result=evidence.license();
        // Assert
        prop_assert!(result.is_err());
    }
    #[test]
    fn official_name_case_does_not_change_identity(mask in any::<u32>()) {
        // Arrange
        let name: String="MIT License".chars().enumerate().map(|(i,c)|if mask & (1 << i)==0 {c.to_ascii_lowercase()}else{c.to_ascii_uppercase()}).collect();
        let evidence=legacy(&name);
        // Act
        let result=evidence.license().unwrap();
        // Assert
        prop_assert_eq!(result.text(),"MIT");
    }
}
#[rstest]
#[case::or("MIT OR Apache-2.0")]
#[case::and("BSD-2-Clause AND MIT")]
#[case::custom("LicenseRef-Reviewed")]
fn valid_legacy_expressions_keep_their_semantics(#[case] text: &str) {
    // Arrange
    let evidence = legacy(text);
    // Act
    let expression = evidence.license().unwrap();
    // Assert
    assert_eq!(expression.text(), text);
}

