use super::*;
use rstest::rstest;
fn fixture() -> (Vec<u8>, Vec<u8>, ArtifactRequest) {
    use std::io::Write;
    let metadata =
        b"Metadata-Version: 2.4\nName: example\nVersion: 1.0\nLicense-File: LICENSE\n\n".to_vec();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .start_file(
            "example-1.0.dist-info/METADATA",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
    writer.write_all(&metadata).unwrap();
    let bytes = writer.finish().unwrap().into_inner();
    let source = crate::model::SourceId::parse("registry", "https://example.test/simple").unwrap();
    let request = ArtifactRequest::new(
        PackageId::parse("example", "1.0", source).unwrap(),
        ArtifactLocator::registry(
            "https://example.test/simple",
            "https://files.test/example.whl",
        )
        .unwrap(),
        ArtifactFormat::Wheel,
        Some(ContentHash::digest(&bytes)),
    )
    .unwrap();
    (metadata, bytes, request)
}

#[test]
fn old_archive_observations_need_one_file_upgrade() {
    // Arrange: reproduce the old schema's actual field omission and checksum.
    let (_, bytes, request) = fixture();
    let mut record = Evidence::from_archive(&request, &bytes)
        .unwrap()
        .to_record();
    let data = Arc::make_mut(&mut record.data);
    data.license_files = None;
    record.checksum = checksum(data).unwrap();
    let encoded = serde_json::to_vec(&record).unwrap();
    let decoded = serde_json::from_slice(&encoded).unwrap();
    // Act
    let restored = Evidence::from_record(&request, decoded).unwrap();
    // Assert
    assert!(restored.needs_license_files());
    assert!(restored.license().is_err());
    assert!(
        !String::from_utf8(encoded)
            .unwrap()
            .contains("license_files")
    );
}

#[test]
fn sidecar_records_cannot_claim_observed_license_files() {
    // Arrange
    let (metadata, _, request) = fixture();
    let hash = request.hash().unwrap();
    let mut record = Evidence::from_sidecar(
        &request,
        SidecarAssociation::new(hash, hash, ContentHash::digest(&metadata)).unwrap(),
        &metadata,
    )
    .unwrap()
    .to_record();
    let data = Arc::make_mut(&mut record.data);
    data.license_files = Some(FileObservation::Complete { files: Vec::new() });
    record.checksum = checksum(data).unwrap();
    // Act
    let error = Evidence::from_record(&request, record).unwrap_err();
    // Assert
    assert_eq!(error.code, Code::Cache);
}

#[rstest]
#[case::file_size(1,archive_files::MAX_LICENSE_BYTES+1)]
#[case::file_count(archive_files::MAX_LICENSE_FILES+1,0)]
#[case::total_size(5, archive_files::MAX_LICENSE_BYTES)]
fn oversized_file_observations_are_rejected_on_restore(#[case] count: usize, #[case] size: usize) {
    // Arrange
    let (_, bytes, request) = fixture();
    let mut record = Evidence::from_archive(&request, &bytes)
        .unwrap()
        .to_record();
    let files = (0..count)
        .map(|n| archive_files::LicenseFile {
            path: format!("LICENSE-{n}"),
            text: "a".repeat(size),
        })
        .collect();
    let data = Arc::make_mut(&mut record.data);
    data.license_files = Some(FileObservation::Complete { files });
    record.checksum = checksum(data).unwrap();
    // Act
    let error = Evidence::from_record(&request, record).unwrap_err();
    // Assert
    assert_eq!(error.code, Code::Cache);
}
