//! Runtime setup and rendering; inspection owns all verdict rules.
use crate::acquisition::{Acquirer, AcquisitionBatch, AcquisitionOptions, EvidenceCache};
mod artifacts;
mod describe;
mod json;
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use std::{
    borrow::Cow,
    ffi::OsString,
    fmt::Write as _,
    io::{IsTerminal as _, Read, Write as _},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use taloon::{
    evidence::LicenseOrigin,
    inspection::Inspection,
    model::{ArtifactRequest, Code, Error, EvaluationDate},
    report::{Diagnostic, Report, Status},
};
#[derive(Parser)]
#[command(version, about)]
struct Arguments {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    /// Describe declared licenses of a locked package and its dependencies.
    Describe {
        package: String,
        /// Select the exact directory containing uv.lock.
        #[arg(long)]
        project: Option<PathBuf>,
        #[arg(long, value_enum, default_value = "text")]
        format: Format,
        /// Use local artifacts and sufficient cached evidence; never access the network.
        #[arg(long)]
        offline: bool,
        /// Show each artifact and its complete observed declarations.
        #[arg(long)]
        verbose: bool,
    },
    /// Manage taloon's persisted evidence cache.
    Cache {
        #[command(subcommand)]
        command: CacheCommand,
    },
    /// Check every external package and recorded artifact in uv.lock.
    Check {
        /// Read uv.lock and pyproject.toml in this exact directory.
        #[arg(long)]
        project: Option<PathBuf>,
        #[arg(long, value_enum, default_value = "text")]
        format: Format,
        /// Use local artifacts and sufficient cached evidence; never access the network.
        #[arg(long)]
        offline: bool,
        /// Show each artifact and its complete observed declarations.
        #[arg(long)]
        verbose: bool,
        /// Fail a complete check when it emits warnings.
        #[arg(long)]
        deny_warnings: bool,
    },
}
#[derive(Subcommand)]
enum CacheCommand {
    /// Remove taloon cache records, retaining unrelated files and directories.
    Clear,
}
#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Text,
    Json,
}
#[derive(Serialize)]
struct Envelope<'a> {
    #[serde(flatten)]
    report: &'a Report,
    exit_code: u8,
}
pub(crate) fn run() -> u8 {
    let args: Vec<OsString> = std::env::args_os().collect();
    let describing = args.get(1).is_some_and(|arg| arg == "describe");
    let format = if args.iter().any(|s| s == "--format=json")
        || args
            .windows(2)
            .any(|s| s[0] == "--format" && s[1] == "json")
    {
        Format::Json
    } else {
        Format::Text
    };
    match Arguments::try_parse_from(args) {
        Ok(Arguments {
            command:
                Command::Describe {
                    package,
                    project,
                    format,
                    offline,
                    verbose,
                },
        }) => {
            let report =
                describe::run(project.as_deref(), &package, offline).unwrap_or_else(|error| {
                    taloon::description::DescriptionReport::from_input_error(&package, error)
                });
            describe::emit(&report, format, verbose)
        }
        Ok(Arguments {
            command:
                Command::Check {
                    project,
                    format,
                    offline,
                    verbose,
                    deny_warnings,
                },
        }) => {
            let report =
                check(project.as_deref(), offline).unwrap_or_else(Report::from_input_error);
            emit(&report, format, deny_warnings, verbose)
        }
        Ok(Arguments {
            command: Command::Cache {
                command: CacheCommand::Clear,
            },
        }) => clear_cache(),
        Err(error) if !error.use_stderr() => {
            if error.print().is_ok() {
                0
            } else {
                2
            }
        }
        Err(_) if describing => describe::emit(
            &taloon::description::DescriptionReport::from_input_error(
                "",
                input("invalid command line; run taloon describe --help"),
            ),
            format,
            false,
        ),
        Err(_) => emit(
            &Report::from_input_error(Error::new(
                Code::Configuration,
                "invalid command line; run taloon --help",
            )),
            format,
            false,
            false,
        ),
    }
}
fn clear_cache() -> u8 {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let result = cache_directory(home.as_deref()).and_then(|root| EvidenceCache::new(root).clear());
    if let Ok(removed) = result {
        if writeln!(
            std::io::stdout().lock(),
            "Removed {removed} taloon cache entries."
        )
        .is_ok()
        {
            return 0;
        }
        let _ = writeln!(std::io::stderr().lock(), "cannot write cache clear result");
    } else {
        let _ = writeln!(std::io::stderr().lock(), "cannot clear taloon cache");
    }
    2
}
fn check(project: Option<&Path>, offline: bool) -> Result<Report, Error> {
    let root = project_root(project)?;
    let lock = read_document(
        &root.join("uv.lock"),
        64 * 1024 * 1024,
        "cannot read UTF-8 uv.lock",
    )?;
    let config = read_document(
        &root.join("pyproject.toml"),
        16 * 1024 * 1024,
        "cannot read sibling UTF-8 pyproject.toml",
    )?;
    let date = EvaluationDate::parse(&time::OffsetDateTime::now_utc().date().to_string())?;
    let inspection = Inspection::from_documents(&lock, &config, date)?;
    let batch = acquire(root, offline, inspection.requests())?;
    Ok(inspection
        .evaluate(batch.results)
        .with_warnings(batch.warnings)
        .with_acquisition_summary(batch.summary))
}
fn project_root(project: Option<&Path>) -> Result<PathBuf, Error> {
    let cwd = std::env::current_dir().map_err(|_| input("cannot determine current directory"))?;
    project.map_or_else(|| discover(&cwd), |p| Ok(cwd.join(p)))
}
fn acquire(
    root: PathBuf,
    offline: bool,
    requests: &[ArtifactRequest],
) -> Result<AcquisitionBatch, Error> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let netrc = selected_netrc(
        std::env::var_os("NETRC").map(PathBuf::from),
        home.as_deref(),
    );
    let cache_dir = cache_directory(home.as_deref())?;
    let acquirer = Acquirer::new(AcquisitionOptions {
        project_dir: root,
        cache_dir,
        offline,
        netrc,
    })?;
    let interactive = std::io::stderr().is_terminal();
    let started = Instant::now();
    let mut last_render = None;
    let batch = acquirer.acquire_with_progress(requests, |progress| {
        if !interactive {
            return;
        }
        let elapsed = started.elapsed();
        if progress.completed == progress.total
            || last_render.is_none_or(|last| elapsed.saturating_sub(last) >= Duration::from_secs(1))
        {
            let mut stderr = std::io::stderr().lock();
            let _ = write!(
                stderr,
                "\rtaloon: license evidence {}/{} artifacts ({}s)",
                progress.completed,
                progress.total,
                elapsed.as_secs()
            );
            let _ = stderr.flush();
            last_render = Some(elapsed);
        }
    });
    if interactive {
        let _ = writeln!(std::io::stderr());
    }
    Ok(batch)
}
fn input(message: &str) -> Error {
    Error::new(Code::Configuration, message)
}
fn discover(cwd: &Path) -> Result<PathBuf, Error> {
    for directory in cwd.ancestors() {
        match std::fs::metadata(directory.join("uv.lock")) {
            Ok(_) => return Ok(directory.to_owned()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(input("cannot inspect candidate uv.lock")),
        }
    }
    Err(input(
        "no uv.lock found in current directory or its parents",
    ))
}
fn read_document(path: &Path, limit: u64, message: &str) -> Result<String, Error> {
    let file = std::fs::File::open(path).map_err(|_| input(message))?;
    let mut text = String::new();
    file.take(limit + 1)
        .read_to_string(&mut text)
        .map_err(|_| input(message))?;
    if u64::try_from(text.len()).unwrap_or(u64::MAX) > limit {
        return Err(input("project document exceeds its reading limit"));
    }
    Ok(text)
}
fn selected_netrc(explicit: Option<PathBuf>, home: Option<&Path>) -> Option<PathBuf> {
    explicit.or_else(|| home.map(|p| p.join(".netrc")).filter(|p| p.exists()))
}
fn cache_directory(home: Option<&Path>) -> Result<PathBuf, Error> {
    if let Some(path) = std::env::var_os("TALOON_CACHE_DIR") {
        // An empty root already writes records in the current directory via
        // Path::join. Resolve it explicitly so directory operations agree.
        return Ok(if path.is_empty() {
            PathBuf::from(".")
        } else {
            PathBuf::from(path)
        });
    }
    let base = if cfg!(target_os = "macos") {
        home.map(|p| p.join("Library/Caches"))
    } else if cfg!(windows) {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| home.map(|p| p.join(".cache")))
    };
    base.map(|p| p.join("taloon"))
        .ok_or_else(|| input("cannot determine cache directory; set TALOON_CACHE_DIR"))
}
fn emit(report: &Report, format: Format, deny_warnings: bool, verbose: bool) -> u8 {
    let code = report.exit_code(deny_warnings);
    let encoded = match format {
        Format::Json => json::encode(&Envelope {
            report,
            exit_code: code,
        })
        .map_err(|_| ()),
        Format::Text => Ok(text_report(report, verbose).into_bytes()),
    };
    let Ok(mut bytes) = encoded else {
        return 2;
    };
    bytes.push(b'\n');
    if std::io::stdout().lock().write_all(&bytes).is_err() {
        let _ = writeln!(std::io::stderr().lock(), "cannot write taloon report");
        return 2;
    }
    code
}
// Ordinary report fields borrow their text; only escaped output needs storage.
fn safe(text: &str) -> Cow<'_, str> {
    if !text.chars().any(needs_terminal_escape) {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if needs_terminal_escape(c) {
            out.extend(c.escape_default());
        } else {
            out.push(c);
        }
    }
    Cow::Owned(out)
}
fn needs_terminal_escape(c: char) -> bool {
    c.is_control() || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

fn diagnostic(out: &mut String, d: &Diagnostic) {
    let _ = writeln!(
        out,
        "  {:?} {:?}: {}",
        d.severity,
        d.error.code,
        safe(&d.error.message)
    );
}
fn text_report(report: &Report, verbose: bool) -> String {
    let mut out = String::new();
    let artifacts: usize = report.packages().iter().map(|p| p.artifacts.len()).sum();
    let _ = writeln!(
        out,
        "taloon: {:?}; {} packages, {artifacts} artifacts",
        report.status(),
        report.packages().len()
    );
    if let Some(mode) = report.policy_mode() {
        let _ = writeln!(out, "policy: {}", mode.as_str());
    }
    for d in report.diagnostics() {
        diagnostic(&mut out, d);
    }
    for package in report.packages() {
        if !verbose
            && package.status == Status::Compliant
            && package.artifacts.iter().all(|a| {
                a.diagnostics.is_empty()
                    && a.decision.as_ref().is_none_or(|d| d.overrides.is_empty())
                    && !a
                        .evidence
                        .as_ref()
                        .is_some_and(|e| e.license_origin == Some(LicenseOrigin::LegacySupplement))
            })
        {
            continue;
        }
        let _ = writeln!(out, "{:?}: {}", package.status, safe(&package.package));
        let _ = writeln!(
            out,
            "  source: {:?}: {}",
            package.source.kind,
            safe(&package.source.location)
        );
        for d in &package.diagnostics {
            diagnostic(&mut out, d);
        }
        artifacts::render(
            &mut out,
            package.artifacts.iter().map(artifacts::Artifact::from),
            package.artifacts.len(),
            verbose,
        );
        if let Some(note) = &package.origin_note {
            let _ = writeln!(out, "  {}", safe(note));
        }
        for path in &package.paths {
            let _ = writeln!(out, "  from {}", safe(&path.entry));
            for step in &path.steps {
                let _ = writeln!(
                    out,
                    "    {} via {} marker {} branches {}",
                    safe(&step.package),
                    safe(&step.via.join(", ")),
                    safe(step.marker.as_deref().unwrap_or("unconditional")),
                    safe(&step.package_markers.join(", "))
                );
            }
        }
    }
    acquisition_summary(&mut out, report.acquisition());
    out
}

fn acquisition_summary(out: &mut String, counters: &taloon::report::AcquisitionSummary) {
    let _ = write!(
        out,
        "requests={}, downloaded_bytes={}, archive_extractions={}, cache_hits={}, elapsed_us={}, index_cache_hits={}, index_wait_us={}, index_parses={}, index_parse_us={}",
        counters.network_requests,
        counters.downloaded_bytes,
        counters.archive_extractions,
        counters.cache_hits,
        counters.elapsed_us,
        counters.index_cache_hits,
        counters.index_wait_us,
        counters.index_parse_count,
        counters.index_parse_us
    );
    for (name, phase) in [
        ("index", &counters.http.index),
        ("metadata", &counters.http.metadata),
        ("archive", &counters.http.archive),
    ] {
        if phase.requests > 0 {
            let _ = write!(
                out,
                "\nhttp_{name}: requests={}, downloaded_bytes={}, request_time_us={}, retry_delay_us={}",
                phase.requests, phase.downloaded_bytes, phase.request_time_us, phase.retry_delay_us
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_netrc_path_precedes_home_and_missing_explicit_does_not_fall_back() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(
            temp.path().join(".netrc"),
            "machine home login home password token",
        )
        .unwrap();
        let explicit = temp.path().join("selected");
        assert_eq!(
            selected_netrc(Some(explicit.clone()), Some(temp.path())),
            Some(explicit)
        );
        assert_eq!(
            selected_netrc(None, Some(temp.path())),
            Some(temp.path().join(".netrc"))
        );
    }
}

#[cfg(test)]
mod text_tests;
