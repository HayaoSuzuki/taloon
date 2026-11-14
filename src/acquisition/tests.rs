use super::cache::EvidenceCache;
use crate::testfixtures::{metadata, request, wheel};
use taloon::{
    evidence::{Evidence, SidecarAssociation},
    model::{ArtifactFormat, ContentHash, EvidenceRequirement},
};
#[rstest::rstest]
#[case::matching(true, true)]
#[case::different_content(false, false)]
fn archive_cache_requires_matching_content(#[case] matching: bool, #[case] expected: bool) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let cache = EvidenceCache::new(temp.path().to_owned());
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let req = request(&bytes, ArtifactFormat::Wheel);
    let evidence = Evidence::from_archive(&req, &bytes).unwrap();
    cache.put(&req, &evidence).unwrap();
    let hash = ContentHash::digest(if matching { &bytes } else { b"wrong" });
    // Act
    let replay = cache.get(&req, hash).unwrap();
    // Assert
    assert_eq!(replay.is_some(), expected);
}

#[rstest::rstest]
#[case::associated(EvidenceRequirement::IndexAssociated, true)]
#[case::archive_required(EvidenceRequirement::ArchiveVerified, false)]
fn sidecar_cache_requires_sufficient_strength(
    #[case] requirement: EvidenceRequirement,
    #[case] expected: bool,
) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let cache = EvidenceCache::new(temp.path().to_owned());
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let req = request(&bytes, ArtifactFormat::Wheel).requiring(requirement);
    let hash = ContentHash::digest(&bytes);
    let meta = metadata("License-Expression: MIT");
    let evidence = Evidence::from_sidecar(
        &req,
        SidecarAssociation::new(hash, hash, ContentHash::digest(&meta)).unwrap(),
        &meta,
    )
    .unwrap();
    cache.put(&req, &evidence).unwrap();
    // Act
    let replay = cache.get(&req, hash).unwrap();
    // Assert
    assert_eq!(replay.is_some(), expected);
}

fn local_fixture() -> (
    tempfile::TempDir,
    taloon::model::ArtifactRequest,
    EvidenceCache,
) {
    use taloon::model::{ArtifactLocator, ArtifactRequest, PackageId, SourceId};
    let temp = tempfile::tempdir().unwrap();
    let bytes = wheel(&metadata("License-Expression: MIT"));
    std::fs::write(temp.path().join("example.whl"), &bytes).unwrap();
    let req = ArtifactRequest::new(
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
    .unwrap();
    let cache = EvidenceCache::new(temp.path().join("cache"));
    (temp, req, cache)
}

#[test]
fn local_archive_without_lock_digest_is_extracted() {
    // Arrange
    let (temp, req, cache) = local_fixture();
    let mut stats = taloon::report::AcquisitionSummary::default();
    let mut warnings = Vec::new();
    // Act
    let evidence =
        super::acquire_local(&req, temp.path(), &cache, &mut stats, &mut warnings).unwrap();
    // Assert
    assert_eq!(stats.archive_extractions, 1);
    assert_eq!(evidence.license().unwrap().text(), "MIT");
}

#[test]
fn unchanged_unpinned_local_archive_reuses_cached_evidence() {
    // Arrange
    let (temp, req, cache) = local_fixture();
    let mut stats = taloon::report::AcquisitionSummary::default();
    let mut warnings = Vec::new();
    let first = super::acquire_local(&req, temp.path(), &cache, &mut stats, &mut warnings).unwrap();
    stats = taloon::report::AcquisitionSummary::default();
    // Act
    let again = super::acquire_local(&req, temp.path(), &cache, &mut stats, &mut warnings).unwrap();
    // Assert
    assert_eq!(first.content_hash(), again.content_hash());
    assert_eq!(stats.cache_hits, 1);
    assert_eq!(stats.archive_extractions, 0);
}

#[test]
fn changed_unpinned_local_archive_is_rehashed() {
    // Arrange
    let (temp, req, cache) = local_fixture();
    let mut stats = taloon::report::AcquisitionSummary::default();
    let mut warnings = Vec::new();
    let first = super::acquire_local(&req, temp.path(), &cache, &mut stats, &mut warnings).unwrap();
    let changed = wheel(&metadata("License-Expression: ISC"));
    std::fs::write(temp.path().join("example.whl"), &changed).unwrap();
    // Act
    let last = super::acquire_local(&req, temp.path(), &cache, &mut stats, &mut warnings).unwrap();
    // Assert
    assert_ne!(first.content_hash(), last.content_hash());
    assert_eq!(last.license().unwrap().text(), "ISC");
}

#[test]
fn changed_pinned_local_archive_is_rejected() {
    // Arrange
    let (temp, req, cache) = local_fixture();
    let mut stats = taloon::report::AcquisitionSummary::default();
    let mut warnings = Vec::new();
    // Warm evidence must not bypass re-reading and validating the changed archive.
    super::acquire_local(&req, temp.path(), &cache, &mut stats, &mut warnings).unwrap();
    let bytes = std::fs::read(temp.path().join("example.whl")).unwrap();
    let pinned = taloon::model::ArtifactRequest::new(
        req.package().clone(),
        req.locator().clone(),
        req.format(),
        Some(ContentHash::digest(&bytes)),
    )
    .unwrap();
    std::fs::write(
        temp.path().join("example.whl"),
        wheel(&metadata("License-Expression: ISC")),
    )
    .unwrap();
    // Act
    let error =
        super::acquire_local(&pinned, temp.path(), &cache, &mut stats, &mut warnings).unwrap_err();
    // Assert
    assert_eq!(error.code, taloon::model::Code::HashMismatch);
}

#[test]
fn removed_local_archive_cannot_reuse_cached_evidence() {
    // Arrange
    let (temp, req, cache) = local_fixture();
    let mut stats = taloon::report::AcquisitionSummary::default();
    let mut warnings = Vec::new();
    super::acquire_local(&req, temp.path(), &cache, &mut stats, &mut warnings).unwrap();
    std::fs::remove_file(temp.path().join("example.whl")).unwrap();
    // Act
    let result = super::acquire_local(&req, temp.path(), &cache, &mut stats, &mut warnings);
    // Assert
    assert!(result.is_err());
}

fn cache_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect()
}

