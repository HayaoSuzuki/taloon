//! Raw bounded archive observations, reinterpreted rather than caching a verdict.
use super::{
    archive_files::{self, LicenseFile},
    text_match,
};
use crate::{
    model::{ArtifactFormat, Code, ContentHash, Error},
    policy::LicenseExpression,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum FileObservation {
    Complete { files: Vec<LicenseFile> },
    Unavailable { message: String },
}

/// How a bundled file participates in conservative license inference.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LicenseFileKind {
    License,
    Attribution,
}

/// Archive-bound provenance of an observed file; full text remains in the cache.
#[derive(Clone, Debug, Serialize)]
pub struct LicenseFileSummary {
    pub path: String,
    pub content_hash: ContentHash,
    pub kind: LicenseFileKind,
    pub matched_expression: Option<&'static str>,
}

fn kind(path: &str) -> LicenseFileKind {
    let name = path.rsplit('/').next().unwrap_or(path);
    if ["AUTHORS", "AUTHORS.txt", "AUTHORS.md"]
        .iter()
        .any(|candidate| name.eq_ignore_ascii_case(candidate))
    {
        LicenseFileKind::Attribution
    } else {
        LicenseFileKind::License
    }
}
fn unknown(message: impl Into<String>) -> Error {
    Error::new(Code::UnknownLicense, message)
}
impl FileObservation {
    pub(super) fn read(
        bytes: &[u8],
        format: ArtifactFormat,
        declared: &[String],
        root: &str,
    ) -> Self {
        match archive_files::read(bytes, format, declared, root) {
            Ok(files) => Self::Complete { files },
            Err(error) => Self::Unavailable {
                message: error.message,
            },
        }
    }
    pub(super) fn valid(&self) -> bool {
        match self {
            Self::Unavailable { message } => message.len() <= 4096,
            Self::Complete { files } => {
                files.len() <= archive_files::MAX_LICENSE_FILES
                    && files.iter().all(|f| {
                        f.text.len() <= archive_files::MAX_LICENSE_BYTES
                            && f.path.len() <= archive_files::MAX_LICENSE_PATH_BYTES
                    })
                    && files.iter().map(|f| f.text.len()).sum::<usize>()
                        <= archive_files::MAX_LICENSE_TOTAL
            }
        }
    }
    pub(super) fn license(&self) -> Result<LicenseExpression, Error> {
        let files = match self {
            Self::Complete { files } => files,
            Self::Unavailable { message } => return Err(unknown(message.clone())),
        };
        let mut expression = None;
        for file in files {
            if matches!(kind(&file.path), LicenseFileKind::Attribution) {
                continue;
            }
            let matched = text_match::identify(&file.text)
                .ok_or_else(|| unknown("unrecognized bundled license text"))?;
            if expression.is_some_and(|previous| previous != matched) {
                return Err(unknown(
                    "bundled license files require different expressions",
                ));
            }
            expression = Some(matched);
        }
        LicenseExpression::parse(
            expression.ok_or_else(|| unknown("no recognizable bundled license file"))?,
        )
    }
    pub(super) fn summary(&self) -> Vec<LicenseFileSummary> {
        let Self::Complete { files } = self else {
            return Vec::new();
        };
        files
            .iter()
            .map(|file| LicenseFileSummary {
                path: file.path.clone(),
                content_hash: ContentHash::digest(file.text.as_bytes()),
                kind: kind(&file.path),
                matched_expression: if matches!(kind(&file.path), LicenseFileKind::License) {
                    text_match::identify(&file.text)
                } else {
                    None
                },
            })
            .collect()
    }
}
