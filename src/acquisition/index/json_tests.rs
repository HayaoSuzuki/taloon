//! Typed JSON protocol contract and opt-in measurements.
use super::*;
use rstest::rstest;
use std::{hint::black_box, time::Instant};

fn response(body: Vec<u8>) -> Response {
    Response {
        status: 200,
        location: None,
        content_type: "application/json".into(),
        body,
        url: Url::parse("https://index.test/simple/example/").unwrap(),
    }
}
fn hex(bytes: &[u8]) -> String {
    ContentHash::digest(bytes)
        .to_string()
        .replace("sha256:", "")
}
#[rstest]
#[case::one_thousand(1000)]
#[case::ten_thousand(10000)]
#[ignore = "opt-in release JSON protocol measurement, no time threshold"]
fn historical_json_index_parse(#[case] historical: usize) {
    // Arrange: all fixture construction is outside timing.
    let archive = hex(b"archive");
    let metadata = hex(b"metadata");
    let mut files: Vec<_> = (0..historical)
        .map(|n| {
            serde_json::json!({
                "filename": format!("old-{n}.whl"), "url": format!("https://history.test/{n}.whl"),
                "hashes": {"sha256": archive}, "core-metadata": {"sha256": metadata},
                "requires-python": ">=3.14", "yanked": false,
                "extension": {"nested": [1,2,3], "text": "x".repeat(128)}
            })
        })
        .collect();
    let target = Url::parse("https://files.test/target.whl").unwrap();
    files.push(
        serde_json::json!({"url":target.as_str(), "hashes":{"sha256":archive},
        "core-metadata":{"sha256":metadata}}),
    );
    let response = response(serde_json::to_vec(&serde_json::json!({"files":files})).unwrap());
    let targets = BTreeSet::from([target]);
    for sample in 1..=5 {
        // Act
        let started = Instant::now();
        let index = black_box(PackageIndex::parse(black_box(&response), &targets)).unwrap();
        let elapsed_us = started.elapsed().as_micros();
        // Assert
        assert_eq!(index.links.len(), 1);
        println!(
            "historical={historical} bytes={} sample={sample} elapsed_us={elapsed_us}",
            response.body.len()
        );
    }
}

#[rstest]
#[case::unsupported_api_version(r#"{"meta":{"api-version":"2.0"},"files":[]}"#)]
#[case::missing_url(r#"{"files":[{}]}"#)]
#[case::numeric_url(r#"{"files":[{"url":4}]}"#)]
#[case::invalid_meta(r#"{"meta":4,"files":[]}"#)]
#[case::numeric_version(r#"{"meta":{"api-version":4},"files":[]}"#)]
#[case::duplicate_files(r#"{"files":[],"files":[]}"#)]
#[case::duplicate_url(r#"{"files":[{"url":"a","url":"b"}]}"#)]
#[case::invalid_hashes(r#"{"files":[{"url":"a","hashes":"bad"}]}"#)]
#[case::numeric_hash(r#"{"files":[{"url":"a","hashes":{"sha256":7}}]}"#)]
#[case::null_modern(r#"{"files":[{"url":"a","core-metadata":null}]}"#)]
#[case::numeric_modern(r#"{"files":[{"url":"a","core-metadata":17}]}"#)]
#[case::invalid_legacy(r#"{"files":[{"url":"a","dist-info-metadata":"wrong"}]}"#)]
#[case::positional_document(r"[{},[]]")]
#[case::positional_meta(r#"{"meta":["1.0"],"files":[]}"#)]
#[case::positional_file(r#"{"files":[["https://files.test/target.whl",{"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},{"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},false]]}"#)]
#[case::positional_hashes(r#"{"files":[{"url":"https://files.test/target.whl","hashes":["aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"],"core-metadata":{"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}}]}"#)]
#[case::positional_modern(r#"{"files":[{"url":"https://files.test/target.whl","core-metadata":["aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"]}]}"#)]
#[case::positional_legacy(r#"{"files":[{"url":"https://files.test/target.whl","dist-info-metadata":["aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"]}]}"#)]
fn malformed_or_duplicate_json_fields_are_rejected(#[case] text: &str) {
    // Arrange
    let response = response(text.as_bytes().to_vec());
    let targets = BTreeSet::from([Url::parse("https://files.test/target.whl").unwrap()]);
    // Act
    let result = PackageIndex::parse(&response, &targets);
    // Assert
    assert_eq!(result.err().unwrap().code, Code::InvalidMetadata);
}

fn advertisement(code: u8) -> Option<serde_json::Value> {
    // The fixed and generated cases select the six advertised states by index.
    [
        None,
        Some(serde_json::json!(false)),
        Some(serde_json::json!(true)),
        Some(serde_json::json!({"sha256":hex(b"metadata")})),
        Some(serde_json::json!({"sha256":"invalid"})),
        Some(serde_json::json!({"sha512":"unusable"})),
    ][usize::from(code)]
    .clone()
}
fn page(modern: u8, legacy: u8, extension: &str) -> Response {
    let mut file = serde_json::json!({"url":"https://files.test/target.whl",
        "hashes":{"sha256":hex(b"archive")}, "extension":{"text":extension}});
    if let Some(value) = advertisement(modern) {
        file["core-metadata"] = value;
    }
    if let Some(value) = advertisement(legacy) {
        file["dist-info-metadata"] = value;
    }
    response(serde_json::to_vec(&serde_json::json!({"files":[file], "extension":[1,2,3]})).unwrap())
}
#[rstest]
#[case::absent(0, 0, false)]
#[case::modern_false(1, 0, false)]
#[case::modern_true(2, 0, false)]
#[case::modern_hash(3, 0, true)]
#[case::legacy_hash(0, 3, true)]
#[case::modern_false_wins(1, 3, false)]
#[case::modern_true_wins(2, 3, false)]
#[case::modern_invalid_hash_wins(4, 3, false)]
#[case::modern_other_algorithm_wins(5, 3, false)]
#[case::modern_valid_wins(3, 4, true)]
fn json_advertisements_require_a_selected_digest(
    #[case] modern: u8,
    #[case] legacy: u8,
    #[case] associated: bool,
) {
    // Arrange
    let response = page(modern, legacy, "unknown extensions are ignored");
    let target = Url::parse("https://files.test/target.whl").unwrap();
    // Act
    let result = sidecar(&response, &target, ContentHash::digest(b"archive"));
    // Assert
    assert_eq!(result.unwrap().is_some(), associated);
}

use proptest::prelude::*;
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn metadata_selection_matches_integer_advertisement_oracle(
        modern in 0_u8..6, legacy in 0_u8..6, extension in ".{0,128}",
    ) {
        // Arrange: only integer code 3 supplies SHA-256; code 0 is absence.
        let expected = if modern == 0 { legacy == 3 } else { modern == 3 };
        let response = page(modern, legacy, &extension);
        let target = Url::parse("https://files.test/target.whl").unwrap();
        // Act
        let result = sidecar(&response, &target, ContentHash::digest(b"archive"));
        // Assert
        prop_assert_eq!(result.unwrap().is_some(), expected);
    }
}

#[test]
fn malformed_unselected_url_does_not_create_an_association() {
    // Arrange
    let response = response(br#"{"files":[{"url":"https://[invalid"}]}"#.to_vec());
    let target = Url::parse("https://files.test/target.whl").unwrap();
    // Act
    let result = PackageIndex::parse(&response, &BTreeSet::from([target]));
    // Assert
    assert_eq!(result.unwrap().links.len(), 0);
}
