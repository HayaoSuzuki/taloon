//! Public identity and policy boundary regressions.
use rstest::rstest;
use taloon::model::{
    ArtifactFormat, ArtifactLocator, ArtifactRequest, Code, ContentHash, EvaluationDate, PackageId,
    SourceId,
};
use taloon::policy::{LicenseExpression, Policy};
use taloon::resolution::Resolution;

#[rstest]
#[case::missing_algorithm("aabb")]
#[case::wrong_algorithm("sha512:aaaaaaaa")]
#[case::short("sha256:ab")]
#[case::non_hex("sha256:zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz")]
fn invalid_content_hash_cannot_bind_evidence(#[case] value: &str) {
    // Act
    let error = ContentHash::parse(value).unwrap_err();
    // Assert
    assert_eq!(error.code, Code::Configuration);
}

#[rstest]
#[case::empty("")]
#[case::non_calendar("2026-02-30")]
fn date_rejects_non_calendar_input(#[case] value: &str) {
    // Act
    let error = EvaluationDate::parse(value).unwrap_err();
    // Assert
    assert_eq!(error.code, Code::Configuration);
}

#[test]
fn empty_source_location_cannot_be_an_identity() {
    // Act
    let error = SourceId::parse("path", "").unwrap_err();
    // Assert
    assert_eq!(error.message, "empty source location");
}

#[test]
fn malformed_package_version_cannot_be_an_identity() {
    // Arrange
    let source = SourceId::parse("path", "example.whl").unwrap();
    // Act
    let error = PackageId::parse("example", "not a version", source).unwrap_err();
    // Assert
    assert_eq!(error.message, "invalid Python version");
}

#[test]
fn diagnostic_identity_views_redact_credentials() {
    // Arrange
    let source = SourceId::parse(
        "registry",
        "https://alice:password@example.test/simple?token=secret",
    )
    .unwrap();
    let package = PackageId::parse("example", "1", source.clone()).unwrap();
    let locator = ArtifactLocator::registry(
        "https://alice:password@example.test/simple",
        "https://alice:password@example.test/a.whl?token=secret",
    )
    .unwrap();
    // Act
    let display = format!("{source} {source:?} {package} {locator:?}");
    // Assert
    assert!(!display.contains("password"));
    assert!(!display.contains("secret"));
    assert!(!display.contains("alice"));
    assert!(display.contains("example.test"));
}

#[test]
fn local_locator_diagnostic_retains_path() {
    // Arrange
    let locator = ArtifactLocator::local("wheels/example.whl").unwrap();
    // Act
    let display = format!("{locator:?}");
    // Assert
    assert_eq!(display, "Local(\"wheels/example.whl\")");
    assert_eq!(locator.display(), "wheels/example.whl");
}

#[test]
fn direct_request_revalidates_registry_index() {
    // Arrange
    let package =
        PackageId::parse("example", "1", SourceId::parse("path", "a.whl").unwrap()).unwrap();
    let locator = ArtifactLocator::Remote {
        index: Some("file:///tmp/index".into()),
        url: "https://example.test/a.whl".into(),
    };
    // Act
    let error = ArtifactRequest::new(
        package,
        locator,
        ArtifactFormat::Wheel,
        Some(ContentHash::digest(b"a")),
    )
    .unwrap_err();
    // Assert
    assert_eq!(error.code, Code::UnsupportedSource);
}

#[rstest]
#[case::invalid_toml("[tool", "invalid pyproject.toml")]
#[case::missing_policy("[project]\nname='example'", "missing [tool.taloon] policy")]
#[case::both_modes(
    "[tool.taloon]\nschema-version=1\nallow=[]\ndeny=[]",
    "exactly one of allow or deny is required"
)]
#[case::neither_mode(
    "[tool.taloon]\nschema-version=1",
    "exactly one of allow or deny is required"
)]
fn invalid_policy_configuration_is_rejected(#[case] text: &str, #[case] message: &str) {
    // Arrange
    let date = EvaluationDate::parse("2026-11-12").unwrap();
    // Act
    let error = Policy::from_pyproject(text, date).unwrap_err();
    // Assert
    assert_eq!(error.code, Code::Configuration);
    assert_eq!(error.message, message);
}

#[test]
fn invalid_clarification_expression_is_configuration_error() {
    // Arrange
    let date = EvaluationDate::parse("2026-11-12").unwrap();
    let text = format!(
        "[tool.taloon]\nschema-version=1\nallow=[]\n[[tool.taloon.clarifications]]\nartifact-hash='{}'\nexpression='invalid-license'\nreason='reviewed'",
        ContentHash::digest(b"a")
    );
    // Act
    let error = Policy::from_pyproject(&text, date).unwrap_err();
    // Assert
    assert_eq!(error.message, "invalid clarification expression");
}

