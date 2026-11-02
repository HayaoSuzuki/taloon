mod common;
use common::{metadata, request};
use rstest::rstest;
use std::io::{Cursor, Write};
use taloon::{
    evidence::Evidence,
    model::{ArtifactFormat, Code},
    policy::LicenseExpression,
};

fn archive(format: ArtifactFormat, fields: &str, files: &[(&str, &str)]) -> Vec<u8> {
    let metadata = metadata(fields);
    match format {
        ArtifactFormat::Wheel => {
            let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
            writer
                .start_file(
                    "example-1.0.dist-info/METADATA",
                    zip::write::SimpleFileOptions::default(),
                )
                .unwrap();
            writer.write_all(&metadata).unwrap();
            for (name, text) in files {
                writer
                    .start_file(
                        format!("example-1.0.dist-info/licenses/{name}"),
                        zip::write::SimpleFileOptions::default(),
                    )
                    .unwrap();
                writer.write_all(text.as_bytes()).unwrap();
            }
            writer.finish().unwrap().into_inner()
        }
        ArtifactFormat::Sdist => {
            let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
            let mut writer = tar::Builder::new(encoder);
            let entries = std::iter::once(("PKG-INFO", metadata.as_slice()))
                .chain(files.iter().map(|(name, text)| (*name, text.as_bytes())));
            for (name, bytes) in entries {
                let mut header = tar::Header::new_gnu();
                header.set_mode(0o644);
                header.set_size(u64::try_from(bytes.len()).unwrap());
                header.set_cksum();
                writer
                    .append_data(
                        &mut header,
                        format!("example-1.0/{name}"),
                        Cursor::new(bytes),
                    )
                    .unwrap();
            }
            writer.into_inner().unwrap().finish().unwrap()
        }
    }
}
fn mit() -> &'static str {
    spdx::license_id("MIT").unwrap().text()
}

#[rstest]
#[case::wheel(ArtifactFormat::Wheel)]
#[case::sdist(ArtifactFormat::Sdist)]
fn missing_declaration_uses_verified_license_file(#[case] format: ArtifactFormat) {
    // Arrange
    let bytes = archive(format, "License-File: LICENSE", &[("LICENSE", mit())]);
    let request = request(&bytes, format);
    // Act
    let evidence = Evidence::from_archive(&request, &bytes).unwrap();
    // Assert
    assert_eq!(
        evidence.license().map(|e| e.text().to_owned()).unwrap(),
        "MIT"
    );
}

#[rstest]
#[case::expression("License-Expression: Apache-2.0", Some("Apache-2.0"))]
#[case::invalid_expression("License-Expression: invalid", None)]
#[case::legacy("License: Apache-2.0", Some("Apache-2.0"))]
#[case::invalid_legacy("License: custom terms", None)]
#[case::repeated("License-Expression: MIT\nLicense-Expression: MIT", None)]
fn explicit_declaration_prevents_file_fallback(
    #[case] declaration: &str,
    #[case] expected: Option<&str>,
) {
    // Arrange
    let bytes = archive(
        ArtifactFormat::Wheel,
        &format!("{declaration}\nLicense-File: LICENSE"),
        &[("LICENSE", mit())],
    );
    let request = request(&bytes, ArtifactFormat::Wheel);
    // Act
    let evidence = Evidence::from_archive(&request, &bytes).unwrap();
    // Assert
    assert_eq!(
        evidence
            .license()
            .ok()
            .as_ref()
            .map(LicenseExpression::text),
        expected
    );
}

#[test]
fn attribution_file_does_not_hide_the_license_file() {
    // Arrange
    let bytes = archive(
        ArtifactFormat::Wheel,
        "License-File: AUTHORS\nLicense-File: LICENSE",
        &[("AUTHORS", "Example contributors"), ("LICENSE", mit())],
    );
    let request = request(&bytes, ArtifactFormat::Wheel);
    // Act
    let evidence = Evidence::from_archive(&request, &bytes).unwrap();
    // Assert
    assert_eq!(evidence.license().unwrap().text(), "MIT");
}