#[test]
fn obsolete_payload_is_a_disk_cache_miss() {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let cache = EvidenceCache::new(temp.path().to_owned());
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let req = request(&bytes, ArtifactFormat::Wheel);
    let hash = ContentHash::digest(&bytes);
    cache
        .put(&req, &Evidence::from_archive(&req, &bytes).unwrap())
        .unwrap();
    let path = cache_files(temp.path()).pop().unwrap();
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    value["observation"]["data"]["schema"] = 1.into();
    std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
    // Act
    let restored = cache.get(&req, hash).unwrap();
    // Assert
    assert!(restored.is_none());
}
enum CacheDamage {
    Interrupted,
    Truncated,
    Incompatible,
}

fn damage_cache_record(
    path: &std::path::Path,
    root: &std::path::Path,
    original: &[u8],
    damage: &CacheDamage,
) {
    match damage {
        CacheDamage::Interrupted => {
            std::fs::rename(path, root.join(".evidence-interrupted")).unwrap();
        }
        CacheDamage::Truncated => std::fs::write(path, &original[..original.len() / 2]).unwrap(),
        CacheDamage::Incompatible => {
            let mut value: serde_json::Value = serde_json::from_slice(original).unwrap();
            value["schema"] = 99.into();
            std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
        }
    }
}

