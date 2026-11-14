//! Domain debug formatting must not reveal fetching credentials or query values.
use proptest::prelude::*;
use rstest::rstest;
use taloon::model::{
    ArtifactFormat, ArtifactLocator, ArtifactRequest, ContentHash, PackageId, SourceId,
};

#[derive(Clone, Copy, Debug)]
enum Value {
    Source,
    Package,
    Locator,
    Request,
}
fn debug_value(value: Value, secret: &str) -> String {
    let url = format!("https://user:{secret}@example.test/file.whl?token={secret}#fragment");
    let source = SourceId::parse("url", &url).unwrap();
    let package = PackageId::parse("example", "1.0", source.clone()).unwrap();
    let locator = ArtifactLocator::registry(&url, &url).unwrap();
    match value {
        Value::Source => format!("{source:?}"),
        Value::Package => format!("{package:?}"),
        Value::Locator => format!("{locator:?}"),
        Value::Request => format!(
            "{:?}",
            ArtifactRequest::new(
                package,
                locator,
                ArtifactFormat::Wheel,
                Some(ContentHash::digest(b"content"))
            )
            .unwrap()
        ),
    }
}
#[rstest]
#[case::source(Value::Source)]
#[case::package(Value::Package)]
#[case::locator(Value::Locator)]
#[case::request(Value::Request)]
fn debug_representation_redacts_nested_fetch_secrets(#[case] value: Value) {
    // Arrange
    let secret = "private-credential";
    // Act
    let rendered = debug_value(value, secret);
    // Assert
    assert!(!rendered.contains(secret));
    assert!(!rendered.contains("user:"));
    assert!(rendered.contains("example.test"));
    assert!(rendered.contains("redacted"));
}
proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]
    #[test]
    fn arbitrary_query_secrets_do_not_escape_debug(suffix in "[a-zA-Z0-9]{1,32}", value in prop::sample::select(vec![Value::Source, Value::Package, Value::Locator, Value::Request])) {
        // Arrange
        let secret = format!("private-{suffix}");
        // Act
        let rendered = debug_value(value, &secret);
        // Assert
        prop_assert!(!rendered.contains(&secret));
        prop_assert!(rendered.contains("example.test"));
    }
}
