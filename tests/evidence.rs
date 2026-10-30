mod common;
use common::{metadata, request, sdist, wheel};
use proptest::prelude::*;
use taloon::{
    evidence::{Evidence, EvidenceRecord, SidecarAssociation},
    model::{ArtifactFormat, Code, ContentHash, EvidenceRequirement},
};

#[test]
fn archive_and_sidecar_proofs_remain_distinct() {
    let meta = metadata("License-Expression: MIT");
    let bytes = wheel(&meta);
    let req = request(&bytes, ArtifactFormat::Wheel);
    let hash = ContentHash::digest(&bytes);
    let archive = Evidence::from_archive(&req, &bytes).unwrap();
    assert!(archive.requirement_satisfied(EvidenceRequirement::ArchiveVerified));
    let sidecar = Evidence::from_sidecar(
        &req,
        SidecarAssociation::new(hash, hash, ContentHash::digest(&meta)).unwrap(),
        &meta,
    )
    .unwrap();
    assert!(sidecar.requirement_satisfied(EvidenceRequirement::IndexAssociated));
    assert!(!sidecar.requirement_satisfied(EvidenceRequirement::ArchiveVerified));
    assert!(
        SidecarAssociation::new(
            hash,
            ContentHash::digest(b"wrong"),
            ContentHash::digest(&meta)
        )
        .is_err()
    );
    let mut changed = bytes.clone();
    changed[0] ^= 1;
    assert_eq!(
        Evidence::from_archive(&req, &changed).unwrap_err().code,
        Code::HashMismatch
    );
    assert!(
        Evidence::from_sidecar(
            &req,
            SidecarAssociation::new(hash, hash, ContentHash::digest(b"wrong")).unwrap(),
            &meta
        )
        .is_err()
    );
}

#[test]
fn metadata_license_precedence_and_identity() {
    for (fields, expected) in [
        ("License-Expression: MIT\nLicense: Apache-2.0", Some("MIT")),
        ("License-Expression: WRONG\nLicense: MIT", None),
        ("License: MIT OR Apache-2.0", Some("MIT OR Apache-2.0")),
        ("Classifier: License :: OSI Approved :: MIT License", None),
        ("License: This is MIT-ish", None),
    ] {
        let bytes = wheel(&metadata(fields));
        let e = Evidence::from_archive(&request(&bytes, ArtifactFormat::Wheel), &bytes).unwrap();
        assert_eq!(
            e.license()
                .ok()
                .as_ref()
                .map(taloon::policy::LicenseExpression::text),
            expected
        );
    }
    let wrong = String::from_utf8(metadata("License-Expression: MIT"))
        .unwrap()
        .replace("Name: example", "Name: wrong");
    let bytes = wheel(wrong.as_bytes());
    assert_eq!(
        Evidence::from_archive(&request(&bytes, ArtifactFormat::Wheel), &bytes)
            .unwrap_err()
            .code,
        Code::IdentityMismatch
    );
}

#[test]
fn folded_duplicate_and_future_metadata_have_explicit_outcomes() {
    for (fields, expected) in [
        ("License-Expression: MIT OR\n Apache-2.0", true),
        (
            "License-Expression: MIT\nLicense-Expression: Apache-2.0",
            false,
        ),
    ] {
        let bytes = wheel(&metadata(fields));
        let e = Evidence::from_archive(&request(&bytes, ArtifactFormat::Wheel), &bytes).unwrap();
        assert_eq!(e.license().is_ok(), expected);
    }
    for (version, valid, warnings) in [
        ("1.2", true, 0),
        ("2.6", true, 0),
        ("2.7", true, 1),
        ("3.0", false, 0),
    ] {
        let meta = String::from_utf8(metadata("License: MIT"))
            .unwrap()
            .replace("2.4", version);
        let bytes = wheel(meta.as_bytes());
        let e = Evidence::from_archive(&request(&bytes, ArtifactFormat::Wheel), &bytes);
        assert_eq!(e.is_ok(), valid);
        if let Ok(e) = e {
            assert_eq!(e.summary().warnings.len(), warnings);
        }
    }
    let meta = metadata("Name: repeated\nLicense-Expression: MIT");
    let bytes = wheel(&meta);
    assert!(Evidence::from_archive(&request(&bytes, ArtifactFormat::Wheel), &bytes).is_err());
}

#[test]
fn source_distributions_do_not_claim_future_wheel_metadata() {
    let meta = metadata("License-Expression: MIT\nDynamic: License-Expression");
    let bytes = sdist(&meta);
    let e = Evidence::from_archive(&request(&bytes, ArtifactFormat::Sdist), &bytes).unwrap();
    assert_eq!(e.license().unwrap().text(), "MIT");
    let duplicated = common::wheel_with_two_metadata(&meta);
    assert!(
        Evidence::from_archive(&request(&duplicated, ArtifactFormat::Wheel), &duplicated).is_err()
    );
}

