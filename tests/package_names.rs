use rstest::rstest;
use taloon::model::{Code, normalize_name};

#[rstest]
#[case::empty("")]
#[case::unicode("名前")]
#[case::leading("_name")]
#[case::trailing("name.")]
#[case::slash("some/name")]
fn invalid_package_name_preserves_public_error(#[case] name: &str) {
    // Arrange: an invalid public string input.
    // Act
    let error = normalize_name(name).unwrap_err();
    // Assert
    assert_eq!(error.code, Code::Configuration);
    assert_eq!(error.message, "invalid Python package name");
}
