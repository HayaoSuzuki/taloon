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

#[test]
fn documented_policies_use_the_implemented_schema() {
    let guide = include_str!("../docs/user/configuration.md");
    let date = taloon::model::EvaluationDate::parse("2026-10-31").unwrap();
    let mut count = 0;
    for rest in guide.split("```toml\n").skip(1) {
        let code = rest.split("```").next().unwrap();
        taloon::policy::Policy::from_pyproject(code, date).unwrap();
        count += 1;
    }
    assert_eq!(count, 2);
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
