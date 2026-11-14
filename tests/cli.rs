mod common;
use std::{
    path::Path,
    process::{Command, Output},
};
fn invoke(root: &Path, cache: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_taloon"))
        .current_dir(root)
        .env("TALOON_CACHE_DIR", cache)
        .env_remove("NETRC")
        .args(args)
        .output()
        .unwrap()
}
fn command_arguments(command: &str) -> Vec<&str> {
    match command {
        "describe" => vec!["describe", "example"],
        _ => vec![command],
    }
}
fn json(output: &Output) -> serde_json::Value {
    let mut objects =
        serde_json::Deserializer::from_slice(&output.stdout).into_iter::<serde_json::Value>();
    let value = objects
        .next()
        .expect("report missing")
        .expect("report JSON");
    assert!(objects.next().is_none(), "stdout has extra data");
    assert_eq!(value["schema_version"], 1);
    value
}
fn policy(root: &Path, allow: &str) {
    std::fs::write(
        root.join("pyproject.toml"),
        format!("[tool.taloon]\nschema-version=1\nallow=[{allow}]\n"),
    )
    .unwrap();
}
fn project(root: &Path, fields: &str) {
    let bytes = common::wheel(&common::metadata(fields));
    std::fs::write(root.join("example.whl"), &bytes).unwrap();
    std::fs::write(root.join("uv.lock"),"version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={path='example.whl'}").unwrap();
    policy(root, "'MIT'");
}
#[rstest::rstest]
#[case::nearest(vec!["check", "--offline", "--format=json"], 0, 1)]
#[case::explicit_member(vec!["check", "--project", ".", "--format=json"], 2, 0)]
fn cli_project_discovery(#[case] args: Vec<&str>, #[case] expected: i32, #[case] packages: usize) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    project(temp.path(), "License-Expression: MIT");
    let member = temp.path().join("members/a");
    std::fs::create_dir_all(&member).unwrap();
    // Act
    let output = invoke(&member, &temp.path().join("cache"), &args);
    // Assert
    assert_eq!(output.status.code(), Some(expected));
    assert_eq!(
        json(&output)["packages"].as_array().unwrap().len(),
        packages
    );
}

#[test]
fn nearest_lock_requires_its_own_sibling_policy() {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    project(temp.path(), "License-Expression: MIT");
    let member = temp.path().join("members/a");
    std::fs::create_dir_all(&member).unwrap();
    std::fs::write(member.join("uv.lock"), "version=1\nrevision=5").unwrap();
    // Act
    let output = invoke(
        &member,
        &temp.path().join("cache"),
        &["check", "--offline", "--format=json"],
    );
    // Assert
    assert_eq!(output.status.code(), Some(2));
    json(&output);
}

#[test]
fn empty_inventory_with_sibling_policy_succeeds() {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("uv.lock"), "version=1\nrevision=5").unwrap();
    policy(temp.path(), "");
    // Act
    let output = invoke(
        temp.path(),
        &temp.path().join("cache"),
        &["check", "--offline", "--format=json"],
    );
    // Assert
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(json(&output)["packages"], serde_json::json!([]));
}

#[rstest::rstest]
#[case::allowed("'MIT'", 0)]
#[case::denied("", 1)]
fn cli_complete_json_report(#[case] allow: &str, #[case] expected: i32) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    project(temp.path(), "License-Expression: MIT");
    policy(temp.path(), allow);
    // Act
    let output = invoke(
        temp.path(),
        &temp.path().join("cache"),
        &["check", "--offline", "--format=json"],
    );
    // Assert
    assert_eq!(output.status.code(), Some(expected));
    assert_eq!(json(&output)["packages"].as_array().unwrap().len(), 1);
    assert!(
        output.stderr.is_empty(),
        "piped reports must not emit terminal progress"
    );
}

#[test]
fn cli_missing_archive_is_incomplete() {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    project(temp.path(), "License-Expression: MIT");
    std::fs::remove_file(temp.path().join("example.whl")).unwrap();
    // Act
    let output = invoke(
        temp.path(),
        &temp.path().join("cache"),
        &["check", "--offline", "--format=json"],
    );
    // Assert
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(json(&output)["status"], "incomplete");
}

#[test]
fn cli_unknown_argument_preserves_json_output() {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    project(temp.path(), "License-Expression: MIT");
    // Act
    let output = invoke(
        temp.path(),
        &temp.path().join("cache"),
        &["check", "--format=json", "--unknown"],
    );
    // Assert
    assert_eq!(output.status.code(), Some(2));
    json(&output);
}

#[rstest::rstest]
#[case::revoked("", vec!["check", "--offline", "--format=json"])]
#[case::unused_permission("'MIT','ISC'", vec!["check", "--offline", "--format=json", "--deny-warnings"])]
fn cli_offline_policy_change_rechecks_cached_evidence(
    #[case] allow: &str,
    #[case] args: Vec<&str>,
) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    project(root, "License-Expression: MIT");
    let cache = root.join("cache");
    let allowed = invoke(root, &cache, &["check", "--format", "json"]);
    assert_eq!(allowed.status.code(), Some(0));
    assert_eq!(json(&allowed)["acquisition"]["archive_extractions"], 1);
    policy(root, allow);
    // Act
    let denied = invoke(root, &cache, &args);
    // Assert
    assert_eq!(denied.status.code(), Some(1));
    let report = json(&denied);
    assert_eq!(report["acquisition"]["archive_extractions"], 0);
    assert_eq!(report["acquisition"]["network_requests"], 0);
    assert_eq!(report["acquisition"]["cache_hits"], 1);
}