#[rstest::rstest]
#[case::interrupted(&CacheDamage::Interrupted)]
#[case::truncated(&CacheDamage::Truncated)]
#[case::incompatible(&CacheDamage::Incompatible)]
fn invalid_cache_record_is_a_miss(#[case] damage: &CacheDamage) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let cache = EvidenceCache::new(temp.path().to_owned());
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let req = request(&bytes, ArtifactFormat::Wheel);
    let hash = ContentHash::digest(&bytes);
    let evidence = Evidence::from_archive(&req, &bytes).unwrap();
    cache.put(&req, &evidence).unwrap();
    let path = cache_files(temp.path()).pop().unwrap();
    let original = std::fs::read(&path).unwrap();
    damage_cache_record(&path, temp.path(), &original, damage);
    // Act
    let replay = cache.get(&req, hash).unwrap();
    // Assert
    assert!(replay.is_none());
}
#[test]
fn concurrent_publication_never_exposes_partial_records() {
    let temp = tempfile::tempdir().unwrap();
    let cache = EvidenceCache::new(temp.path().to_owned());
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let req = request(&bytes, ArtifactFormat::Wheel);
    let hash = ContentHash::digest(&bytes);
    let e = Evidence::from_archive(&req, &bytes).unwrap();
    cache.put(&req, &e).unwrap();
    std::thread::scope(|scope| {
        let a = scope.spawn(|| {
            for _ in 0..16 {
                cache.put(&req, &e).unwrap();
            }
        });
        let b = scope.spawn(|| {
            for _ in 0..16 {
                cache.put(&req, &e).unwrap();
            }
        });
        for _ in 0..64 {
            let observed = cache
                .get(&req, hash)
                .unwrap()
                .expect("published record remains present during replacement");
            assert_eq!(observed.content_hash(), hash);
        }
        a.join().unwrap();
        b.join().unwrap();
    });
    assert!(cache.get(&req, hash).unwrap().is_some());
    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
}
#[test]
fn unavailable_cache_does_not_discard_in_memory_success() {
    use taloon::model::{ArtifactLocator, ArtifactRequest, PackageId, SourceId};
    let temp = tempfile::tempdir().unwrap();
    let bytes = wheel(&metadata("License-Expression: MIT"));
    std::fs::write(temp.path().join("example.whl"), &bytes).unwrap();
    let root = temp.path().join("cache");
    std::fs::write(&root, b"not a directory").unwrap();
    let req = ArtifactRequest::new(
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
    .unwrap();
    let mut stats = taloon::report::AcquisitionSummary::default();
    let mut warnings = Vec::new();
    let e = super::acquire_local(
        &req,
        temp.path(),
        &EvidenceCache::new(root),
        &mut stats,
        &mut warnings,
    )
    .unwrap();
    assert_eq!(e.license().unwrap().text(), "MIT");
    assert!(!warnings.is_empty());
    assert_eq!(stats.archive_extractions, 1);
}
#[rstest::rstest]
#[case::other_source("example", "https://other.test/simple", ArtifactFormat::Wheel, true)]
#[case::wrong_identity("wrong", "https://example.test/simple", ArtifactFormat::Wheel, false)]
#[case::wrong_format("example", "https://example.test/simple", ArtifactFormat::Sdist, false)]
fn global_archive_cache_validates_request(
    #[case] name: &str,
    #[case] source: &str,
    #[case] format: ArtifactFormat,
    #[case] expected: bool,
) {
    use taloon::model::{ArtifactRequest, PackageId, SourceId};
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let cache = EvidenceCache::new(temp.path().to_owned());
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let req = request(&bytes, ArtifactFormat::Wheel);
    let hash = ContentHash::digest(&bytes);
    cache
        .put(&req, &Evidence::from_archive(&req, &bytes).unwrap())
        .unwrap();
    let next = ArtifactRequest::new(
        PackageId::parse(name, "1.0", SourceId::parse("registry", source).unwrap()).unwrap(),
        req.locator().clone(),
        format,
        Some(hash),
    )
    .unwrap();
    // Act
    let replay = cache.get(&next, hash).unwrap();
    // Assert
    assert_eq!(replay.is_some(), expected);
}
proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(256))]
    #[test]
    fn persisted_observations_roundtrip(license in proptest::sample::select(vec!["MIT", "MIT OR ISC", "Apache-2.0"])) {
        // Arrange
        let temp = tempfile::tempdir().unwrap();
        let cache = EvidenceCache::new(temp.path().to_owned());
        let bytes = wheel(&metadata(&format!("License-Expression: {license}")));
        let req = request(&bytes, ArtifactFormat::Wheel);
        let hash = ContentHash::digest(&bytes);
        cache.put(&req, &Evidence::from_archive(&req, &bytes).unwrap()).unwrap();
        // Act
        let replay = cache.get(&req, hash).unwrap().unwrap();
        // Assert: expected declaration comes from the generated metadata, not replay.
        proptest::prop_assert_eq!(replay.license().unwrap().text().to_owned(), license);
    }

    #[test]
    fn persisted_header_mutations_miss(license in proptest::sample::select(vec!["MIT", "MIT OR ISC", "Apache-2.0"]), bit in 0_u8..8) {
        // Arrange
        let temp = tempfile::tempdir().unwrap();
        let cache = EvidenceCache::new(temp.path().to_owned());
        let bytes = wheel(&metadata(&format!("License-Expression: {license}")));
        let req = request(&bytes, ArtifactFormat::Wheel);
        let hash = ContentHash::digest(&bytes);
        cache.put(&req, &Evidence::from_archive(&req, &bytes).unwrap()).unwrap();
        let path = cache_files(temp.path()).pop().unwrap();
        let mut encoded = std::fs::read(&path).unwrap();
        let at = encoded.windows(license.len()).position(|s| s == license.as_bytes()).unwrap();
        encoded[at] ^= 1 << bit;
        std::fs::write(path, encoded).unwrap();
        // Act
        let replay = cache.get(&req, hash).unwrap();
        // Assert: modified raw evidence cannot retain its original digest binding.
        proptest::prop_assert!(replay.is_none());
    }
}

