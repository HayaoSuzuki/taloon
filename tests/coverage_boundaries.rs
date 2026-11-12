//! Public boundary regressions identified by source coverage.
use rstest::rstest;
use taloon::model::{
    ArtifactFormat, ArtifactLocator, ArtifactRequest, Code, EvaluationDate, PackageId, SourceId,
    SourceKind,
};
use taloon::policy::Policy;

#[rstest]
#[case::unknown("unknown")]
#[case::uppercase("REGISTRY")]
#[case::empty("")]
fn unrecognized_source_kind_is_not_guessed(#[case] value: &str) {
    // Act
    let error = SourceKind::parse(value).unwrap_err();
    // Assert
    assert_eq!(error.code, Code::UnsupportedSource);
}

#[rstest]
#[case::file("file:///tmp/example.whl")]
#[case::ftp("ftp://example.test/example.whl")]
fn artifact_locator_rejects_non_web_scheme(#[case] value: &str) {
    // Act
    let error = ArtifactLocator::url(value).unwrap_err();
    // Assert
    assert_eq!(error.code, Code::UnsupportedSource);
}

#[test]
fn empty_local_locator_is_rejected() {
    // Act
    let error = ArtifactLocator::local("").unwrap_err();
    // Assert
    assert_eq!(error.code, Code::Configuration);
}

#[rstest]
#[case::empty_local(ArtifactLocator::Local(String::new()), "empty local artifact path")]
#[case::unhashed_remote(ArtifactLocator::url("https://example.test/example.whl").unwrap(), "remote artifact lacks a lock digest")]
fn request_revalidates_public_locator_before_binding_identity(
    #[case] locator: ArtifactLocator,
    #[case] message: &str,
) {
    // Arrange
    let package = PackageId::parse(
        "example",
        "1.0",
        SourceId::parse("path", "example.whl").unwrap(),
    )
    .unwrap();
    // Act
    let error = ArtifactRequest::new(package, locator, ArtifactFormat::Wheel, None).unwrap_err();
    // Assert
    assert_eq!(error.code, Code::MissingArtifact);
    assert_eq!(error.message, message);
}

#[test]
fn unsupported_policy_schema_is_rejected() {
    // Arrange
    let date = EvaluationDate::parse("2026-10-30").unwrap();
    // Act
    let error =
        Policy::from_pyproject("[tool.taloon]\nschema-version=2\nallow=[]", date).unwrap_err();
    // Assert
    assert_eq!(error.code, Code::Configuration);
    assert_eq!(error.message, "unsupported policy schema version");
}

#[test]
fn duplicate_clarification_hash_cannot_replace_the_first_decision() {
    // Arrange
    let date = EvaluationDate::parse("2026-10-30").unwrap();
    let hash = "11".repeat(32);
    let text = format!(
        "[tool.taloon]\nschema-version=1\nallow=[]\n[[tool.taloon.clarifications]]\nartifact-hash='sha256:{hash}'\nexpression='MIT'\nreason='first'\n[[tool.taloon.clarifications]]\nartifact-hash='sha256:{hash}'\nexpression='ISC'\nreason='second'"
    );
    // Act
    let error = Policy::from_pyproject(&text, date).unwrap_err();
    // Assert
    assert_eq!(error.message, "duplicate clarification hash");
}

#[rstest]
#[case::multiple_source_kinds(
    "[[package]]\nname='example'\nversion='1'\nsource={path='example.whl',url='https://example.test/a.whl'}",
    "a package source must have one recognized kind"
)]
#[case::empty_source(
    "[[package]]\nname='example'\nversion='1'\nsource={}",
    "a package source must have one recognized kind"
)]
#[case::duplicate_package(
    "[[package]]\nname='example'\nversion='1'\nsource={path='a.whl'}\n[[package]]\nname='example'\nversion='1'\nsource={path='a.whl'}",
    "duplicate locked package identity"
)]
#[case::ambiguous_member(
    "[manifest]\nmembers=['example']\n[[package]]\nname='example'\nversion='1'\nsource={editable='one'}\n[[package]]\nname='example'\nversion='2'\nsource={editable='two'}",
    "ambiguous workspace membership"
)]
#[case::duplicate_extra(
    "[[package]]\nname='example'\nversion='1'\nsource={virtual='.'}\n[package.optional-dependencies]\nMy_Extra=[]\nmy-extra=[]",
    "duplicate normalized extra"
)]
fn ambiguous_lock_structure_is_rejected(#[case] body: &str, #[case] message: &str) {
    // Arrange
    let lock = format!("version=1\nrevision=5\n{body}");
    // Act
    let error = taloon::resolution::Resolution::from_uv_lock(&lock)
        .err()
        .unwrap();
    // Assert
    assert_eq!(error.code, Code::LockFormat);
    assert_eq!(error.message, message);
}

#[test]
fn blank_clarification_reason_cannot_authorize_an_override() {
    // Arrange
    let date = EvaluationDate::parse("2026-10-30").unwrap();
    let text = format!(
        "[tool.taloon]\nschema-version=1\nallow=[]\n[[tool.taloon.clarifications]]\nartifact-hash='sha256:{}'\nexpression='MIT'\nreason='  '",
        "11".repeat(32)
    );
    // Act
    let error = Policy::from_pyproject(&text, date).unwrap_err();
    // Assert
    assert_eq!(error.code, Code::Configuration);
    assert_eq!(error.message, "override reason must not be blank");
}