#[cfg(unix)]
#[test]
fn cli_never_resolves_builds_or_mutates_inputs() {
    use std::{io::Cursor, os::unix::fs::PermissionsExt};
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let marker = root.join("executed");
    let meta = common::metadata("License-Expression: MIT");
    let mut tar = tar::Builder::new(flate2::write::GzEncoder::new(
        Vec::new(),
        flate2::Compression::fast(),
    ));
    let code = format!(
        "from pathlib import Path\nPath({:?}).write_text('executed')\n",
        marker.to_string_lossy()
    );
    for (name, bytes) in [
        ("example-1.0/PKG-INFO", meta.as_slice()),
        ("example-1.0/setup.py", code.as_bytes()),
    ] {
        let mut header = tar::Header::new_gnu();
        header.set_mode(0o755);
        header.set_size(u64::try_from(bytes.len()).unwrap());
        header.set_cksum();
        tar.append_data(&mut header, name, Cursor::new(bytes))
            .unwrap();
    }
    std::fs::write(
        root.join("example.tar.gz"),
        tar.into_inner().unwrap().finish().unwrap(),
    )
    .unwrap();
    std::fs::write(root.join("uv.lock"),"version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={path='example.tar.gz'}").unwrap();
    policy(root, "'MIT'");
    let before_lock = std::fs::read(root.join("uv.lock")).unwrap();
    let before_config = std::fs::read(root.join("pyproject.toml")).unwrap();
    let bin = root.join("bin");
    std::fs::create_dir(&bin).unwrap();
    for executable in ["uv", "python", "python3"] {
        let path = bin.join(executable);
        std::fs::write(
            &path,
            format!(
                "#!/bin/sh\necho executed > '{}'\nexit 97\n",
                marker.display()
            ),
        )
        .unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let output = Command::new(env!("CARGO_BIN_EXE_taloon"))
        .current_dir(root)
        .env("PATH", bin)
        .env("TALOON_CACHE_DIR", root.join("cache"))
        .args(["check", "--offline", "--format", "json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    json(&output);
    assert!(!marker.exists());
    assert_eq!(before_lock, std::fs::read(root.join("uv.lock")).unwrap());
    assert_eq!(
        before_config,
        std::fs::read(root.join("pyproject.toml")).unwrap()
    );
}
#[test]
fn cli_redacts_locator_secrets() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    policy(root, "'MIT'");
    let hash = taloon::model::ContentHash::digest(b"missing");
    let lock = format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={{url='https://user:password@files.test/a.whl?signature=signed-secret'}}\nwheels=[{{url='https://user:password@files.test/a.whl?signature=signed-secret',hash='{hash}'}}]"
    );
    std::fs::write(root.join("uv.lock"), lock).unwrap();
    let output = invoke(
        root,
        &root.join("cache"),
        &["check", "--offline", "--format", "json"],
    );
    assert_eq!(output.status.code(), Some(2));
    json(&output);
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(!text.contains("password"));
    assert!(!text.contains("signed-secret"));
}

#[test]
fn cli_escapes_terminal_controls() {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    policy(root, "'MIT'");
    std::fs::write(root.join("uv.lock"),"version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={path=\"evil\\u001b[31m.whl\"}").unwrap();
    let output = invoke(root, &root.join("cache"), &["check", "--offline"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(!output.stdout.contains(&27));
}
#[rstest::rstest]
#[case::online(vec!["check", "--format=json"], 2)]
#[case::offline(vec!["check", "--offline", "--format=json"], 0)]
fn cli_missing_netrc(#[case] args: Vec<&str>, #[case] expected: i32) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    project(temp.path(), "License-Expression: MIT");
    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_taloon"))
        .current_dir(temp.path())
        .env("NETRC", temp.path().join("missing-netrc"))
        .env("TALOON_CACHE_DIR", temp.path().join("cache"))
        .args(args)
        .output()
        .unwrap();
    // Assert
    assert_eq!(output.status.code(), Some(expected));
    json(&output);
}
proptest::proptest! {
 #![proptest_config(proptest::test_runner::Config::with_cases(32))]
 #[test]
 fn cli_generated_verdict_exit_and_report_agree(allow in proptest::bool::ANY,missing in proptest::bool::ANY,warnings in proptest::bool::ANY,deny in proptest::bool::ANY){
 let temp=tempfile::tempdir().unwrap();let root=temp.path();project(root,"License-Expression: MIT");let mut units=Vec::new();if allow{units.push("'MIT'");}
 if warnings{units.push("'ISC'");}policy(root,&units.join(","));if missing{std::fs::remove_file(root.join("example.whl")).unwrap();}
 let mut args=vec!["check","--offline","--format","json"];if deny{args.push("--deny-warnings");}let output=invoke(root,&root.join("cache"),&args);let report=json(&output);let expected=if missing{2}else{u8::from(!allow||(warnings&&deny))};
 proptest::prop_assert_eq!(output.status.code(),Some(i32::from(expected)));proptest::prop_assert_eq!(report["exit_code"].as_u64(),Some(u64::from(expected)));
 }
}

#[rstest::rstest]
#[case::allow(1)]
#[case::deny(2)]
#[case::scoped_overrides(3)]
fn documented_policy_uses_the_implemented_schema(#[case] example: usize) {
    // Arrange
    let guide = include_str!("../docs/user/configuration.md");
    let date = taloon::model::EvaluationDate::parse("2026-10-31").unwrap();
    let code = guide
        .split("```toml\n")
        .nth(example)
        .unwrap()
        .split("```")
        .next()
        .unwrap();
    // Act
    let policy = taloon::policy::Policy::from_pyproject(code, date);
    // Assert
    assert!(policy.is_ok(), "{policy:?}");
}

#[test]
fn text_failures_explain_every_relevant_license_unit() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    project(root, "License-Expression: MIT AND (Apache-2.0 OR ISC)");
    let output = invoke(root, &root.join("cache"), &["check", "--offline"]);
    assert_eq!(output.status.code(), Some(1));
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("permitted unit: MIT"));
    assert!(text.contains("denied unit: Apache-2.0"));
    assert!(text.contains("denied unit: ISC"));
}

#[test]
fn cache_clear_removes_only_taloon_records_without_project_discovery() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    project(root, "License-Expression: MIT");
    let cache = root.join("shared");
    let first = invoke(root, &cache, &["check", "--format", "json"]);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(json(&first)["acquisition"]["archive_extractions"], 1);
    let damaged = format!(
        "v1-index-sha256-{}-sdist-sha256-{}.json",
        "a".repeat(64),
        "b".repeat(64)
    );
    std::fs::write(cache.join(&damaged), b"damaged").unwrap();
    for name in [
        "notes.json",
        "v1-archive-wheel-sha256-invalid.json",
        ".evidence-unfinished",
        "v2-archive-wheel-sha256-unknown.json",
    ] {
        std::fs::write(cache.join(name), b"keep").unwrap();
    }
    let directory = cache.join(format!("v1-archive-sdist-sha256-{}.json", "c".repeat(64)));
    std::fs::create_dir(&directory).unwrap();
    std::fs::write(directory.join("keep"), b"keep").unwrap();
    let outside = tempfile::tempdir().unwrap();
    let output = invoke(outside.path(), &cache, &["cache", "clear"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stdout, b"Removed 2 taloon cache entries.\n");
    assert_eq!(output.stderr, b"");
    assert!(cache.is_dir());
    assert!(!cache.join(damaged).exists());
    let retained: Vec<_> = [
        "notes.json",
        "v1-archive-wheel-sha256-invalid.json",
        ".evidence-unfinished",
        "v2-archive-wheel-sha256-unknown.json",
    ]
    .map(|name| std::fs::read(cache.join(name)).unwrap())
    .into_iter()
    .collect();
    assert_eq!(retained, vec![b"keep".to_vec(); 4]);
    assert_eq!(std::fs::read(directory.join("keep")).unwrap(), b"keep");
}

#[test]
fn cache_clear_is_idempotent() {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    project(temp.path(), "License-Expression: MIT");
    let cache = temp.path().join("cache");
    assert_eq!(
        invoke(temp.path(), &cache, &["check"]).status.code(),
        Some(0)
    );
    assert_eq!(
        invoke(temp.path(), &cache, &["cache", "clear"])
            .status
            .code(),
        Some(0)
    );
    // Act
    let output = invoke(temp.path(), &cache, &["cache", "clear"]);
    // Assert
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"Removed 0 taloon cache entries.\n");
}

