use super::cache::EvidenceCache;
use crate::testfixtures::{metadata, request, wheel};
use taloon::{
    evidence::{Evidence, SidecarAssociation},
    model::{ArtifactFormat, ContentHash, EvidenceRequirement},
};
#[test]
fn cache_requires_matching_identity_strength_and_format() {
    let temp = tempfile::tempdir().unwrap();
    let cache = EvidenceCache::new(temp.path().to_owned());
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let req = request(&bytes, ArtifactFormat::Wheel);
    let hash = ContentHash::digest(&bytes);
    let e = Evidence::from_archive(&req, &bytes).unwrap();
    cache.put(&req, &e).unwrap();
    assert!(cache.get(&req, hash).unwrap().is_some());
    assert!(
        cache
            .get(&req, ContentHash::digest(b"wrong"))
            .unwrap()
            .is_none()
    );
    let temp = tempfile::tempdir().unwrap();
    let cache = EvidenceCache::new(temp.path().to_owned());
    let meta = metadata("License-Expression: MIT");
    let sidecar = Evidence::from_sidecar(
        &req,
        SidecarAssociation::new(hash, hash, ContentHash::digest(&meta)).unwrap(),
        &meta,
    )
    .unwrap();
    cache.put(&req, &sidecar).unwrap();
    assert!(
        cache
            .get(
                &req.clone().requiring(EvidenceRequirement::ArchiveVerified),
                hash
            )
            .unwrap()
            .is_none()
    );
    assert!(cache.get(&req, hash).unwrap().is_some());
}

#[test]
fn local_archive_without_lock_digest_is_rehashed() {
    use taloon::model::{ArtifactLocator, ArtifactRequest, Code, PackageId, SourceId};
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("example.whl");
    let bytes = wheel(&metadata("License-Expression: MIT"));
    std::fs::write(&path, &bytes).unwrap();
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
    let mut stats = taloon::report::AcquisitionSummary::default();
    let mut warnings = Vec::new();
    let first = super::acquire_local(&req, temp.path(), &cache, &mut stats, &mut warnings).unwrap();
    assert_eq!(stats.archive_extractions, 1);
    let again = super::acquire_local(&req, temp.path(), &cache, &mut stats, &mut warnings).unwrap();
    assert_eq!(first.content_hash(), again.content_hash());
    assert_eq!(stats.cache_hits, 1);
    let changed = wheel(&metadata("License-Expression: ISC"));
    std::fs::write(&path, &changed).unwrap();
    let last = super::acquire_local(&req, temp.path(), &cache, &mut stats, &mut warnings).unwrap();
    assert_ne!(first.content_hash(), last.content_hash());
    assert_eq!(last.license().unwrap().text(), "ISC");
    let pinned = ArtifactRequest::new(
        req.package().clone(),
        req.locator().clone(),
        req.format(),
        Some(ContentHash::digest(&bytes)),
    )
    .unwrap();
    assert_eq!(
        super::acquire_local(&pinned, temp.path(), &cache, &mut stats, &mut warnings)
            .unwrap_err()
            .code,
        Code::HashMismatch
    );
    std::fs::remove_file(path).unwrap();
    assert!(super::acquire_local(&req, temp.path(), &cache, &mut stats, &mut warnings).is_err());
}

fn cache_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect()
}
#[test]
fn corrupt_incompatible_and_interrupted_records_are_misses() {
    let temp = tempfile::tempdir().unwrap();
    let cache = EvidenceCache::new(temp.path().to_owned());
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let req = request(&bytes, ArtifactFormat::Wheel);
    let hash = ContentHash::digest(&bytes);
    let e = Evidence::from_archive(&req, &bytes).unwrap();
    std::fs::write(temp.path().join(".evidence-interrupted"), b"partial").unwrap();
    assert!(cache.get(&req, hash).unwrap().is_none());
    cache.put(&req, &e).unwrap();
    let path = cache_files(temp.path()).pop().unwrap();
    let original = std::fs::read(&path).unwrap();
    std::fs::write(&path, &original[..original.len() / 2]).unwrap();
    assert!(cache.get(&req, hash).unwrap().is_none());
    let mut incompatible: serde_json::Value = serde_json::from_slice(&original).unwrap();
    incompatible["schema"] = 99.into();
    std::fs::write(&path, serde_json::to_vec(&incompatible).unwrap()).unwrap();
    assert!(cache.get(&req, hash).unwrap().is_none());
    cache.put(&req, &e).unwrap();
    assert!(cache.get(&req, hash).unwrap().is_some());
}
#[test]
fn concurrent_publication_never_exposes_partial_records() {
    let temp = tempfile::tempdir().unwrap();
    let cache = EvidenceCache::new(temp.path().to_owned());
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let req = request(&bytes, ArtifactFormat::Wheel);
    let hash = ContentHash::digest(&bytes);
    let e = Evidence::from_archive(&req, &bytes).unwrap();
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
            if let Some(observed) = cache.get(&req, hash).unwrap() {
                assert_eq!(observed.content_hash(), hash);
            }
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
#[test]
fn global_archive_cache_still_validates_identity_and_format() {
    use taloon::model::{ArtifactRequest, PackageId, SourceId};
    let temp = tempfile::tempdir().unwrap();
    let cache = EvidenceCache::new(temp.path().to_owned());
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let req = request(&bytes, ArtifactFormat::Wheel);
    let hash = ContentHash::digest(&bytes);
    let e = Evidence::from_archive(&req, &bytes).unwrap();
    cache.put(&req, &e).unwrap();
    let other_source = ArtifactRequest::new(
        PackageId::parse(
            "example",
            "1.0",
            SourceId::parse("registry", "https://other.test/simple").unwrap(),
        )
        .unwrap(),
        req.locator().clone(),
        req.format(),
        Some(hash),
    )
    .unwrap();
    assert!(cache.get(&other_source, hash).unwrap().is_some());
    let wrong = ArtifactRequest::new(
        PackageId::parse("wrong", "1.0", req.package().source().clone()).unwrap(),
        req.locator().clone(),
        req.format(),
        Some(hash),
    )
    .unwrap();
    assert!(cache.get(&wrong, hash).unwrap().is_none());
    assert!(
        cache
            .get(&request(&bytes, ArtifactFormat::Sdist), hash)
            .unwrap()
            .is_none()
    );
}
proptest::proptest! {
 #![proptest_config(proptest::test_runner::Config::with_cases(256))]
 #[test]
 fn persisted_observations_roundtrip_and_header_mutations_miss(license in proptest::sample::select(vec!["MIT","MIT OR ISC","Apache-2.0"]),bit in 0_u8..7){
 let temp=tempfile::tempdir().unwrap();let cache=EvidenceCache::new(temp.path().to_owned());let bytes=wheel(&metadata(&format!("License-Expression: {license}")));let req=request(&bytes,ArtifactFormat::Wheel);let hash=ContentHash::digest(&bytes);let e=Evidence::from_archive(&req,&bytes).unwrap();cache.put(&req,&e).unwrap();let replay=cache.get(&req,hash).unwrap().unwrap();proptest::prop_assert_eq!(replay.license().unwrap().text().to_owned(),license);
 let path=cache_files(temp.path()).pop().unwrap();let mut encoded=std::fs::read(&path).unwrap();let at=encoded.windows(license.len()).position(|s|s==license.as_bytes()).unwrap();encoded[at]^=1<<bit;std::fs::write(path,encoded).unwrap();proptest::prop_assert!(cache.get(&req,hash).unwrap().is_none());
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
