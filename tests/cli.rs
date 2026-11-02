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
#[test]
fn cli_finds_nearest_lock_and_requires_sibling_policy() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    project(root, "License-Expression: MIT");
    let member = root.join("members/a");
    std::fs::create_dir_all(&member).unwrap();
    let cache = root.join("cache");
    let output = invoke(&member, &cache, &["check", "--offline", "--format", "json"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(json(&output)["packages"].as_array().unwrap().len(), 1);
    let explicit = invoke(
        &member,
        &cache,
        &["check", "--project", ".", "--format", "json"],
    );
    assert_eq!(explicit.status.code(), Some(2));
    json(&explicit);
    std::fs::write(member.join("uv.lock"), "version=1\nrevision=5").unwrap();
    let missing = invoke(&member, &cache, &["check", "--offline", "--format=json"]);
    assert_eq!(missing.status.code(), Some(2));
    json(&missing);
    policy(&member, "");
    let empty = invoke(&member, &cache, &["check", "--offline", "--format", "json"]);
    assert_eq!(empty.status.code(), Some(0));
    assert_eq!(json(&empty)["packages"], serde_json::json!([]));
}
#[test]
fn cli_json_reports_complete_and_incomplete_runs() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    project(root, "License-Expression: MIT");
    let cache = root.join("cache");
    for (allow, expected) in [("'MIT'", 0), ("", 1)] {
        policy(root, allow);
        let output = invoke(root, &cache, &["check", "--offline", "--format", "json"]);
        assert_eq!(output.status.code(), Some(expected));
        json(&output);
        assert!(
            output.stderr.is_empty(),
            "piped reports must not emit terminal progress"
        );
    }
    std::fs::remove_file(root.join("example.whl")).unwrap();
    let output = invoke(root, &cache, &["check", "--offline", "--format", "json"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(json(&output)["status"], "incomplete");
    let bad = invoke(root, &cache, &["check", "--format", "json", "--unknown"]);
    assert_eq!(bad.status.code(), Some(2));
    json(&bad);
}
#[test]
fn cli_offline_policy_change_rechecks_cached_evidence() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    project(root, "License-Expression: MIT");
    let cache = root.join("cache");
    let allowed = invoke(root, &cache, &["check", "--format", "json"]);
    assert_eq!(allowed.status.code(), Some(0));
    assert_eq!(json(&allowed)["acquisition"]["archive_extractions"], 1);
    policy(root, "");
    let denied = invoke(root, &cache, &["check", "--offline", "--format", "json"]);
    assert_eq!(denied.status.code(), Some(1));
    let report = json(&denied);
    assert_eq!(report["acquisition"]["archive_extractions"], 0);
    assert_eq!(report["acquisition"]["network_requests"], 0);
    assert_eq!(report["acquisition"]["cache_hits"], 1);
    policy(root, "'MIT','ISC'");
    let warning = invoke(
        root,
        &cache,
        &["check", "--offline", "--format", "json", "--deny-warnings"],
    );
    assert_eq!(warning.status.code(), Some(1));
    json(&warning);
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
fn cli_redacts_locator_secrets_and_escapes_terminal_controls() {
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
    std::fs::write(root.join("uv.lock"),"version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={path=\"evil\\u001b[31m.whl\"}").unwrap();
    let output = invoke(root, &root.join("cache"), &["check", "--offline"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(!output.stdout.contains(&27));
}
#[test]
fn cli_netrc_override_is_explicit_and_offline_does_not_need_credentials() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    project(root, "License-Expression: MIT");
    let output = Command::new(env!("CARGO_BIN_EXE_taloon"))
        .current_dir(root)
        .env("NETRC", root.join("missing-netrc"))
        .env("TALOON_CACHE_DIR", root.join("cache"))
        .args(["check", "--format", "json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    json(&output);
    let output = Command::new(env!("CARGO_BIN_EXE_taloon"))
        .current_dir(root)
        .env("NETRC", root.join("missing-netrc"))
        .env("TALOON_CACHE_DIR", root.join("cache"))
        .args(["check", "--offline", "--format", "json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
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
    for name in [
        "notes.json",
        "v1-archive-wheel-sha256-invalid.json",
        ".evidence-unfinished",
        "v2-archive-wheel-sha256-unknown.json",
    ] {
        assert_eq!(std::fs::read(cache.join(name)).unwrap(), b"keep");
    }
    assert_eq!(std::fs::read(directory.join("keep")).unwrap(), b"keep");
    let second = invoke(outside.path(), &cache, &["cache", "clear"]);
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(second.stdout, b"Removed 0 taloon cache entries.\n");
    let check = invoke(root, &cache, &["check", "--format", "json"]);
    assert_eq!(check.status.code(), Some(0));
    assert_eq!(json(&check)["acquisition"]["archive_extractions"], 1);
    assert_eq!(json(&check)["acquisition"]["cache_hits"], 0);
}

#[test]
fn cache_clear_handles_missing_cache_and_invalid_root() {
    let temp = tempfile::tempdir().unwrap();
    let cache = temp.path().join("missing");
    let output = invoke(temp.path(), &cache, &["cache", "clear"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stdout, b"Removed 0 taloon cache entries.\n");
    assert!(!cache.exists());
    std::fs::write(&cache, b"keep").unwrap();
    let output = invoke(temp.path(), &cache, &["cache", "clear"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"cannot clear taloon cache\n");
    assert_eq!(std::fs::read(&cache).unwrap(), b"keep");
}

#[cfg(unix)]
#[test]
fn cache_clear_preserves_symlinks_and_rejects_symlink_root() {
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
    let cache = if cfg!(target_os = "macos") {
        temp.path().join("Library/Caches/taloon")
    } else {
        base.join("taloon")
    };
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
#[test]
fn cache_clear_rejects_root_symlink_with_redundant_components() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    let cache = temp.path().join("cache");
    std::fs::create_dir(&cache).unwrap();
    let record = cache.join(format!("v1-archive-wheel-sha256-{}.json", "e".repeat(64)));
    std::fs::write(&record, b"keep").unwrap();
    let root_link = temp.path().join("linked-cache");
    symlink(&cache, &root_link).unwrap();
    for suffix in ["/", "/.", "/./", "//"] {
        let spelling = std::path::PathBuf::from(format!("{}{suffix}", root_link.display()));
        let output = invoke(temp.path(), &spelling, &["cache", "clear"]);
        assert_eq!(output.status.code(), Some(2), "suffix {suffix}: {output:?}");
        assert_eq!(output.stderr, b"cannot clear taloon cache\n");
        assert!(root_link.is_symlink());
        assert_eq!(std::fs::read(&record).unwrap(), b"keep");
    }
}

fn describe_project(root: &Path) {
    let mit = common::wheel(&common::metadata("License-Expression: MIT"));
    let bsd = common::wheel(&common::metadata("License-Expression: BSD-3-Clause"));
    std::fs::write(root.join("mit.whl"), mit).unwrap();
    std::fs::write(root.join("bsd.whl"), bsd).unwrap();
    std::fs::write(root.join("uv.lock"),"version=1\nrevision=5\n[[package]]\nname='app'\nversion='1.0'\nsource={virtual='.'}\ndependencies=[{name='example'}]\n[[package]]\nname='example'\nversion='1.0'\nsource={path='mit.whl'}\nwheels=[{path='mit.whl'},{path='bsd.whl'}]\n[[package]]\nname='unrelated'\nversion='1.0'\nsource={path='does-not-exist.whl'}").unwrap();
}
#[test]
fn describe_cli_is_policy_free_selected_and_shares_check_cache() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let cache = root.join("cache");
    describe_project(root);
    let output = invoke(root, &cache, &["describe", "EXAMPLE", "--format", "json"]);
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
    // An existing policy can be malformed/expired: describe does not read it.
    std::fs::write(root.join("pyproject.toml"), "not valid toml").unwrap();
    let output = invoke(
        root,
        &cache,
        &["describe", "example", "--offline", "--format=json"],
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(json(&output)["acquisition"]["cache_hits"], 2);
    assert_eq!(json(&output)["acquisition"]["network_requests"], 0);
    let text = invoke(root, &cache, &["describe", "example", "--offline"]);
    assert_eq!(text.status.code(), Some(0));
    let text = String::from_utf8(text.stdout).unwrap();
    assert!(text.contains("MIT"));
    assert!(text.contains("BSD-3-Clause"));
    assert!(!text.contains("Compliant"));
    // Check can consume the same original evidence even after a describe run.
    policy(root, "'MIT','BSD-3-Clause'");
    let check = invoke(root, &cache, &["check", "--offline", "--format", "json"]);
    assert_eq!(check.status.code(), Some(2)); // unrelated missing artifact remains in check scope
    assert_eq!(json(&check)["acquisition"]["cache_hits"], 2);
    let project = invoke(
        root,
        &cache,
        &["describe", "app", "--offline", "--format", "json"],
    );
    assert_eq!(project.status.code(), Some(2));
    let report = json(&project);
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
#[test]
fn describe_cli_reports_its_own_json_errors_and_exact_project_selection() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let cache = root.join("cache");
    describe_project(root);
    for args in [
        vec!["describe", "missing", "--format", "json"],
        vec!["describe", "--format=json"],
        vec!["describe", "example", "--unknown", "--format", "json"],
        vec![
            "describe",
            "example",
            "--project",
            "missing",
            "--format",
            "json",
        ],
    ] {
        let output = invoke(root, &cache, &args);
        assert_eq!(output.status.code(), Some(2));
        let report = json(&output);
        assert_eq!(report["command"], "describe");
        assert_eq!(report["status"], "incomplete");
        assert_eq!(report["acquisition"]["network_requests"], 0);
    }
    let member = root.join("subdir");
    std::fs::create_dir(&member).unwrap();
    let output = invoke(
        &member,
        &cache,
        &["describe", "example", "--format", "json"],
    );
    assert_eq!(output.status.code(), Some(0));
    let output = invoke(
        &member,
        &cache,
        &["describe", "example", "--project", ".", "--format", "json"],
    );
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(json(&output)["command"], "describe");
}

#[test]
fn describe_cli_redacts_sources_and_escapes_conditions() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let hash = taloon::model::ContentHash::digest(b"archive");
    let lock = format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={{registry='https://alice:password@index.test/simple?token=secret'}}\nresolution-markers=[\"\\u001b[31m\"]\nwheels=[{{url='https://alice:password@files.test/example.whl?token=secret',hash='{hash}'}}]\ndependencies=[{{name='example',marker=\"\\u001b[31m\"}}]"
    );
    std::fs::write(root.join("uv.lock"), lock).unwrap();
    for format in ["text", "json"] {
        let output = invoke(
            root,
            &root.join("cache"),
            &["describe", "example", "--offline", "--format", format],
        );
        assert_eq!(output.status.code(), Some(2));
        let stdout = String::from_utf8(output.stdout.clone()).unwrap();
        for secret in ["alice", "password", "secret", "\u{1b}"] {
            assert!(!stdout.contains(secret), "{stdout}");
        }
        if format == "json" {
            assert_eq!(json(&output)["command"], "describe");
        } else {
            assert!(stdout.contains("marker:"));
            assert!(stdout.contains("resolution markers"));
        }
    }
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
fn describe_cli_empty_project_and_multiple_source_versions_are_explicit() {
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
    let mut args = vec![command];
    if command == "describe" {
        args.push("example");
    }
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
    let mut args = vec![command];
    if command == "describe" {
        args.push("example");
    }
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
    let mut args = vec![command];
    if command == "describe" {
        args.push("example");
    }
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

#[rstest::rstest]
#[case::json("json")]
#[case::text("text")]
fn legacy_supplement_clarification_keeps_evidence_source(#[case] format: &str) {
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
        &["check", "--offline", "--format", format],
    );

    // Assert
    if format == "json" {
        let report = json(&output);
        let artifact = &report["packages"][0]["artifacts"][0];
        assert_eq!(artifact["evidence"]["license_origin"], "legacy_supplement");
        assert_eq!(
            artifact["evidence"]["original_license"],
            serde_json::json!(["BSD"])
        );
        assert_eq!(artifact["interpreted_expression"], "MIT");
    } else {
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(
            text.contains("license source: legacy declaration supplemented by bundled files"),
            "{text}"
        );
        assert!(text.contains("license: MIT"), "{text}");
        assert!(text.contains("Reviewed synthetic artifact"), "{text}");
    }
    assert_eq!(output.status.code(), Some(0));
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
    let mut args = vec![command];
    if command == "describe" {
        args.push("example");
    }
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
#[case::allow("allow=['MIT']", "allow", "License-Expression: MIT", 0)]
#[case::deny("deny=['Apache-2.0']", "deny", "License-Expression: MIT", 0)]
#[case::violation("deny=['MIT']", "deny", "License-Expression: MIT", 1)]
#[case::incomplete("deny=[]", "deny", "License-Expression: MIT OR", 2)]
fn cli_json_identifies_selected_policy_mode(
    #[case] settings: &str,
    #[case] mode: &str,
    #[case] fields: &str,
    #[case] exit: i32,
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
    if exit != 2 {
        assert_eq!(
            report["packages"][0]["artifacts"][0]["decision"]["mode"],
            mode
        );
    }
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