#[test]
fn cache_clear_forces_archive_reextraction() {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    project(temp.path(), "License-Expression: MIT");
    let cache = temp.path().join("cache");
    assert_eq!(
        invoke(temp.path(), &cache, &["check"]).status.code(),
        Some(0)
    );
    assert_eq!(
        invoke(temp.path(), &cache, &["cache", "clear"])
            .status
            .code(),
        Some(0)
    );
    // Act
    let output = invoke(temp.path(), &cache, &["check", "--format=json"]);
    // Assert
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(json(&output)["acquisition"]["archive_extractions"], 1);
    assert_eq!(json(&output)["acquisition"]["cache_hits"], 0);
}

#[test]
fn cache_clear_preserves_missing_cache() {
    let temp = tempfile::tempdir().unwrap();
    let cache = temp.path().join("missing");
    let output = invoke(temp.path(), &cache, &["cache", "clear"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stdout, b"Removed 0 taloon cache entries.\n");
    assert!(!cache.exists());
}

#[test]
fn cache_clear_rejects_file_root() {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let cache = temp.path().join("cache-file");
    std::fs::write(&cache, b"keep").unwrap();
    let output = invoke(temp.path(), &cache, &["cache", "clear"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"cannot clear taloon cache\n");
    assert_eq!(std::fs::read(&cache).unwrap(), b"keep");
}

#[cfg(unix)]
#[test]
fn cache_clear_preserves_symlink_entries() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    let cache = temp.path().join("cache");
    std::fs::create_dir(&cache).unwrap();
    let name = format!("v1-archive-wheel-sha256-{}.json", "d".repeat(64));
    let target = temp.path().join("keep");
    std::fs::write(&target, b"keep").unwrap();
    symlink(&target, cache.join(&name)).unwrap();
    let output = invoke(temp.path(), &cache, &["cache", "clear"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stdout, b"Removed 0 taloon cache entries.\n");
    assert!(cache.join(&name).is_symlink());
    assert_eq!(std::fs::read(&target).unwrap(), b"keep");
}

#[cfg(unix)]
#[test]
fn cache_clear_rejects_symlink_root() {
    use std::os::unix::fs::symlink;
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let cache = temp.path().join("cache");
    std::fs::create_dir(&cache).unwrap();
    let target = cache.join("keep");
    std::fs::write(&target, b"keep").unwrap();
    let root_link = temp.path().join("linked-cache");
    symlink(&cache, &root_link).unwrap();
    let output = invoke(temp.path(), &root_link, &["cache", "clear"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stderr, b"cannot clear taloon cache\n");
    assert!(root_link.is_symlink());
    assert_eq!(std::fs::read(&target).unwrap(), b"keep");
}

#[test]
fn cache_clear_uses_default_location_and_ignores_project_and_credentials() {
    let temp = tempfile::tempdir().unwrap();
    let base = temp.path().join("base");
    #[cfg(target_os = "macos")]
    let cache = temp.path().join("Library/Caches/taloon");
    #[cfg(not(target_os = "macos"))]
    let cache = base.join("taloon");
    std::fs::create_dir_all(&cache).unwrap();
    let record = cache.join(format!("v1-archive-wheel-sha256-{}.json", "0".repeat(64)));
    std::fs::write(&record, b"damaged").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_taloon"))
        .current_dir(temp.path())
        .env_remove("TALOON_CACHE_DIR")
        .env("HOME", temp.path())
        .env("XDG_CACHE_HOME", &base)
        .env("LOCALAPPDATA", &base)
        .env("NETRC", temp.path().join("missing-credentials"))
        .args(["cache", "clear"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stdout, b"Removed 1 taloon cache entries.\n");
    assert!(!record.exists());
}

#[test]
fn cache_clear_empty_override_matches_existing_cache_writes() {
    let temp = tempfile::tempdir().unwrap();
    project(temp.path(), "License-Expression: MIT");
    let first = invoke(temp.path(), Path::new(""), &["check", "--format", "json"]);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(json(&first)["acquisition"]["cache_hits"], 0);
    let output = invoke(temp.path(), Path::new(""), &["cache", "clear"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"Removed 1 taloon cache entries.\n");
    assert!(temp.path().join("uv.lock").is_file());
    assert!(temp.path().join("pyproject.toml").is_file());
}

#[cfg(unix)]
#[rstest::rstest]
#[case::slash("/")]
#[case::dot("/.")]
#[case::dot_slash("/./")]
#[case::double_slash("//")]
fn cache_clear_rejects_root_symlink_with_redundant_components(#[case] suffix: &str) {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    let cache = temp.path().join("cache");
    std::fs::create_dir(&cache).unwrap();
    let record = cache.join(format!("v1-archive-wheel-sha256-{}.json", "e".repeat(64)));
    std::fs::write(&record, b"keep").unwrap();
    let root_link = temp.path().join("linked-cache");
    symlink(&cache, &root_link).unwrap();
    let spelling = std::path::PathBuf::from(format!("{}{suffix}", root_link.display()));
    let output = invoke(temp.path(), &spelling, &["cache", "clear"]);
    assert_eq!(output.status.code(), Some(2), "suffix {suffix}: {output:?}");
    assert_eq!(output.stderr, b"cannot clear taloon cache\n");
    assert!(root_link.is_symlink());
    assert_eq!(std::fs::read(&record).unwrap(), b"keep");
}

fn describe_project(root: &Path) {
    let mit = common::wheel(&common::metadata("License-Expression: MIT"));
    let bsd = common::wheel(&common::metadata("License-Expression: BSD-3-Clause"));
    std::fs::write(root.join("mit.whl"), mit).unwrap();
    std::fs::write(root.join("bsd.whl"), bsd).unwrap();
    std::fs::write(root.join("uv.lock"),"version=1\nrevision=5\n[[package]]\nname='app'\nversion='1.0'\nsource={virtual='.'}\ndependencies=[{name='example'}]\n[[package]]\nname='example'\nversion='1.0'\nsource={path='mit.whl'}\nwheels=[{path='mit.whl'},{path='bsd.whl'}]\n[[package]]\nname='unrelated'\nversion='1.0'\nsource={path='does-not-exist.whl'}").unwrap();
}
fn cached_description(root: &Path) -> std::path::PathBuf {
    describe_project(root);
    let cache = root.join("cache");
    let output = invoke(root, &cache, &["describe", "example", "--format=json"]);
    assert_eq!(output.status.code(), Some(0), "cache fixture: {output:?}");
    cache
}

#[test]
fn describe_cli_selects_all_artifacts_without_a_policy() {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    describe_project(root);
    // Act
    let output = invoke(
        root,
        &root.join("cache"),
        &["describe", "EXAMPLE", "--format=json"],
    );
    // Assert
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let report = json(&output);
    assert_eq!(report["command"], "describe");
    assert_eq!(report["query"], "example");
    assert_eq!(report["packages"].as_array().unwrap().len(), 1);
    assert_eq!(
        report["packages"][0]["licenses"].as_array().unwrap().len(),
        2
    );
    assert_eq!(report["acquisition"]["archive_extractions"], 2);
    assert_eq!(output.stderr, b"");
}

#[test]
fn describe_cli_ignores_malformed_policy_with_cached_evidence() {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let cache = cached_description(root);
    std::fs::write(root.join("pyproject.toml"), "not valid toml").unwrap();
    // Act
    let output = invoke(
        root,
        &cache,
        &["describe", "example", "--offline", "--format=json"],
    );
    // Assert
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(json(&output)["acquisition"]["cache_hits"], 2);
    assert_eq!(json(&output)["acquisition"]["network_requests"], 0);
}

#[test]
fn describe_cli_text_lists_licenses_without_a_verdict() {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let cache = cached_description(temp.path());
    // Act
    let output = invoke(temp.path(), &cache, &["describe", "example", "--offline"]);
    // Assert
    assert_eq!(output.status.code(), Some(0));
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("MIT"));
    assert!(text.contains("BSD-3-Clause"));
    assert!(!text.contains("Compliant"));
}

#[test]
fn check_reuses_description_cache_with_full_inventory() {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let cache = cached_description(temp.path());
    policy(temp.path(), "'MIT','BSD-3-Clause'");
    // Act
    let output = invoke(
        temp.path(),
        &cache,
        &["check", "--offline", "--format=json"],
    );
    // Assert: unrelated missing artifact remains in check scope.
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(json(&output)["acquisition"]["cache_hits"], 2);
}

#[test]
fn describe_workspace_selects_whole_external_inventory() {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let cache = cached_description(temp.path());
    // Act
    let output = invoke(
        temp.path(),
        &cache,
        &["describe", "app", "--offline", "--format=json"],
    );
    // Assert
    assert_eq!(output.status.code(), Some(2));
    let report = json(&output);
    assert_eq!(report["selection_mode"], "all_external");
    assert_eq!(report["packages"].as_array().unwrap().len(), 2);
    assert!(
        report["packages"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["name"] != "app")
    );
}

#[rstest::rstest]
#[case::missing_query(vec!["describe", "--format=json"])]
#[case::unknown_query(vec!["describe", "missing", "--format=json"])]
#[case::unknown_argument(vec!["describe", "example", "--unknown", "--format=json"])]
#[case::missing_project(vec!["describe", "example", "--project", "missing", "--format=json"])]
fn describe_cli_reports_its_own_json_errors(#[case] args: Vec<&str>) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    describe_project(temp.path());
    // Act
    let output = invoke(temp.path(), &temp.path().join("cache"), &args);
    // Assert
    assert_eq!(output.status.code(), Some(2));
    let report = json(&output);
    assert_eq!(report["command"], "describe");
    assert_eq!(report["status"], "incomplete");
    assert_eq!(report["acquisition"]["network_requests"], 0);
}

#[rstest::rstest]
#[case::nearest(vec!["describe", "example", "--format=json"], 0)]
#[case::explicit_member(vec!["describe", "example", "--project", ".", "--format=json"], 2)]
fn describe_cli_project_discovery(#[case] args: Vec<&str>, #[case] expected: i32) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    describe_project(temp.path());
    let member = temp.path().join("subdir");
    std::fs::create_dir(&member).unwrap();
    // Act
    let output = invoke(&member, &temp.path().join("cache"), &args);
    // Assert
    assert_eq!(output.status.code(), Some(expected));
    assert_eq!(json(&output)["command"], "describe");
}

fn redacted_description_output(format: &str) -> Output {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let hash = taloon::model::ContentHash::digest(b"archive");
    let lock = format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={{registry='https://alice:password@index.test/simple?token=secret'}}\nresolution-markers=[\"\\u001b[31m\"]\nwheels=[{{url='https://alice:password@files.test/example.whl?token=secret',hash='{hash}'}}]\ndependencies=[{{name='example',marker=\"\\u001b[31m\"}}]"
    );
    std::fs::write(root.join("uv.lock"), lock).unwrap();
    invoke(
        root,
        &root.join("cache"),
        &["describe", "example", "--offline", "--format", format],
    )
}

#[test]
fn describe_text_redacts_sources_and_escapes_conditions() {
    // Arrange
    let format = "text";
    // Act
    let output = redacted_description_output(format);
    // Assert
    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    assert!(!stdout.contains("alice"), "{stdout}");
    assert!(!stdout.contains("password"), "{stdout}");
    assert!(!stdout.contains("secret"), "{stdout}");
    assert!(!stdout.contains('\u{1b}'), "{stdout}");
}

#[test]
fn describe_json_redacts_sources_and_escapes_conditions() {
    // Arrange
    let format = "json";
    // Act
    let output = redacted_description_output(format);
    // Assert
    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    assert!(!stdout.contains("alice"), "{stdout}");
    assert!(!stdout.contains("password"), "{stdout}");
    assert!(!stdout.contains("secret"), "{stdout}");
    assert!(!stdout.contains('\u{1b}'), "{stdout}");
    assert_eq!(json(&output)["command"], "describe");
}

#[test]
fn describe_text_keeps_marker_annotations() {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("uv.lock"), "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={path='missing.whl'}\nresolution-markers=['python_version > 3']\ndependencies=[{name='example',marker='python_version > 3'}]").unwrap();
    // Act
    let output = invoke(
        temp.path(),
        &temp.path().join("cache"),
        &["describe", "example", "--offline"],
    );
    // Assert
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("marker:"));
    assert!(stdout.contains("resolution markers"));
}

#[test]
fn describe_cli_error_query_uses_python_name_normalization() {
    let temp = tempfile::tempdir().unwrap();
    describe_project(temp.path());
    let output = invoke(
        temp.path(),
        &temp.path().join("cache"),
        &["describe", "MISSING_Name", "--format", "json"],
    );
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(json(&output)["query"], "missing-name");
}

#[test]
fn describe_cli_empty_project_is_explicit() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let cache = root.join("cache");
    std::fs::write(
        root.join("uv.lock"),
        "version=1\nrevision=5\n[[package]]\nname='app'\nversion='1.0'\nsource={virtual='.'}",
    )
    .unwrap();
    let output = invoke(
        root,
        &cache,
        &["describe", "app", "--offline", "--format", "json"],
    );
    assert_eq!(output.status.code(), Some(0));
    let report = json(&output);
    assert_eq!(report["packages"], serde_json::json!([]));
    assert_eq!(report["selection_mode"], "all_external");
}

#[test]
fn describe_cli_multiple_source_versions_are_explicit() {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let cache = root.join("cache");
    let hash = taloon::model::ContentHash::digest(b"archive");
    let lock = format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={{registry='https://one.test/simple'}}\nwheels=[{{url='https://one.test/a.whl',hash='{hash}'}}]\n[[package]]\nname='example'\nversion='2.0'\nsource={{registry='https://two.test/simple'}}\nwheels=[{{url='https://two.test/a.whl',hash='{hash}'}}]"
    );
    std::fs::write(root.join("uv.lock"), lock).unwrap();
    let output = invoke(
        root,
        &cache,
        &["describe", "example", "--offline", "--format", "json"],
    );
    assert_eq!(output.status.code(), Some(2));
    let report = json(&output);
    assert_eq!(report["roots"].as_array().unwrap().len(), 2);
    assert_eq!(report["packages"].as_array().unwrap().len(), 2);
    assert_eq!(report["packages"][0]["version"], "1.0");
    assert_eq!(report["packages"][1]["version"], "2.0");
    assert_eq!(
        report["packages"][0]["source"]["location"],
        "https://one.test/simple"
    );
    assert_eq!(
        report["packages"][1]["source"]["location"],
        "https://two.test/simple"
    );
    assert_eq!(report["acquisition"]["network_requests"], 0);
}

