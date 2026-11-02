mod common;
use common::{metadata, request, wheel, wheel_with_files};
use rstest::rstest;
use std::fmt::Write as _;
use taloon::{
    evidence::{Evidence, SidecarAssociation},
    model::{ArtifactFormat, ContentHash},
};

#[rstest]
#[case::absent("", "missing_declaration")]
#[case::duplicate_legacy("License: MIT\nLicense: MIT", "repeated_declaration")]
#[case::duplicate_expression(
    "License-Expression: MIT\nLicense-Expression: ISC",
    "repeated_declaration"
)]
#[case::invalid_expression("License-Expression: bad\nLicense: MIT", "invalid_expression")]
#[case::unsupported_expression(
    "License-Expression: DocumentRef-other:LicenseRef-custom",
    "invalid_expression"
)]
#[case::legacy("License: custom terms", "unrecognized_legacy")]
fn sidecar_failure_exposes_its_interpretation_cause(#[case] fields: &str, #[case] reason: &str) {
    // Arrange
    let bytes = metadata(fields);
    let request = request(b"archive", ArtifactFormat::Wheel);
    let hash = request.hash().unwrap();
    let evidence = Evidence::from_sidecar(
        &request,
        SidecarAssociation::new(hash, hash, ContentHash::digest(&bytes)).unwrap(),
        &bytes,
    )
    .unwrap();
    // Act
    let error = serde_json::to_value(evidence.license().unwrap_err()).unwrap();
    // Assert
    assert_eq!(error["code"], "unknown_license");
    assert_eq!(error["license_problem"], reason);
}

#[rstest]
#[case::unavailable("", &[], "license_files_unavailable")]
#[case::unrecognized("", &["custom terms"], "unrecognized_license_text")]
#[case::conflicting("", &["MIT", "ISC"], "conflicting_license_files")]
#[case::family("License: BSD", &["MIT"], "legacy_family_mismatch")]
fn archive_failure_cause_survives_cache_replay(
    #[case] fields: &str,
    #[case] texts: &[&str],
    #[case] reason: &str,
) {
    // Arrange
    let entries: Vec<_> = texts
        .iter()
        .enumerate()
        .map(|(i, text)| {
            (
                format!("example-1.0.dist-info/licenses/LICENSE{i}"),
                spdx::license_id(text).map_or(*text, |id| id.text()),
            )
        })
        .collect();
    let mut declarations = String::new();
    for (name, _) in &entries {
        write!(
            declarations,
            "\nLicense-File: {}",
            name.rsplit('/').next().unwrap()
        )
        .unwrap();
    }
    let files: Vec<_> = entries
        .iter()
        .map(|(name, text)| (name.as_str(), text.as_bytes()))
        .collect();
    let bytes = wheel_with_files(
        &metadata(format!("{fields}{declarations}").trim_start_matches('\n')),
        &files,
    );
    let request = request(&bytes, ArtifactFormat::Wheel);
    let record = Evidence::from_archive(&request, &bytes)
        .unwrap()
        .to_record();
    // Act
    let restored = Evidence::from_record(&request, record).unwrap();
    let error = serde_json::to_value(restored.license().unwrap_err()).unwrap();
    // Assert
    assert_eq!(error["code"], "unknown_license");
    assert_eq!(error["license_problem"], reason);
}

proptest::proptest! {
    #[test]
    fn repeated_values_are_not_misdiagnosed_as_expression_syntax(count in 2_usize..12, value in "[A-Z]{1,20}") {
        // Arrange
        let fields = std::iter::repeat_n(format!("License-Expression: {value}"),count).collect::<Vec<_>>().join("\n");
        let bytes = wheel(&metadata(&fields));
        let request = request(&bytes,ArtifactFormat::Wheel);
        let evidence = Evidence::from_archive(&request,&bytes).unwrap();
        // Act
        let error = serde_json::to_value(evidence.license().unwrap_err()).unwrap();
        // Assert
        proptest::prop_assert_eq!(&error["license_problem"], "repeated_declaration");
    }
}
