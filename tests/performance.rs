use std::{
    fmt::Write as _,
    io::{Cursor, Write},
    path::Path,
    process::Command,
    time::Instant,
};
use taloon::model::ContentHash;
fn fixture(root: &Path) -> u64 {
    let mut total_size = 0_u64;
    let mut seen = std::collections::BTreeSet::new();
    let artifacts = root.join("artifacts");
    std::fs::create_dir(&artifacts).unwrap();
    let mut lock = "version=1\nrevision=5\n".to_owned();
    for package in 0..1000 {
        let name = format!("example-{package:04}");
        let _ = writeln!(
            lock,
            "[[package]]\nname='{name}'\nversion='1.0'\nsource={{path='artifacts/{name}-0.whl'}}\nwheels=["
        );
        for variant in 0..10 {
            let metadata = format!(
                "Metadata-Version: 2.4\nName: {name}\nVersion: 1.0\nLicense-Expression: MIT\nArtifact-Variant: {variant}\n\n"
            );
            let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
            archive
                .start_file(
                    format!("{name}-1.0.dist-info/METADATA"),
                    zip::write::SimpleFileOptions::default(),
                )
                .unwrap();
            archive.write_all(metadata.as_bytes()).unwrap();
            let bytes = archive.finish().unwrap().into_inner();
            total_size += u64::try_from(bytes.len()).unwrap();
            assert!(
                seen.insert(ContentHash::digest(&bytes)),
                "fixture digests must be distinct"
            );
            let path = format!("artifacts/{name}-{variant}.whl");
            std::fs::write(root.join(&path), &bytes).unwrap();
            let _ = writeln!(
                lock,
                "{{path='{path}',hash='{}'}},",
                ContentHash::digest(&bytes)
            );
        }
        lock.push_str("]\n");
    }
    std::fs::write(root.join("uv.lock"), lock).unwrap();
    std::fs::write(
        root.join("pyproject.toml"),
        "[tool.taloon]\nschema-version=1\nallow=['MIT']",
    )
    .unwrap();
    assert_eq!(seen.len(), 10000);
    total_size
}
fn run(binary: &Path, root: &Path) -> (f64, serde_json::Value) {
    let start = Instant::now();
    let output = Command::new(binary)
        .current_dir(root)
        .env("TALOON_CACHE_DIR", root.join("cache"))
        .env_remove("NETRC")
        .args(["check", "--offline", "--format", "json"])
        .output()
        .unwrap();
    let elapsed = start.elapsed().as_secs_f64();
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["packages"].as_array().unwrap().len(), 1000);
    assert_eq!(
        report["packages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["artifacts"].as_array().unwrap().len())
            .sum::<usize>(),
        10000
    );
    assert_eq!(report["acquisition"]["network_requests"], 0);
    assert_eq!(report["acquisition"]["downloaded_bytes"], 0);
    (elapsed, report)
}
#[test]
#[ignore = "opt-in release process benchmark; creates 10000 synthetic local archives"]
fn warm_cache_process_target() {
    let binary = std::env::var_os("TALOON_BENCH_BINARY")
        .expect("set TALOON_BENCH_BINARY to a release executable");
    let binary = std::fs::canonicalize(binary).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let total_size = fixture(temp.path());
    println!(
        "fixture_packages=1000, fixture_artifacts=10000, distinct_digests=10000, local_archive_bytes={total_size}"
    );
    let (cold, report) = run(&binary, temp.path());
    assert_eq!(report["acquisition"]["archive_extractions"], 10000);
    println!(
        "cold_local_seconds={cold:.6}, counters={}",
        report["acquisition"]
    );
    drop(report);
    for attempt in 1..=5 {
        let (seconds, report) = run(&binary, temp.path());
        assert_eq!(report["acquisition"]["archive_extractions"], 0);
        assert_eq!(report["acquisition"]["cache_hits"], 10000);
        println!(
            "warm_run={attempt}, seconds={seconds:.6}, goal_seconds=2, counters={}",
            report["acquisition"]
        );
    }
}