#[rstest::rstest]
#[case::check_license("check", "License", "original_license")]
#[case::describe_license("describe", "License", "original_license")]
#[case::check_expression("check", "License-Expression", "original_expression")]
#[case::describe_expression("describe", "License-Expression", "original_expression")]
fn declaration_json_preserves_prose_and_protects_terminal(
    #[case] command: &str,
    #[case] header: &str,
    #[case] field: &str,
) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let declaration = "Copyright <author@example.org> \u{1b}[31m\u{202e} see https://alice:password@licenses.test/license?token=hidden#private";
    project(temp.path(), &format!("{header}: {declaration}"));
    let mut args = command_arguments(command);
    args.extend(["--offline", "--format=json"]);
    // Act
    let output = invoke(temp.path(), &temp.path().join("cache"), &args);
    // Assert
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    assert!(!stdout.contains('\u{1b}'));
    assert!(!stdout.contains('\u{202e}'));
    assert!(!stdout.contains("password"));
    assert!(!stdout.contains("token=hidden"));
    let value = json(&output);
    let actual = value["packages"][0]["artifacts"][0]["evidence"][field][0]
        .as_str()
        .unwrap();
    assert!(actual.contains("author@example.org"), "{actual}");
    assert!(
        actual.contains("https://licenses.test/license?redacted"),
        "{actual}"
    );
}

