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

#[rstest]
#[case::orphan(" continuation\n", "orphan metadata continuation")]
#[case::missing_colon("broken\n", "malformed metadata header")]
#[case::empty_key(": value\n", "invalid metadata header characters")]
#[case::space_key("Bad Key: value\n", "invalid metadata header characters")]
#[case::embedded_cr("Bad: val\rue\n", "invalid metadata header characters")]
#[case::version_no_dot("Metadata-Version: 2\n", "invalid Metadata-Version")]
#[case::version_major("Metadata-Version: x.4\n", "invalid metadata major version")]
#[case::version_minor("Metadata-Version: 2.x\n", "invalid metadata minor version")]
#[case::version_unsupported("Metadata-Version: 3.0\n", "unsupported metadata major version")]
#[case::version_legacy("Metadata-Version: 1.3\n", "unrecognized historical metadata version")]
#[case::version_gap("Metadata-Version: 2.0\n", "unrecognized historical metadata version")]
fn malformed_headers_report_specific_cause(#[case] header: &str, #[case] message: &str) {
    // Arrange
    let (_, _, request) = fixture();
    let text = format!("{header}Name: example\nVersion: 1.0\n\n");
    // Act
    let error = parse_metadata(&request, &text).unwrap_err();
    // Assert
    assert_eq!(error.code, Code::InvalidMetadata);
    assert_eq!(error.message, message);
}

#[test]
fn invalid_metadata_package_identity_reports_invalid_metadata() {
    // Arrange
    let (_, _, request) = fixture();
    let text = "Metadata-Version: 2.4\nName: /invalid\nVersion: 1.0\n\n";
    // Act
    let error = parse_metadata(&request, text).unwrap_err();
    // Assert
    assert_eq!(error.code, Code::InvalidMetadata);
    assert_eq!(error.message, "invalid metadata package identity");
}

#[test]
fn future_minor_metadata_keeps_known_declaration_with_warning() {
    // Arrange
    let (_, _, request) = fixture();
    let text = "Metadata-Version: 2.7\nName: example\nVersion: 1.0\nLicense-Expression: MIT\n\n";
    // Act
    let metadata = parse_metadata(&request, text).unwrap();
    // Assert
    assert_eq!(metadata.expression, ["MIT"]);
    assert_eq!(metadata.warnings.len(), 1);
    assert_eq!(metadata.warnings[0].code, Code::MetadataVersion);
}

#[test]
fn unread_file_observation_cannot_supply_license() {
    // Arrange
    let observation = FileObservation::NotRead;
    // Act
    let error = observation.license().unwrap_err();
    // Assert
    assert_eq!(error.message, "bundled license files have not been read");
    assert!(observation.summary().is_empty());
}

#[test]
fn unavailable_file_observation_preserves_failure_reason() {
    // Arrange
    let observation = FileObservation::Unavailable {
        message: "archive truncated".into(),
    };
    // Act
    let error = observation.license().unwrap_err();
    // Assert
    assert_eq!(error.message, "archive truncated");
    assert!(observation.summary().is_empty());
}

#[test]
fn metadata_reader_propagates_read_failure() {
    // Arrange
    struct Broken;
    impl std::io::Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("fixture read failure"))
        }
    }
    // Act
    let error = read_metadata(Broken).unwrap_err();
    // Assert
    assert_eq!(error.code, Code::Archive);
    assert_eq!(error.message, "cannot read distribution metadata");
}

#[test]
fn metadata_reader_rejects_excess_size() {
    // Arrange
    let bytes = vec![b'a'; usize::try_from(MAX_METADATA_BYTES).unwrap() + 1];
    // Act
    let error = read_metadata(bytes.as_slice()).unwrap_err();
    // Assert
    assert_eq!(error.message, "metadata exceeds the 4 MiB reading limit");
}

#[test]
fn evidence_constructor_rejects_excess_metadata() {
    // Arrange
    let (_, _, request) = fixture();
    let bytes = vec![b'a'; usize::try_from(MAX_METADATA_BYTES).unwrap() + 1];
    // Act
    let error = Evidence::new(
        &request,
        bytes,
        ContentHash::digest(b"archive"),
        Proof::ArchiveVerified,
    )
    .unwrap_err();
    // Assert
    assert_eq!(error.code, Code::InvalidMetadata);
    assert_eq!(error.message, "metadata exceeds the 4 MiB reading limit");
}

#[test]
fn evidence_constructor_rejects_non_utf8_metadata() {
    // Arrange
    let (_, _, request) = fixture();
    // Act
    let error = Evidence::new(
        &request,
        vec![255],
        ContentHash::digest(b"archive"),
        Proof::ArchiveVerified,
    )
    .unwrap_err();
    // Assert
    assert_eq!(error.message, "metadata must be UTF-8");
}