#[test]
fn expression_debug_retains_original_spelling() {
    // Arrange
    let expression = LicenseExpression::parse("MIT OR ISC").unwrap();
    // Act
    let debug = format!("{expression:?}");
    // Assert
    assert_eq!(debug, "LicenseExpression(\"MIT OR ISC\")");
}

#[rstest]
#[case::missing_version(
    "[[package]]\nname='example'\nsource={path='a.whl'}",
    "external package lacks version"
)]
#[case::invalid_constraint(
    "[[package]]\nname='root'\nsource={virtual='.'}\ndependencies=[{name='example',specifier='not-valid'}]\n[[package]]\nname='example'\nversion='1'\nsource={path='a.whl'}",
    "invalid locked origin constraint"
)]
fn invalid_locked_identity_is_rejected(#[case] body: &str, #[case] message: &str) {
    // Arrange
    let lock = format!("version=1\nrevision=5\n{body}");
    // Act
    let error = Resolution::from_uv_lock(&lock).err().unwrap();
    // Assert
    assert_eq!(error.code, Code::LockFormat);
    assert_eq!(error.message, message);
}

#[rstest]
#[case::registry_without_url(
    "source={registry='https://example.test/simple'}\nwheels=[{}]",
    Code::MissingArtifact,
    "registry artifact lacks URL"
)]
#[case::invalid_hash(
    "source={url='https://example.test/a.whl'}\nwheels=[{hash='sha256:bad'}]",
    Code::Configuration,
    "invalid SHA-256 digest"
)]
#[case::no_archive(
    "source={registry='https://example.test/simple'}",
    Code::MissingArtifact,
    "package has no recorded distribution archives"
)]
#[case::duplicate_archive(
    "source={path='a.whl'}\nwheels=[{path='a.whl'},{path='a.whl'}]",
    Code::LockFormat,
    "duplicate artifact association"
)]
fn unsupported_archive_observation_remains_explicit_target(
    #[case] body: &str,
    #[case] code: Code,
    #[case] message: &str,
) {
    // Arrange
    let lock = format!("version=1\nrevision=5\n[[package]]\nname='example'\nversion='1'\n{body}");
    // Act
    let resolution = Resolution::from_uv_lock(&lock).unwrap();
    // Assert
    assert_eq!(resolution.targets().len(), 1);
    assert_eq!(resolution.targets()[0].issues()[0].code, code);
    assert_eq!(resolution.targets()[0].issues()[0].message, message);
}

#[rstest]
#[case::wheel("example.WHL", ArtifactFormat::Wheel)]
#[case::sdist("example.tar.gz", ArtifactFormat::Sdist)]
fn path_only_archive_infers_container_format(#[case] path: &str, #[case] format: ArtifactFormat) {
    // Arrange
    let lock = format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1'\nsource={{path='{path}'}}"
    );
    // Act
    let resolution = Resolution::from_uv_lock(&lock).unwrap();
    // Assert
    let target = &resolution.targets()[0];
    assert_eq!(target.issues(), []);
    assert_eq!(target.requests()[0].format(), format);
    assert_eq!(target.requests()[0].locator().display(), path);
}

#[test]
fn description_graph_deduplicates_edges_and_sorts_origin_edges() {
    // Arrange
    let lock = "version=1\nrevision=5\n[[package]]\nname='root'\nsource={virtual='.'}\ndependencies=[{name='zeta'},{name='alpha'},{name='alpha'}]\n[[package]]\nname='zeta'\nversion='1'\nsource={path='zeta.whl'}\n[[package]]\nname='alpha'\nversion='1'\nsource={path='alpha.whl'}";
    let description = taloon::description::Description::from_uv_lock(lock, "root").unwrap();
    // Act
    let report = description.evaluate(vec![]);
    // Assert
    assert_eq!(report.graph().edges.len(), 2);
    assert_eq!(report.graph().origins.len(), 1);
    let origin = &report.graph().origins[0];
    assert_eq!(origin.entry, "root");
    assert_eq!(origin.edges.len(), 3);
    assert!(origin.edges.windows(2).all(|pair| pair[0].to <= pair[1].to));
}

#[test]
fn external_selection_terminates_on_dependency_cycle() {
    // Arrange
    let lock = "version=1\nrevision=5\n[[package]]\nname='alpha'\nversion='1'\nsource={path='alpha.whl'}\ndependencies=[{name='beta'}]\n[[package]]\nname='beta'\nversion='1'\nsource={path='beta.whl'}\ndependencies=[{name='alpha'}]";
    let description = taloon::description::Description::from_uv_lock(lock, "alpha").unwrap();
    // Act
    let report = description.evaluate(vec![]);
    // Assert
    assert_eq!(report.graph().nodes.len(), 2);
    assert_eq!(report.graph().edges.len(), 2);
    assert_eq!(report.packages().len(), 2);
}

