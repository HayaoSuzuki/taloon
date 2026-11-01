//! HTML encounter-order contracts and opt-in synthetic parser measurements.
use super::*;
use proptest::prelude::*;
use rstest::rstest;
use std::{fmt::Write as _, hint::black_box, time::Instant};

fn response(body: String) -> Response {
    Response {
        status: 200,
        location: None,
        content_type: "text/html".into(),
        body: body.into_bytes(),
        url: Url::parse("https://index.test/simple/example/").unwrap(),
    }
}
fn hex(bytes: &[u8]) -> String {
    ContentHash::digest(bytes)
        .to_string()
        .replace("sha256:", "")
}
fn anchor(href: &str, attributes: &str) -> String {
    format!(
        "<a href='{href}#sha256={}' {attributes}>target</a>",
        hex(b"archive")
    )
}
fn link() -> String {
    anchor(
        "target.whl",
        &format!("data-core-metadata='sha256={}'", hex(b"metadata")),
    )
}

#[rstest]
#[case::response_base("{link}", "https://index.test/simple/example/target.whl", false)]
#[case::declared_base(
    "<base href='https://cdn.test/files/'>{link}",
    "https://cdn.test/files/target.whl",
    false
)]
#[case::late_base(
    "{link}<base href='https://cdn.test/files/'>",
    "https://index.test/simple/example/target.whl",
    false
)]
#[case::first_base(
    "<base href='https://cdn.test/first/'><base href='https://cdn.test/second/'>{link}",
    "https://cdn.test/first/target.whl",
    false
)]
#[case::missing_href(
    "<base><base href='https://cdn.test/files/'>{link}",
    "https://cdn.test/files/target.whl",
    false
)]
#[case::late_version_error(
    "{link}<meta name='pypi:repository-version' content='2.0'>",
    "https://index.test/simple/example/target.whl",
    true
)]
#[case::duplicate_target("{link}{link}", "https://index.test/simple/example/target.whl", true)]
#[case::invalid_base(
    "<base href='https://['>{link}",
    "https://index.test/simple/example/target.whl",
    true
)]
fn html_associations_preserve_encounter_order(
    #[case] template: &str,
    #[case] target: &str,
    #[case] invalid: bool,
) {
    // Arrange
    let response = response(template.replace("{link}", &link()));
    let target = Url::parse(target).unwrap();
    // Act
    let result = sidecar(&response, &target, ContentHash::digest(b"archive"));
    // Assert
    if invalid {
        assert_eq!(result.err().unwrap().code, Code::InvalidMetadata);
    } else {
        let (url, _) = result.unwrap().unwrap();
        assert_eq!(url.as_str(), format!("{target}.metadata"));
    }
}

#[rstest]
#[case::script("script")]
#[case::style("style")]
#[case::xmp("xmp")]
#[case::iframe("iframe")]
#[case::noembed("noembed")]
#[case::noframes("noframes")]
#[case::textarea("textarea")]
#[case::title("title")]
fn raw_text_does_not_advertise_a_link(#[case] tag: &str) {
    // Arrange
    let response = response(format!("<{tag}>{}</{tag}>", link()));
    let target = Url::parse("https://index.test/simple/example/target.whl").unwrap();
    // Act
    let result = sidecar(&response, &target, ContentHash::digest(b"archive"));
    // Assert
    assert!(result.unwrap().is_none());
}

#[rstest]
#[case::modern_valid(
    "data-core-metadata='sha256={hash}' data-dist-info-metadata='false'",
    true
)]
#[case::modern_invalid(
    "data-core-metadata='false' data-dist-info-metadata='sha256={hash}'",
    false
)]
#[case::duplicate_modern_first_valid(
    "data-core-metadata='sha256={hash}' data-core-metadata='false'",
    true
)]
#[case::duplicate_modern_first_invalid(
    "data-core-metadata='false' data-core-metadata='sha256={hash}'",
    false
)]
#[case::uppercase_attribute("DATA-CORE-METADATA='sha256={hash}'", true)]
#[case::legacy_only("data-dist-info-metadata='sha256={hash}'", true)]
fn modern_metadata_attribute_has_precedence(#[case] attributes: &str, #[case] found: bool) {
    // Arrange
    let attributes = attributes.replace("{hash}", &hex(b"metadata"));
    let response = response(anchor("target.whl", &attributes));
    let target = Url::parse("https://index.test/simple/example/target.whl").unwrap();
    // Act
    let result = sidecar(&response, &target, ContentHash::digest(b"archive"));
    // Assert
    assert_eq!(result.unwrap().is_some(), found);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn base_changes_only_subsequent_anchors(
        historical in 0_usize..128,
        base_before in any::<bool>(),
        suffix in "[a-z]{1,16}",
    ) {
        // Arrange: absolute expected URL comes from the chosen encounter order.
        let mut html = String::new();
        for n in 0..historical {
            write!(html, "<a href='https://history.test/{n}.whl'>old</a>").unwrap();
        }
        let base = format!("<base href='https://cdn.test/{suffix}/'>");
        if base_before { html.push_str(&base); }
        html.push_str(&link());
        if !base_before { html.push_str(&base); }
        html.push_str("<base href='https://ignored.test/'>");
        let target = if base_before {
            format!("https://cdn.test/{suffix}/target.whl")
        } else {
            "https://index.test/simple/example/target.whl".to_owned()
        };
        let response = response(html);
        let target = Url::parse(&target).unwrap();
        let targets = BTreeSet::from([target.clone()]);

        // Act
        let parsed = PackageIndex::parse(&response, &targets).unwrap();
        let result = parsed.sidecar(&target, ContentHash::digest(b"archive"));

        // Assert
        prop_assert_eq!(parsed.links.len(), 1);
        prop_assert_eq!(result.unwrap().unwrap().0.to_string(), format!("{target}.metadata"));
    }
}

#[rstest]
#[case::one_thousand(1000)]
#[case::ten_thousand(10000)]
#[ignore = "opt-in release measurement; no CI elapsed-time threshold"]
fn historical_html_index_parse(#[case] historical: usize) {
    // Arrange: body construction is outside measured parsing.
    let mut html = String::new();
    for n in 0..historical {
        write!(html, "<a href='https://history.test/{n}.whl#sha256={}' data-core-metadata='sha256={}' data-requires-python='&gt;=3.14'>old</a>", hex(b"archive"), hex(b"metadata")).unwrap();
    }
    html.push_str(&link());
    let response = response(html);
    let targets =
        BTreeSet::from([Url::parse("https://index.test/simple/example/target.whl").unwrap()]);
    for sample in 1..=5 {
        // Act
        let started = Instant::now();
        let parsed = black_box(PackageIndex::parse(&response, &targets)).unwrap();
        let elapsed_us = started.elapsed().as_micros();
        // Assert
        assert_eq!(parsed.links.len(), 1);
        println!(
            "historical={historical} bytes={} sample={sample} elapsed_us={elapsed_us}",
            response.body.len()
        );
    }
}