#[test]
fn metadata_at_the_supported_limit_survives_json_cache_expansion() {
    let temp = tempfile::tempdir().unwrap();
    let cache = EvidenceCache::new(temp.path().to_owned());
    let mut meta =
        b"Metadata-Version: 2.4\nName: example\nVersion: 1.0\nLicense-Expression: MIT\nX-Long: "
            .to_vec();
    meta.resize(4 * 1024 * 1024 - 2, b'\\');
    meta.extend_from_slice(b"\n\n");
    let req = request(b"locked archive", ArtifactFormat::Wheel);
    let hash = req.hash().unwrap();
    let evidence = Evidence::from_sidecar(
        &req,
        SidecarAssociation::new(hash, hash, ContentHash::digest(&meta)).unwrap(),
        &meta,
    )
    .unwrap();
    cache.put(&req, &evidence).unwrap();
    let cached = cache.get(&req, hash).unwrap();
    assert!(
        cached.is_some(),
        "supported metadata must survive JSON escaping in the cache"
    );
    assert_eq!(cached.unwrap().license().unwrap().text(), "MIT");
}

#[test]
fn local_acquisition_refuses_remote_request_without_reading_files() {
    // Arrange
    let directory = tempfile::tempdir().unwrap();
    let req = request(b"archive", ArtifactFormat::Wheel);
    let cache = EvidenceCache::new(directory.path().join("cache"));
    let mut summary = taloon::report::AcquisitionSummary::default();
    let mut warnings = Vec::new();
    // Act
    let error = super::acquire_local(&req, directory.path(), &cache, &mut summary, &mut warnings)
        .err()
        .unwrap();
    // Assert
    assert_eq!(error.message, "request is not a local archive");
    assert_eq!(summary.archive_extractions, 0);
}

#[test]
fn local_archive_above_input_limit_is_rejected_before_extraction() {
    use taloon::model::{ArtifactLocator, ArtifactRequest, PackageId, SourceId};
    // Arrange: sparse data exercises the reader boundary without storing a large fixture.
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("example.whl");
    std::fs::File::create(&path)
        .unwrap()
        .set_len(super::MAX_ARCHIVE + 1)
        .unwrap();
    let req = ArtifactRequest::new(
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
    .unwrap();
    let cache = EvidenceCache::new(directory.path().join("cache"));
    let mut summary = taloon::report::AcquisitionSummary::default();
    let mut warnings = Vec::new();
    // Act
    let error = super::acquire_local(&req, directory.path(), &cache, &mut summary, &mut warnings)
        .err()
        .unwrap();
    // Assert
    assert_eq!(error.message, "archive exceeds the 256 MiB input limit");
    assert_eq!(summary.archive_extractions, 0);
}

#[test]
fn local_directory_is_not_read_as_an_archive() {
    use taloon::model::{ArtifactLocator, ArtifactRequest, PackageId, SourceId};
    // Arrange
    let directory = tempfile::tempdir().unwrap();
    let req = ArtifactRequest::new(
        PackageId::parse("example", "1.0", SourceId::parse("path", ".").unwrap()).unwrap(),
        ArtifactLocator::local(".").unwrap(),
        ArtifactFormat::Wheel,
        None,
    )
    .unwrap();
    let cache = EvidenceCache::new(directory.path().join("cache"));
    let mut summary = taloon::report::AcquisitionSummary::default();
    let mut warnings = Vec::new();
    // Act
    let error = super::acquire_local(&req, directory.path(), &cache, &mut summary, &mut warnings)
        .err()
        .unwrap();
    // Assert
    assert_eq!(error.message, "cannot read local artifact");
    assert_eq!(summary.archive_extractions, 0);
}