#[test]
fn source_selector_disambiguates_same_name_locked_forks() {
    // Arrange
    let lock = "version=1\nrevision=5\n[[package]]\nname='root'\nsource={virtual='.'}\ndependencies=[{name='example',source={path='second.whl'}}]\n[[package]]\nname='example'\nversion='1'\nsource={path='first.whl'}\n[[package]]\nname='example'\nversion='1'\nsource={path='second.whl'}";
    // Act
    let resolution = Resolution::from_uv_lock(lock).unwrap();
    // Assert
    let first = resolution
        .targets()
        .iter()
        .find(|target| target.package().source().location() == "first.whl")
        .unwrap();
    let second = resolution
        .targets()
        .iter()
        .find(|target| target.package().source().location() == "second.whl")
        .unwrap();
    assert_eq!(resolution.paths(first.package()).len(), 0);
    assert_eq!(resolution.paths(second.package()).len(), 1);
}

#[test]
fn false_version_constraint_does_not_select_other_locked_fork() {
    // Arrange
    let lock = "version=1\nrevision=5\n[[package]]\nname='root'\nsource={virtual='.'}\ndependencies=[{name='example',specifier='>=2'}]\n[[package]]\nname='example'\nversion='1'\nsource={path='first.whl'}\n[[package]]\nname='example'\nversion='2'\nsource={path='second.whl'}";
    // Act
    let resolution = Resolution::from_uv_lock(lock).unwrap();
    // Assert
    let first = resolution
        .targets()
        .iter()
        .find(|target| target.package().version() == "1")
        .unwrap();
    let second = resolution
        .targets()
        .iter()
        .find(|target| target.package().version() == "2")
        .unwrap();
    assert_eq!(resolution.paths(first.package()).len(), 0);
    assert_eq!(resolution.paths(second.package()).len(), 1);
}

#[test]
fn remote_source_url_supplies_missing_archive_url() {
    // Arrange
    let hash = ContentHash::digest(b"archive");
    let lock = format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1'\nsource={{url='https://example.test/example.tar.gz'}}\nsdist={{hash='{hash}'}}"
    );
    // Act
    let resolution = Resolution::from_uv_lock(&lock).unwrap();
    // Assert
    let target = &resolution.targets()[0];
    assert_eq!(target.issues(), []);
    assert_eq!(
        target.requests()[0].locator().display(),
        "https://example.test/example.tar.gz"
    );
    assert_eq!(target.requests()[0].format(), ArtifactFormat::Sdist);
}

#[test]
fn archive_proof_binding_is_independent_of_fetch_locator() {
    // Arrange
    let proof = taloon_decisions::evidence::ProofBinding::ArchiveVerified;
    // Act
    let reusable = proof.matches(&[1; 32], &[2; 32]);
    // Assert
    assert!(reusable);
}

#[rstest]
#[case::invalid_extra(
    "version=1\nrevision=5\n[[package]]\nname='root'\nsource={virtual='.'}\ndependencies=[{name='example',extra=['?']}]\n[[package]]\nname='example'\nversion='1'\nsource={path='a.whl'}",
    "invalid Python package name"
)]
#[case::invalid_member(
    "version=1\nrevision=5\n[manifest]\nmembers=['?']",
    "invalid Python package name"
)]
#[case::malformed_toml("[", "invalid uv.lock fields")]
fn malformed_graph_identity_is_rejected(#[case] lock: &str, #[case] message: &str) {
    // Act
    let error = Resolution::from_uv_lock(lock).err().unwrap();
    // Assert
    assert_eq!(error.message, message);
}

#[test]
fn rejected_manifest_hint_does_not_remove_locked_target() {
    // Arrange
    let lock = "version=1\nrevision=5\n[manifest.dependency-groups]\ndev=[{name='missing'}]\n[[package]]\nname='example'\nversion='1'\nsource={path='a.whl'}";
    // Act
    let resolution = Resolution::from_uv_lock(lock).unwrap();
    // Assert
    assert_eq!(resolution.targets().len(), 1);
    assert_eq!(resolution.targets()[0].package().name(), "example");
    assert_eq!(resolution.paths(resolution.targets()[0].package()).len(), 0);
}

#[test]
fn malformed_registry_archive_url_is_a_target_issue() {
    // Arrange
    let hash = ContentHash::digest(b"archive");
    let lock = format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1'\nsource={{registry='https://example.test/simple'}}\nwheels=[{{url='file:///tmp/archive.whl',hash='{hash}'}}]"
    );
    // Act
    let resolution = Resolution::from_uv_lock(&lock).unwrap();
    // Assert
    assert_eq!(
        resolution.targets()[0].issues()[0].code,
        Code::UnsupportedSource
    );
    assert_eq!(resolution.targets()[0].requests().len(), 0);
}