fn project_with_bsd_file(root: &Path, fields: &str) -> Vec<u8> {
    project(root, fields);
    let bytes = common::wheel_with_files(
        &common::metadata(fields),
        &[(
            "example-1.0.dist-info/licenses/LICENSE",
            spdx::license_id("BSD-3-Clause").unwrap().text().as_bytes(),
        )],
    );
    std::fs::write(root.join("example.whl"), &bytes).unwrap();
    policy(root, "'BSD-3-Clause'");
    bytes
}

#[rstest::rstest]
#[case::check("check", "interpreted_expression")]
#[case::describe("describe", "expression")]
fn legacy_supplement_json_preserves_declaration_and_source(
    #[case] command: &str,
    #[case] expression_field: &str,
) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    project_with_bsd_file(temp.path(), "License: BSD");
    let mut args = command_arguments(command);
    args.extend(["--offline", "--format=json"]);

    // Act
    let output = invoke(temp.path(), &temp.path().join("cache"), &args);

    // Assert
    let report = json(&output);
    let artifact = &report["packages"][0]["artifacts"][0];
    assert_eq!(artifact["evidence"]["license_origin"], "legacy_supplement");
    assert_eq!(
        artifact["evidence"]["original_license"],
        serde_json::json!(["BSD"])
    );
    assert_eq!(artifact[expression_field], "BSD-3-Clause");
    assert_eq!(artifact["diagnostics"], serde_json::json!([]));
    assert_eq!(output.status.code(), Some(0));
}

