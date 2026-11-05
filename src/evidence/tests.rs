use super::*;
use proptest::prelude::*;
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
fn obsolete_observation_schema_is_rejected() {
    // Arrange: a correctly checksummed record with the obsolete payload version.
    let (_, bytes, request) = fixture();
    let mut record = Evidence::from_archive(&request, &bytes)
        .unwrap()
        .to_record();
    let data = Arc::make_mut(&mut record.data);
    data.schema = 1;
    record.checksum = checksum(data).unwrap();
    let encoded = serde_json::to_vec(&record).unwrap();
    let decoded = serde_json::from_slice(&encoded).unwrap();
    // Act
    let restored = Evidence::from_record(&request, decoded);
    // Assert
    assert_eq!(restored.unwrap_err().code, Code::Cache);
}

#[test]
fn omitted_file_state_is_rejected() {
    // Arrange
    let (_, bytes, request) = fixture();
    let record = Evidence::from_archive(&request, &bytes)
        .unwrap()
        .to_record();
    let mut value = serde_json::to_value(record).unwrap();
    value["data"]
        .as_object_mut()
        .unwrap()
        .remove("license_files");
    // Act
    let restored = serde_json::from_value::<EvidenceRecord>(value);
    // Assert
    assert!(restored.is_err());
}

#[test]
fn null_file_state_is_rejected() {
    // Arrange
    let (_, bytes, request) = fixture();
    let record = Evidence::from_archive(&request, &bytes)
        .unwrap()
        .to_record();
    let mut value = serde_json::to_value(record).unwrap();
    value["data"]["license_files"] = serde_json::Value::Null;
    // Act
    let restored = serde_json::from_value::<EvidenceRecord>(value);
    // Assert
    assert!(restored.is_err());
}

#[test]
fn sidecar_file_state_is_explicitly_not_read() {
    // Arrange
    let (metadata, bytes, request) = fixture();
    let hash = ContentHash::digest(&bytes);
    let sidecar = Evidence::from_sidecar(
        &request,
        SidecarAssociation::new(hash, hash, ContentHash::digest(&metadata)).unwrap(),
        &metadata,
    )
    .unwrap();
    // Act
    let value = serde_json::to_value(sidecar.to_record()).unwrap();
    // Assert
    assert_eq!(value["data"]["license_files"]["status"], "not_read");
}

proptest! {
    #[test]
    fn unsupported_payload_versions_never_restore(
        schema in any::<u32>().prop_filter("not the current schema", |schema| *schema != RECORD_SCHEMA),
    ) {
        // Arrange
        let (_, bytes, request) = fixture();
        let mut record = Evidence::from_archive(&request, &bytes).unwrap().to_record();
        Arc::make_mut(&mut record.data).schema = schema;
        record.checksum = checksum(&record.data).unwrap();
        // Act
        let restored = Evidence::from_record(&request, record);
        // Assert
        prop_assert_eq!(restored.unwrap_err().code, Code::Cache);
    }
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
    data.license_files = FileObservation::Complete { files: Vec::new() };
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
    data.license_files = FileObservation::Complete { files };
    record.checksum = checksum(data).unwrap();
    // Act
    let error = Evidence::from_record(&request, record).unwrap_err();
    // Assert
    assert_eq!(error.code, Code::Cache);
}
