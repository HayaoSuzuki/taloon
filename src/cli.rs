//! Runtime setup and rendering; inspection owns all verdict rules.
use crate::acquisition::{Acquirer, AcquisitionOptions};
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use std::{
    ffi::OsString,
    fmt::Write as _,
    io::{Read, Write as _},
    path::{Path, PathBuf},
};
use taloon::{
    inspection::Inspection,
    model::{Code, Error, EvaluationDate},
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
        /// Fail a complete check when it emits warnings.
        #[arg(long)]
        deny_warnings: bool,
    },
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
                Command::Check {
                    project,
                    format,
                    offline,
                    deny_warnings,
                },
        }) => {
            let report =
                check(project.as_deref(), offline).unwrap_or_else(Report::from_input_error);
            emit(&report, format, deny_warnings)
        }
        Err(error) if !error.use_stderr() => {
            if error.print().is_ok() {
                0
            } else {
                2
            }
        }
        Err(_) => emit(
            &Report::from_input_error(Error::new(
                Code::Configuration,
                "invalid command line; run taloon --help",
            )),
            format,
            false,
        ),
    }
}
fn check(project: Option<&Path>, offline: bool) -> Result<Report, Error> {
    let cwd = std::env::current_dir().map_err(|_| input("cannot determine current directory"))?;
    let root = project.map_or_else(|| discover(&cwd), |p| Ok(cwd.join(p)))?;
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
    let batch = acquirer.acquire(inspection.requests());
    Ok(inspection
        .evaluate(batch.results)
        .with_warnings(batch.warnings)
        .with_acquisition_summary(batch.summary))
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
        return Ok(PathBuf::from(path));
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
fn emit(report: &Report, format: Format, deny_warnings: bool) -> u8 {
    let code = report.exit_code(deny_warnings);
    let encoded = match format {
        Format::Json => serde_json::to_vec(&Envelope {
            report,
            exit_code: code,
        })
        .map_err(|_| ()),
        Format::Text => Ok(text_report(report).into_bytes()),
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
fn safe(text: &str) -> String {
    text.chars()
        .flat_map(|c| {
            if c.is_control() || matches!(c,'\u{202a}'..='\u{202e}'|'\u{2066}'..='\u{2069}') {
                c.escape_default().collect::<Vec<_>>()
            } else {
                vec![c]
            }
        })
        .collect()
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
fn text_report(report: &Report) -> String {
    let mut out = String::new();
    let artifacts: usize = report.packages().iter().map(|p| p.artifacts.len()).sum();
    let _ = writeln!(
        out,
        "taloon: {:?}; {} packages, {artifacts} artifacts",
        report.status(),
        report.packages().len()
    );
    for d in report.diagnostics() {
        diagnostic(&mut out, d);
    }
    for package in report.packages() {
        if package.status == Status::Compliant
            && package.artifacts.iter().all(|a| a.diagnostics.is_empty())
        {
            continue;
        }
        let _ = writeln!(out, "{:?}: {}", package.status, safe(&package.package));
        for d in &package.diagnostics {
            diagnostic(&mut out, d);
        }
        for artifact in &package.artifacts {
            let _ = writeln!(out, "  {:?}: {}", artifact.status, safe(&artifact.locator));
            if let Some(e) = &artifact.evidence {
                let _ = writeln!(out, "    {:?}, observed {}", e.strength, e.content_hash);
            }
            if let Some(expression) = &artifact.interpreted_expression {
                let _ = writeln!(out, "    license: {}", safe(expression));
            }
            if let Some(decision) = &artifact.decision {
                for applied in &decision.overrides {
                    let _ = writeln!(out, "    {:?}: {}", applied.kind, safe(&applied.reason));
                }
            }
            for d in &artifact.diagnostics {
                diagnostic(&mut out, d);
            }
        }
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
    let counters = report.acquisition();
    let _ = write!(
        out,
        "requests={}, downloaded_bytes={}, archive_extractions={}, cache_hits={}",
        counters.network_requests,
        counters.downloaded_bytes,
        counters.archive_extractions,
        counters.cache_hits
    );
    out
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