#[rstest::rstest]
#[case::check("check")]
#[case::describe("describe")]
fn legacy_supplement_text_identifies_successful_evidence(#[case] command: &str) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    project_with_bsd_file(temp.path(), "License: BSD");
    let mut args = command_arguments(command);
    args.extend(["--offline", "--format=text"]);

    // Act
    let output = invoke(temp.path(), &temp.path().join("cache"), &args);

    // Assert
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(
        text.contains("license source: legacy declaration supplemented by bundled files"),
        "{text}"
    );
    assert!(text.contains("example.whl"), "{text}");
    assert!(!text.contains("Warning"), "{text}");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn legacy_supplement_clarification_keeps_json_evidence_source() {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let bytes = project_with_bsd_file(temp.path(), "License: BSD");
    let hash = taloon::model::ContentHash::digest(&bytes);
    std::fs::write(
        temp.path().join("pyproject.toml"),
        format!("[tool.taloon]\nschema-version=1\nallow=['MIT']\n[[tool.taloon.clarifications]]\nartifact-hash='{hash}'\nexpression='MIT'\nreason='Reviewed synthetic artifact'\n"),
    ).unwrap();

    // Act
    let output = invoke(
        temp.path(),
        &temp.path().join("cache"),
        &["check", "--offline", "--format=json"],
    );
    // Assert
    assert_eq!(output.status.code(), Some(0));
    let report = json(&output);
    let artifact = &report["packages"][0]["artifacts"][0];
    assert_eq!(artifact["evidence"]["license_origin"], "legacy_supplement");
    assert_eq!(
        artifact["evidence"]["original_license"],
        serde_json::json!(["BSD"])
    );
    assert_eq!(artifact["interpreted_expression"], "MIT");
}

#[test]
fn legacy_supplement_clarification_keeps_text_evidence_source() {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let bytes = project_with_bsd_file(temp.path(), "License: BSD");
    let hash = taloon::model::ContentHash::digest(&bytes);
    std::fs::write(
        temp.path().join("pyproject.toml"),
        format!("[tool.taloon]\nschema-version=1\nallow=['MIT']\n[[tool.taloon.clarifications]]\nartifact-hash='{hash}'\nexpression='MIT'\nreason='Reviewed synthetic artifact'\n"),
    ).unwrap();

    // Act
    let output = invoke(
        temp.path(),
        &temp.path().join("cache"),
        &["check", "--offline", "--format=text"],
    );
    // Assert
    assert_eq!(output.status.code(), Some(0));
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(
        text.contains("license source: legacy declaration supplemented by bundled files"),
        "{text}"
    );
    assert!(text.contains("license: MIT"), "{text}");
    assert!(text.contains("Reviewed synthetic artifact"), "{text}");
}

#[rstest::rstest]
#[case::check_expression("check", "License-Expression: BSD-3-Clause", 0)]
#[case::describe_expression("describe", "License-Expression: BSD-3-Clause", 0)]
#[case::check_legacy("check", "License: BSD-3-Clause", 0)]
#[case::describe_legacy("describe", "License: BSD-3-Clause", 0)]
#[case::check_bundled("check", "", 0)]
#[case::describe_bundled("describe", "", 0)]
#[case::check_invalid_expression("check", "License-Expression: invalid\nLicense: BSD", 2)]
#[case::describe_invalid_expression("describe", "License-Expression: invalid\nLicense: BSD", 2)]
fn other_license_sources_do_not_claim_legacy_supplement(
    #[case] command: &str,
    #[case] fields: &str,
    #[case] exit_code: i32,
) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    project_with_bsd_file(temp.path(), fields);
    let mut args = command_arguments(command);
    args.extend(["--offline", "--format=text"]);

    // Act
    let output = invoke(temp.path(), &temp.path().join("cache"), &args);

    // Assert
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(
        !text.contains("legacy declaration supplemented by bundled files"),
        "{text}"
    );
    assert_eq!(output.status.code(), Some(exit_code));
}