#[rstest]
#[case::wheel(b"not a zip", ArtifactFormat::Wheel, "invalid wheel ZIP archive")]
#[case::tar(b"not gzip", ArtifactFormat::Sdist, "invalid tar member")]
fn corrupt_distribution_is_rejected(
    #[case] bytes: &[u8],
    #[case] format: ArtifactFormat,
    #[case] message: &str,
) {
    // Arrange / Act
    let error = match format {
        ArtifactFormat::Wheel => wheel_metadata(bytes).unwrap_err(),
        ArtifactFormat::Sdist => sdist_metadata(bytes).unwrap_err(),
    };
    // Assert
    assert_eq!(error.code, Code::Archive);
    assert_eq!(error.message, message);
}

#[test]
fn sidecar_size_bound_precedes_digest_interpretation() {
    // Arrange
    let (_, _, request) = fixture();
    let bytes = vec![b'a'; usize::try_from(MAX_METADATA_BYTES).unwrap() + 1];
    let hash = request.hash().unwrap();
    let binding = SidecarAssociation::new(hash, hash, ContentHash::digest(&bytes)).unwrap();
    // Act
    let error = Evidence::from_sidecar(&request, binding, &bytes).unwrap_err();
    // Assert
    assert_eq!(error.code, Code::InvalidMetadata);
    assert_eq!(error.message, "metadata exceeds the 4 MiB reading limit");
}

#[test]
fn cached_content_hash_must_match_new_locked_artifact() {
    // Arrange
    let (_, bytes, original) = fixture();
    let record = Evidence::from_archive(&original, &bytes)
        .unwrap()
        .to_record();
    let request = ArtifactRequest::new(
        original.package().clone(),
        original.locator().clone(),
        original.format(),
        Some(ContentHash::digest(b"different")),
    )
    .unwrap();
    // Act
    let error = Evidence::from_record(&request, record).unwrap_err();
    // Assert
    assert_eq!(error.code, Code::HashMismatch);
    assert_eq!(error.message, "cached content differs from locked artifact");
}

fn metadata_wheel(entries: &[(&str, &[u8], bool)]) -> Vec<u8> {
    use std::io::Write;
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for &(path, contents, symlink) in entries {
        let options = zip::write::SimpleFileOptions::default();
        if symlink {
            writer.add_symlink(path, "elsewhere", options).unwrap();
        } else {
            writer.start_file(path, options).unwrap();
            writer.write_all(contents).unwrap();
        }
    }
    writer.finish().unwrap().into_inner()
}

#[test]
fn wheel_without_metadata_reports_missing_member() {
    // Arrange
    let bytes = metadata_wheel(&[("unrelated", b"text", false)]);
    // Act
    let error = wheel_metadata(&bytes).unwrap_err();
    // Assert
    assert_eq!(error.message, "wheel has no unique dist-info/METADATA");
}

#[test]
fn wheel_with_two_metadata_roots_is_ambiguous() {
    // Arrange
    let bytes = metadata_wheel(&[
        ("first.dist-info/METADATA", b"text", false),
        ("second.dist-info/METADATA", b"text", false),
    ]);
    // Act
    let error = wheel_metadata(&bytes).unwrap_err();
    // Assert
    assert_eq!(error.message, "multiple wheel metadata members");
}

#[test]
fn wheel_metadata_cannot_be_a_symbolic_link() {
    // Arrange
    let bytes = metadata_wheel(&[("example.dist-info/METADATA", b"", true)]);
    // Act
    let error = wheel_metadata(&bytes).unwrap_err();
    // Assert
    assert_eq!(error.message, "metadata member is not a regular file");
}

fn metadata_sdist(entries: &[(&str, &[u8], bool)]) -> Vec<u8> {
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    let mut writer = tar::Builder::new(encoder);
    for &(path, contents, symlink) in entries {
        let mut header = tar::Header::new_gnu();
        header.set_size(if symlink { 0 } else { contents.len() as u64 });
        header.set_mode(0o644);
        if symlink {
            header.set_entry_type(tar::EntryType::Symlink);
            header.set_link_name("elsewhere").unwrap();
        }
        header.set_cksum();
        writer.append_data(&mut header, path, contents).unwrap();
    }
    writer.into_inner().unwrap().finish().unwrap()
}

#[test]
fn sdist_metadata_cannot_be_a_symbolic_link() {
    // Arrange
    let bytes = metadata_sdist(&[("example/PKG-INFO", b"", true)]);
    // Act
    let error = sdist_metadata(&bytes).unwrap_err();
    // Assert
    assert_eq!(error.message, "source metadata is not a regular file");
}