#[test]
fn unrecognized_additional_declared_file_blocks_inference() {
    // Arrange
    let bytes = archive(
        ArtifactFormat::Wheel,
        "License-File: LICENSE\nLicense-File: TERMS",
        &[("LICENSE", mit()), ("TERMS", "Non-commercial use only")],
    );
    let request = request(&bytes, ArtifactFormat::Wheel);
    // Act
    let evidence = Evidence::from_archive(&request, &bytes).unwrap();
    // Assert
    assert_eq!(evidence.license().unwrap_err().code, Code::UnknownLicense);
}

#[test]
fn observed_license_files_survive_record_roundtrip() {
    // Arrange
    let bytes = archive(
        ArtifactFormat::Wheel,
        "License-File: LICENSE",
        &[("LICENSE", mit())],
    );
    let request = request(&bytes, ArtifactFormat::Wheel);
    let record = Evidence::from_archive(&request, &bytes)
        .unwrap()
        .to_record();
    // Act
    let restored = Evidence::from_record(&request, record).unwrap();
    // Assert
    assert_eq!(restored.license().unwrap().text(), "MIT");
}

#[test]
fn inferred_summary_reports_file_hash_and_keeps_original_declarations_absent() {
    // Arrange
    let bytes = archive(
        ArtifactFormat::Wheel,
        "License-File: LICENSE",
        &[("LICENSE", mit())],
    );
    let request = request(&bytes, ArtifactFormat::Wheel);
    let evidence = Evidence::from_archive(&request, &bytes).unwrap();
    // Act
    let summary = evidence.summary();
    // Assert
    assert_eq!(summary.original_expression, Vec::<String>::new());
    assert_eq!(summary.original_license, Vec::<String>::new());
    assert_eq!(summary.license_files.len(), 1);
    assert_eq!(summary.license_files[0].path, "LICENSE");
    assert_eq!(summary.license_files[0].matched_expression, Some("MIT"));
    assert_eq!(
        summary.license_files[0].content_hash,
        taloon::model::ContentHash::digest(mit().as_bytes())
    );
}

#[rstest]
#[case::same_license("MIT", Some("MIT"))]
#[case::different_license("ISC", None)]
fn multiple_files_require_one_recognized_expression(
    #[case] second: &str,
    #[case] expected: Option<&str>,
) {
    // Arrange
    let other = spdx::license_id(second).unwrap().text();
    let bytes = archive(
        ArtifactFormat::Wheel,
        "License-File: LICENSE\nLicense-File: EXTRA",
        &[("LICENSE", mit()), ("EXTRA", other)],
    );
    let request = request(&bytes, ArtifactFormat::Wheel);
    // Act
    let evidence = Evidence::from_archive(&request, &bytes).unwrap();
    // Assert
    assert_eq!(
        evidence
            .license()
            .ok()
            .as_ref()
            .map(LicenseExpression::text),
        expected
    );
}

#[rstest]
#[case::at_limit(4096, Some("MIT"))]
#[case::over_limit(4097, None)]
fn long_selected_paths_preserve_same_request_rebinding(
    #[case] length: usize,
    #[case] expected: Option<&str>,
) {
    // Arrange
    let path = "a".repeat(length);
    let bytes = archive(
        ArtifactFormat::Wheel,
        &format!("License-File: {path}"),
        &[(&path, mit())],
    );
    let request = request(&bytes, ArtifactFormat::Wheel);
    let evidence = Evidence::from_archive(&request, &bytes).unwrap();
    // Act
    let rebound = evidence.rebind(&request).unwrap();
    // Assert
    assert_eq!(
        rebound.license().ok().as_ref().map(LicenseExpression::text),
        expected
    );
}