#[rstest::rstest]
#[case::allow("allow=['MIT']", "allow", "License-Expression: MIT", 0, serde_json::json!("allow"))]
#[case::deny("deny=['Apache-2.0']", "deny", "License-Expression: MIT", 0, serde_json::json!("deny"))]
#[case::violation("deny=['MIT']", "deny", "License-Expression: MIT", 1, serde_json::json!("deny"))]
#[case::incomplete(
    "deny=[]",
    "deny",
    "License-Expression: MIT OR",
    2,
    serde_json::Value::Null
)]
fn cli_json_identifies_selected_policy_mode(
    #[case] settings: &str,
    #[case] mode: &str,
    #[case] fields: &str,
    #[case] exit: i32,
    #[case] decision_mode: serde_json::Value,
) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    project(root, fields);
    std::fs::write(
        root.join("pyproject.toml"),
        format!("[tool.taloon]\nschema-version=1\n{settings}"),
    )
    .unwrap();
    // Act
    let output = invoke(
        root,
        &root.join("cache"),
        &["check", "--offline", "--format", "json"],
    );
    // Assert
    let report = json(&output);
    assert_eq!(output.status.code(), Some(exit));
    assert_eq!(report["policy_mode"], mode);
    assert_eq!(
        report["packages"][0]["artifacts"][0]["decision"]["mode"],
        decision_mode
    );

    assert!(report.get("allow").is_none());
    assert!(report.get("deny").is_none());
}

#[rstest::rstest]
#[case::allow("allow=[]", "allow")]
#[case::deny("deny=[]", "deny")]
fn cli_text_identifies_policy_on_empty_inventory(#[case] settings: &str, #[case] mode: &str) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::write(root.join("uv.lock"), "version=1\nrevision=5").unwrap();
    std::fs::write(
        root.join("pyproject.toml"),
        format!("[tool.taloon]\nschema-version=1\n{settings}"),
    )
    .unwrap();
    // Act
    let output = invoke(root, &root.join("cache"), &["check", "--offline"]);
    // Assert
    assert_eq!(output.status.code(), Some(0));
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains(&format!("policy: {mode}"))
    );
}

#[test]
fn cli_rejects_both_empty_modes_as_configuration_error() {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    project(root, "License-Expression: MIT");
    std::fs::write(
        root.join("pyproject.toml"),
        "[tool.taloon]\nschema-version=1\nallow=[]\ndeny=[]",
    )
    .unwrap();
    // Act
    let output = invoke(
        root,
        &root.join("cache"),
        &["check", "--offline", "--format", "json"],
    );
    // Assert
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(json(&output)["diagnostics"][0]["code"], "configuration");
}

#[test]
fn describe_ignores_deny_policy_and_omits_permission_mode() {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    project(root, "License-Expression: MIT");
    std::fs::write(
        root.join("pyproject.toml"),
        "[tool.taloon]\nschema-version=1\ndeny=['MIT']",
    )
    .unwrap();
    // Act
    let output = invoke(
        root,
        &root.join("cache"),
        &["describe", "example", "--offline", "--format", "json"],
    );
    // Assert
    let report = json(&output);
    assert_eq!(output.status.code(), Some(0));
    assert!(report.get("policy_mode").is_none());
    assert!(
        report["packages"][0]["artifacts"][0]
            .get("decision")
            .is_none()
    );
}

#[rstest::rstest]
#[case::help("--help", "Usage:")]
#[case::version("--version", "taloon ")]
fn informational_flags_succeed_without_a_project(#[case] flag: &str, #[case] expected: &str) {
    // Arrange
    let directory = tempfile::tempdir().unwrap();
    // Act
    let output = invoke(directory.path(), &directory.path().join("cache"), &[flag]);
    // Assert
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8(output.stdout).unwrap().contains(expected));
    assert_eq!(output.stderr, b"");
}

#[test]
fn missing_lock_is_reported_in_text_output() {
    // Arrange
    let directory = tempfile::tempdir().unwrap();
    // Act
    let output = invoke(
        directory.path(),
        &directory.path().join("cache"),
        &["check"],
    );
    // Assert
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("no uv.lock found")
    );
}

#[test]
fn describe_invalid_input_explains_failure_in_text() {
    // Arrange
    let directory = tempfile::tempdir().unwrap();
    // Act
    let output = invoke(
        directory.path(),
        &directory.path().join("cache"),
        &["describe", "example", "--unknown"],
    );
    // Assert
    assert_eq!(output.status.code(), Some(2));
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("invalid input"));
    assert!(text.contains("invalid command line"));
}