#[test]
fn sdist_with_two_metadata_roots_is_ambiguous() {
    // Arrange
    let bytes = metadata_sdist(&[
        ("first/PKG-INFO", b"text", false),
        ("second/PKG-INFO", b"text", false),
    ]);
    // Act
    let error = sdist_metadata(&bytes).unwrap_err();
    // Assert
    assert_eq!(error.message, "multiple source metadata members");
}

#[test]
fn sdist_without_metadata_reports_missing_member() {
    // Arrange
    let bytes = metadata_sdist(&[("example/unrelated", b"text", false)]);
    // Act
    let error = sdist_metadata(&bytes).unwrap_err();
    // Assert
    assert_eq!(error.message, "source archive lacks top-level PKG-INFO");
}

#[test]
fn sidecar_from_foreign_registry_cannot_bind_locked_package() {
    // Arrange
    let (_, _, original) = fixture();
    let request = ArtifactRequest::new(
        original.package().clone(),
        ArtifactLocator::registry(
            "https://foreign.test/simple",
            "https://files.test/example.whl",
        )
        .unwrap(),
        original.format(),
        original.hash(),
    )
    .unwrap();
    // Act
    let error = source_binding(&request).unwrap_err();
    // Assert
    assert_eq!(error.code, Code::InsufficientEvidence);
    assert_eq!(
        error.message,
        "sidecar lacks matching locked registry association"
    );
}

#[test]
fn cached_sidecar_cannot_substitute_request_proof() {
    // Arrange
    let (metadata, bytes, request) = fixture();
    let hash = ContentHash::digest(&bytes);
    let evidence = Evidence::from_sidecar(
        &request,
        SidecarAssociation::new(hash, hash, ContentHash::digest(&metadata)).unwrap(),
        &metadata,
    )
    .unwrap();
    let mut record = evidence.to_record();
    let data = Arc::make_mut(&mut record.data);
    data.proof = Proof::IndexAssociated {
        source: ContentHash::digest(b"wrong source"),
        request: request.id(),
    };
    record.checksum = checksum(data).unwrap();
    // Act
    let error = Evidence::from_record(&request, record).unwrap_err();
    // Assert
    assert_eq!(error.code, Code::IdentityMismatch);
    assert_eq!(error.message, "cached sidecar belongs to another source");
}

pub(super) fn expansion_limit_bytes() -> &'static [u8] {
    static BYTES: std::sync::LazyLock<Vec<u8>> = std::sync::LazyLock::new(|| {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        std::io::copy(
            &mut std::io::repeat(0).take(MAX_TAR_EXPANSION + 1),
            &mut encoder,
        )
        .unwrap();
        encoder.finish().unwrap()
    });
    &BYTES
}

#[test]
fn sdist_padding_cannot_bypass_expansion_limit() {
    // Arrange
    let bytes = expansion_limit_bytes();
    // Act
    let error = sdist_metadata(bytes).unwrap_err();
    // Assert
    assert_eq!(
        error.message,
        "source archive exceeds the 1 GiB expansion limit"
    );
}

#[test]
fn corrupt_gzip_trailer_after_tar_end_is_rejected() {
    // Arrange
    let mut bytes = metadata_sdist(&[("example/PKG-INFO", b"text", false)]);
    let checksum = bytes.len() - 8;
    bytes[checksum] ^= 1;
    // Act
    let error = sdist_metadata(&bytes).unwrap_err();
    // Assert
    assert_eq!(error.code, Code::Archive);
    assert_eq!(error.message, "corrupt gzip stream");
}

fn duplicate_name_wheel() -> Vec<u8> {
    let mut bytes = metadata_wheel(&[
        ("first.dist-info/METADATA", b"text", false),
        ("other.dist-info/METADATA", b"text", false),
    ]);
    let offsets: Vec<_> = bytes
        .windows(5)
        .enumerate()
        .filter_map(|(offset, value)| (value == b"other").then_some(offset))
        .collect();
    for offset in offsets {
        bytes[offset..offset + 5].copy_from_slice(b"first");
    }
    bytes
}

#[test]
fn duplicate_zip_names_are_rejected_before_metadata_selection() {
    // Arrange
    let bytes = duplicate_name_wheel();
    // Act
    let error = wheel_metadata(&bytes).unwrap_err();
    // Assert
    assert_eq!(error.message, "duplicate ZIP members");
}

#[test]
fn duplicate_zip_names_cannot_supply_license_files() {
    // Arrange
    let bytes = duplicate_name_wheel();
    // Act
    let error =
        archive_files::read(&bytes, ArtifactFormat::Wheel, &[], "first.dist-info/").unwrap_err();
    // Assert
    assert_eq!(error.code, Code::UnknownLicense);
}

