mod common;
use std::{
    ffi::OsString,
    path::Path,
    process::{Command, Output},
};
fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(root)
        .args(["-c", "core.hooksPath=/dev/null"])
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
fn report(output: &Output) -> serde_json::Value {
    let first = output
        .stdout
        .iter()
        .position(|b| *b == b'{')
        .expect("JSON report in failed hook output");
    serde_json::Deserializer::from_slice(&output.stdout[first..])
        .into_iter::<serde_json::Value>()
        .next()
        .unwrap()
        .unwrap()
}
#[test]
fn local_hook_checks_the_full_lock() {
    let workflow = include_str!("../.github/workflows/ci.yml");
    assert!(
        workflow.contains("Local hook integration"),
        "CI must exercise the local hook"
    );
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let bin = root.join("bin");
    std::fs::create_dir(&bin).unwrap();
    let source = std::env::var_os("TALOON_WORKFLOW_BINARY")
        .unwrap_or_else(|| OsString::from(env!("CARGO_BIN_EXE_taloon")));
    std::fs::copy(source, bin.join("taloon")).unwrap();
    let lock = create_project(root);
    let mut paths = vec![bin];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let path = std::env::join_paths(paths).unwrap();
    let cache = root.join("cache");
    let hook = Command::new("prek")
        .current_dir(root)
        .env("PATH", &path)
        .env("TALOON_CACHE_DIR", &cache)
        .env("PREK_HOME", root.join("prek-cache"))
        .env("PRE_COMMIT_HOME", root.join("prek-cache"))
        .env("XDG_CACHE_HOME", root.join("xdg"))
        .args(["run", "taloon", "--files", "touched.txt"])
        .output()
        .expect("workflow test requires prek 0.5.7 on PATH");
    assert_eq!(
        hook.status.code(),
        Some(1),
        "{}\n{}",
        String::from_utf8_lossy(&hook.stdout),
        String::from_utf8_lossy(&hook.stderr)
    );
    let hook_report = report(&hook);
    let ci = Command::new(root.join("bin/taloon"))
        .current_dir(root)
        .env("TALOON_CACHE_DIR", cache)
        .args(["check", "--offline", "--format", "json"])
        .output()
        .unwrap();
    assert_eq!(ci.status.code(), Some(1));
    let ci_report = report(&ci);
    assert_eq!(hook_report["packages"], ci_report["packages"]);
    assert_eq!(hook_report["status"], ci_report["status"]);
    assert_eq!(hook_report["packages"].as_array().unwrap().len(), 2);
    assert_eq!(std::fs::read_to_string(root.join("uv.lock")).unwrap(), lock);
}

fn create_project(root: &Path) -> String {
    for (name, license) in [("direct", "MIT"), ("transitive", "ISC")] {
        let metadata = format!(
            "Metadata-Version: 2.4\nName: {name}\nVersion: 1.0\nLicense-Expression: {license}\n\n"
        );
        std::fs::write(
            root.join(format!("{name}.whl")),
            common::wheel(metadata.as_bytes()),
        )
        .unwrap();
    }
    let lock = "version=1\nrevision=5\n[[package]]\nname='root'\nversion='1.0'\nsource={virtual='.'}\ndependencies=[{name='direct'}]\n[[package]]\nname='direct'\nversion='1.0'\nsource={path='direct.whl'}\ndependencies=[{name='transitive'}]\n[[package]]\nname='transitive'\nversion='1.0'\nsource={path='transitive.whl'}\n";
    std::fs::write(root.join("uv.lock"), lock).unwrap();
    std::fs::write(
        root.join("pyproject.toml"),
        "[tool.taloon]\nschema-version=1\nallow=['MIT']",
    )
    .unwrap();
    std::fs::write(root.join("touched.txt"), "before").unwrap();
    let guide = include_str!("../docs/user/usage.md");
    let hook = guide
        .split("```yaml\n")
        .nth(1)
        .unwrap()
        .split("```")
        .next()
        .unwrap()
        .replace(
            "entry: taloon check",
            "entry: taloon check --offline --format json",
        );
    std::fs::write(root.join(".pre-commit-config.yaml"), hook).unwrap();
    git(root, &["init", "--quiet"]);
    git(
        root,
        &[
            "add",
            "uv.lock",
            "pyproject.toml",
            "direct.whl",
            "transitive.whl",
            "touched.txt",
            ".pre-commit-config.yaml",
        ],
    );
    git(
        root,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "fixture",
        ],
    );
    std::fs::write(root.join("touched.txt"), "after").unwrap();

    lock.to_owned()
}