#[test]
fn describe_text_preserves_conditional_dependency_context() {
    // Arrange
    let directory = tempfile::tempdir().unwrap();
    project(directory.path(), "License-Expression: MIT");
    std::fs::write(directory.path().join("uv.lock"), "version=1\nrevision=5\n[[package]]\nname='app'\nversion='1'\nsource={virtual='.'}\ndependencies=[{name='example',extra=['feature'],marker=\"sys_platform == 'linux'\"}]\n[package.optional-dependencies]\nextra=[{name='example'}]\n[package.dev-dependencies]\ntest=[{name='example'}]\n[[package]]\nname='example'\nversion='1.0'\nsource={path='example.whl'}\n[package.optional-dependencies]\nfeature=[]").unwrap();
    // Act
    let output = invoke(
        directory.path(),
        &directory.path().join("cache"),
        &["describe", "app", "--offline", "--verbose"],
    );
    // Assert
    assert_eq!(output.status.code(), Some(0));
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("all external packages"), "{text}");
    assert!(
        text.contains("workspace context (own license excluded): app"),
        "{text}"
    );
    assert!(text.contains("requested extras: feature"), "{text}");
    assert!(text.contains("marker: sys_platform == 'linux'"), "{text}");
    assert!(text.contains("extra: extra"), "{text}");
    assert!(text.contains("development: test"), "{text}");
}

#[cfg(unix)]
#[rstest::rstest]
#[case::check(vec!["check", "--offline"])]
#[case::describe(vec!["describe", "example", "--offline"])]
#[case::cache_clear(vec!["cache", "clear"])]
#[case::help(vec!["--help"])]
fn unwritable_stdout_is_reported_as_failure(#[case] arguments: Vec<&str>) {
    // Arrange
    let directory = tempfile::tempdir().unwrap();
    project(directory.path(), "License-Expression: MIT");
    let (reader, writer) = std::os::unix::net::UnixStream::pair().unwrap();
    drop(reader);
    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_taloon"))
        .current_dir(directory.path())
        .env("TALOON_CACHE_DIR", directory.path().join("cache"))
        .env_remove("NETRC")
        .args(arguments)
        .stdout(std::os::fd::OwnedFd::from(writer))
        .output()
        .unwrap();
    // Assert
    assert_eq!(output.status.code(), Some(2));
}

#[cfg(target_os = "macos")]
#[test]
fn terminal_progress_reports_completed_artifacts() {
    // Arrange
    let directory = tempfile::tempdir().unwrap();
    project(directory.path(), "License-Expression: MIT");
    std::fs::copy(
        directory.path().join("example.whl"),
        directory.path().join("second.whl"),
    )
    .unwrap();
    std::fs::write(directory.path().join("uv.lock"), "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={path='example.whl'}\nwheels=[{path='example.whl'},{path='second.whl'}]").unwrap();
    // Act: script provides a real terminal without changing the CLI's detection.
    let output = Command::new("/usr/bin/script")
        .args([
            "-q",
            "/dev/null",
            env!("CARGO_BIN_EXE_taloon"),
            "check",
            "--offline",
        ])
        .current_dir(directory.path())
        .env("TALOON_CACHE_DIR", directory.path().join("cache"))
        .env_remove("NETRC")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    // Assert
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(
        text.contains("taloon: license evidence 2/2 artifacts"),
        "{text}"
    );
    assert!(
        text.contains("taloon: Compliant; 1 packages, 2 artifacts"),
        "{text}"
    );
}

#[test]
fn verbose_check_explains_dependency_origin_path() {
    // Arrange
    let directory = tempfile::tempdir().unwrap();
    project(directory.path(), "License-Expression: MIT");
    policy(directory.path(), "'ISC'");
    std::fs::write(directory.path().join("uv.lock"), "version=1\nrevision=5\n[[package]]\nname='app'\nversion='1'\nsource={virtual='.'}\ndependencies=[{name='example',marker=\"sys_platform == 'linux'\"}]\n[[package]]\nname='example'\nversion='1.0'\nsource={path='example.whl'}").unwrap();
    // Act
    let output = invoke(
        directory.path(),
        &directory.path().join("cache"),
        &["check", "--offline", "--verbose"],
    );
    // Assert
    assert_eq!(output.status.code(), Some(1));
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("from app"), "{text}");
    assert!(text.contains("via"), "{text}");
    assert!(text.contains("marker sys_platform == 'linux'"), "{text}");
}

#[test]
fn check_explains_unsupported_package_source_in_text() {
    // Arrange
    let directory = tempfile::tempdir().unwrap();
    policy(directory.path(), "'MIT'");
    std::fs::write(directory.path().join("uv.lock"), "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={git='https://example.test/example.git'}").unwrap();
    // Act
    let output = invoke(
        directory.path(),
        &directory.path().join("cache"),
        &["check", "--offline"],
    );
    // Assert
    assert_eq!(output.status.code(), Some(2));
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("UnsupportedSource"), "{text}");
    assert!(
        text.contains("external source has no supported distribution archive"),
        "{text}"
    );
}

#[cfg(target_os = "macos")]
#[test]
fn missing_cache_home_is_actionable() {
    // Arrange
    let directory = tempfile::tempdir().unwrap();
    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_taloon"))
        .current_dir(directory.path())
        .env_remove("HOME")
        .env_remove("TALOON_CACHE_DIR")
        .args(["cache", "clear"])
        .output()
        .unwrap();
    // Assert
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("cannot clear taloon cache")
    );
}

#[cfg(unix)]
#[test]
fn deleted_working_directory_reports_input_error() {
    // Arrange
    let directory = tempfile::tempdir().unwrap();
    let child = directory.path().join("removed");
    std::fs::create_dir(&child).unwrap();
    // Act: only the isolated child process loses its working directory.
    let output = Command::new("/bin/sh")
        .current_dir(&child)
        .args([
            "-c",
            "rmdir \"$PWD\" && exec \"$1\" check --format json",
            "fixture",
            env!("CARGO_BIN_EXE_taloon"),
        ])
        .output()
        .unwrap();
    // Assert
    assert_eq!(output.status.code(), Some(2));
    let report = json(&output);
    assert_eq!(
        report["diagnostics"][0]["message"],
        "cannot determine current directory"
    );
}