#[test]
fn non_utf8_tar_metadata_name_is_rejected() {
    // Arrange
    let mut header = tar::Header::new_gnu();
    header.set_size(0);
    header.set_mode(0o644);
    header.as_mut_bytes()[0] = 255;
    header.set_cksum();
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    let mut writer = tar::Builder::new(encoder);
    writer.append(&header, std::io::empty()).unwrap();
    let bytes = writer.into_inner().unwrap().finish().unwrap();
    // Act
    let error = sdist_metadata(&bytes).unwrap_err();
    // Assert
    assert_eq!(error.message, "invalid tar member name");
}

#[test]
fn unsupported_zip_member_compression_is_rejected() {
    // Arrange
    let mut bytes = metadata_wheel(&[("example.dist-info/METADATA", b"text", false)]);
    let offsets: Vec<_> = bytes
        .windows(4)
        .enumerate()
        .filter_map(|(offset, value)| (value == b"PK\x01\x02").then_some(offset))
        .collect();
    for offset in offsets {
        bytes[offset + 10..offset + 12].copy_from_slice(&999_u16.to_le_bytes());
    }
    // Act
    let error = wheel_metadata(&bytes).unwrap_err();
    // Assert
    assert_eq!(error.message, "cannot read distribution metadata");
}

#[test]
fn attribution_file_summary_does_not_claim_license_expression() {
    // Arrange
    let observation = FileObservation::Complete {
        files: vec![archive_files::LicenseFile {
            path: "AUTHORS".into(),
            text: "Alice".into(),
        }],
    };
    // Act
    let summary = observation.summary();
    // Assert
    assert_eq!(summary.len(), 1);
    assert_eq!(summary[0].path, "AUTHORS");
    assert!(matches!(
        summary[0].kind,
        file_observation::LicenseFileKind::Attribution
    ));
    assert_eq!(summary[0].matched_expression, None);
    assert_eq!(summary[0].content_hash, ContentHash::digest(b"Alice"));
}

#[test]
fn direct_artifact_cannot_supply_registry_association() {
    // Arrange
    let (_, _, original) = fixture();
    let request = ArtifactRequest::new(
        original.package().clone(),
        ArtifactLocator::url("https://files.test/example.whl").unwrap(),
        original.format(),
        original.hash(),
    )
    .unwrap();
    // Act
    let error = source_binding(&request).unwrap_err();
    // Assert
    assert_eq!(error.code, Code::InsufficientEvidence);
}

#[test]
fn encrypted_zip_member_is_rejected_without_password() {
    // Arrange
    let mut bytes = metadata_wheel(&[("example.dist-info/METADATA", b"text", false)]);
    let offset = bytes
        .windows(4)
        .position(|value| value == b"PK\x01\x02")
        .unwrap();
    bytes[offset + 8] |= 1;
    // Act
    let error = wheel_metadata(&bytes).unwrap_err();
    // Assert
    assert_eq!(error.message, "invalid ZIP member");
}

#[test]
fn zip_member_name_uses_cp437_for_non_utf8_bytes() {
    // Arrange
    let mut bytes = metadata_wheel(&[("example.dist-info/METADATA", b"text", false)]);
    let central = bytes
        .windows(4)
        .position(|value| value == b"PK\x01\x02")
        .unwrap();
    bytes[central + 9] |= 8;
    bytes[central + 46] = 255;
    bytes[7] |= 8;
    bytes[30] = 255;
    // Act
    let (metadata, root) = wheel_metadata(&bytes).unwrap();
    // Assert
    assert_eq!(metadata, b"text");
    assert_eq!(root, "\u{a0}xample.dist-info/");
}

#[test]
fn encrypted_zip_member_cannot_supply_license_files() {
    // Arrange
    let mut bytes = metadata_wheel(&[("example.dist-info/METADATA", b"text", false)]);
    let offset = bytes
        .windows(4)
        .position(|value| value == b"PK\x01\x02")
        .unwrap();
    bytes[offset + 8] |= 1;
    // Act
    let error =
        archive_files::read(&bytes, ArtifactFormat::Wheel, &[], "example.dist-info/").unwrap_err();
    // Assert
    assert_eq!(error.code, Code::UnknownLicense);
}

#[test]
fn license_scan_rejects_corrupt_gzip_trailer_after_tar_end() {
    // Arrange
    let mut bytes = metadata_sdist(&[("example/PKG-INFO", b"text", false)]);
    let checksum = bytes.len() - 8;
    bytes[checksum] ^= 1;
    // Act
    let error = archive_files::read(&bytes, ArtifactFormat::Sdist, &[], "example/").unwrap_err();
    // Assert
    assert_eq!(error.code, Code::UnknownLicense);
}