#[test]
fn cache_record_corruption_or_wrong_identity_cannot_create_evidence() {
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let req = request(&bytes, ArtifactFormat::Wheel);
    let e = Evidence::from_archive(&req, &bytes).unwrap();
    let encoded = serde_json::to_vec(&e.to_record()).unwrap();
    let mut changed = encoded.clone();
    let at = changed.windows(3).position(|s| s == b"MIT").unwrap();
    changed[at..at + 3].copy_from_slice(b"ISC");
    let record: EvidenceRecord = serde_json::from_slice(&changed).unwrap();
    assert!(Evidence::from_record(&req, record).is_err());
    let record = serde_json::from_slice(&encoded).unwrap();
    assert!(Evidence::from_record(&request(b"changed", ArtifactFormat::Wheel), record).is_err());
}

#[test]
fn invalid_historical_metadata_versions_are_not_future_compatible_minors() {
    for version in ["1.3", "2.0"] {
        let meta = String::from_utf8(metadata("License-Expression: MIT"))
            .unwrap()
            .replace("2.4", version);
        let bytes = wheel(meta.as_bytes());
        assert!(
            Evidence::from_archive(&request(&bytes, ArtifactFormat::Wheel), &bytes).is_err(),
            "{version}"
        );
    }
}

#[test]
fn cached_declarations_do_not_retain_description_bodies() {
    let small = metadata("License-Expression: MIT");
    let mut large = small.clone();
    large.extend_from_slice("description".repeat(10_000).as_bytes());
    let small_archive = wheel(&small);
    let large_archive = wheel(&large);
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
    assert!(Evidence::from_archive(&request(&bytes, ArtifactFormat::Wheel), &bytes).is_err());
}

#[test]
fn cached_sidecars_cannot_gain_strength_or_change_registry_associations() {
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
    let restored = Evidence::from_record(&req, e.to_record()).unwrap();
    assert!(!restored.requirement_satisfied(EvidenceRequirement::ArchiveVerified));
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
    assert!(Evidence::from_record(&other, e.to_record()).is_err());
}

#[test]
fn metadata_identity_and_encoding_failures_cannot_be_license_overrides() {
    for fields in [
        "Version: 2.0\nLicense-Expression: MIT",
        "Name: second\nLicense-Expression: MIT",
    ] {
        let bytes = wheel(&metadata(fields));
        assert!(Evidence::from_archive(&request(&bytes, ArtifactFormat::Wheel), &bytes).is_err());
    }
    let mut meta = metadata("License-Expression: MIT");
    meta[0] = 255;
    let bytes = wheel(&meta);
    assert!(Evidence::from_archive(&request(&bytes, ArtifactFormat::Wheel), &bytes).is_err());
}

proptest! {
    #![proptest_config(ProptestConfig{cases:256,failure_persistence:Some(Box::new(proptest::test_runner::FileFailurePersistence::Direct("tests/evidence.proptest-regressions"))),..ProptestConfig::default()})]
    #[test]
    fn changing_archive_bytes_rejects_the_locked_digest(position in any::<usize>(),bit in 0_u8..8){
        let bytes=wheel(&metadata("License-Expression: MIT"));let req=request(&bytes,ArtifactFormat::Wheel);let mut changed=bytes.clone();let length=changed.len();changed[position%length]^=1<<bit;
        prop_assert_eq!(Evidence::from_archive(&req,&changed).unwrap_err().code,Code::HashMismatch);
    }
    #[test]
    fn cache_roundtrip_preserves_declarations_and_strength(license in prop::sample::select(vec!["MIT","Apache-2.0","MIT OR Apache-2.0","WRONG"]),sidecar in any::<bool>()){
        let meta=metadata(&format!("License-Expression: {license}"));let bytes=wheel(&meta);let req=request(&bytes,ArtifactFormat::Wheel);let hash=ContentHash::digest(&bytes);
        let e=if sidecar{Evidence::from_sidecar(&req,SidecarAssociation::new(hash,hash,ContentHash::digest(&meta)).unwrap(),&meta).unwrap()}else{Evidence::from_archive(&req,&bytes).unwrap()};
        let restored=Evidence::from_record(&req,e.to_record()).unwrap();
        prop_assert_eq!(e.license().map(|s|s.text().to_owned()),restored.license().map(|s|s.text().to_owned()));
        prop_assert_eq!(e.requirement_satisfied(EvidenceRequirement::ArchiveVerified),restored.requirement_satisfied(EvidenceRequirement::ArchiveVerified));
    }
}

#[test]
fn archive_records_cannot_change_distribution_format() {
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let req = request(&bytes, ArtifactFormat::Wheel);
    let evidence = Evidence::from_archive(&req, &bytes).unwrap();
    let wrong = request(&bytes, ArtifactFormat::Sdist);
    assert!(Evidence::from_record(&wrong, evidence.to_record()).is_err());
}

#[test]
fn cached_sidecars_require_the_original_artifact_association() {
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
    assert!(Evidence::from_record(&second, evidence.to_record()).is_err());
}
