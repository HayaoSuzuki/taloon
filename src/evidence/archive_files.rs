//! Bounded, exact-path license text observations from validated distribution bytes.
use super::{ArtifactFormat, Code, Error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LicenseFile {
    pub path: String,
    pub text: String,
}

pub(super) const MAX_LICENSE_BYTES: usize = 256 * 1024;
pub(super) const MAX_LICENSE_TOTAL: usize = 1024 * 1024;
pub(super) const MAX_LICENSE_FILES: usize = 32;
pub(super) const MAX_LICENSE_PATH_BYTES: usize = 4096;

use std::{
    collections::BTreeSet,
    io::{Cursor, Read},
};

fn failure() -> Error {
    Error::new(
        Code::UnknownLicense,
        "bundled license files are missing, ambiguous, invalid, or exceed reading limits",
    )
}

fn valid_path(path: &str) -> bool {
    !path.contains(['\\', ':'])
        && !path.chars().any(char::is_control)
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

fn conventional(path: &str) -> bool {
    let base = path
        .strip_suffix(".txt")
        .or_else(|| path.strip_suffix(".md"))
        .unwrap_or(path);
    matches!(base, "LICENSE" | "LICENCE" | "COPYING")
}

// A scan never retains unrelated members. Tar scans drain the capped decoder,
// including bytes after the end marker, so padding cannot bypass the cap.
fn scan(
    bytes: &[u8],
    format: ArtifactFormat,
    mut visit: impl FnMut(&str, bool, u64, &mut dyn Read) -> Result<(), Error>,
) -> Result<(), Error> {
    // zip 9 decodes every name byte sequence as UTF-8 or total CP437.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn invalid_name(_: zip::result::ZipError) -> Error {
        failure()
    }
    // entries() only rejects a nonzero archive cursor; scan always opens a fresh archive.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn invalid_archive(_: std::io::Error) -> Error {
        failure()
    }
    match format {
        ArtifactFormat::Wheel => {
            let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| failure())?;
            if archive.has_duplicate_names() {
                return Err(failure());
            }
            for i in 0..archive.len() {
                let mut file = archive.by_index(i).map_err(|_| failure())?;
                let name = file.name().map_err(invalid_name)?.into_owned();
                let regular = !file.is_dir()
                    && file
                        .unix_mode()
                        .is_none_or(|mode| matches!(mode & 0o170_000, 0 | 0o100_000));
                let size = file.size();
                visit(&name, regular, size, &mut file)?;
            }
        }
        ArtifactFormat::Sdist => {
            let decoder = flate2::read::MultiGzDecoder::new(Cursor::new(bytes));
            let mut archive = tar::Archive::new(decoder.take(super::MAX_TAR_EXPANSION + 1));
            for entry in archive.entries().map_err(invalid_archive)? {
                let mut entry = entry.map_err(|_| failure())?;
                let path = entry.path_bytes().into_owned();
                let name = std::str::from_utf8(&path)
                    .map_err(|_| failure())?
                    .trim_start_matches("./");
                let regular = entry.header().entry_type().is_file();
                let size = entry.size();
                visit(name, regular, size, &mut entry)?;
            }
            let mut limited = archive.into_inner();
            std::io::copy(&mut limited, &mut std::io::sink()).map_err(|_| failure())?;
            if limited.limit() == 0 {
                return Err(failure());
            }
        }
    }
    Ok(())
}

// Validate advertised sizes separately from archive traversal; callers bound allocation first.
fn read_sized_text(reader: &mut dyn Read, size: usize) -> Result<Vec<u8>, Error> {
    let mut text = vec![0; size];
    reader.read_exact(&mut text).map_err(|_| failure())?;
    if reader.read(&mut [0]).map_err(|_| failure())? != 0 {
        return Err(failure());
    }
    Ok(text)
}

/// Read declared files, or a narrow set of conventional names when undeclared.
/// `root` is the validated metadata directory with its trailing slash. One scan
/// rechecks its unique metadata member and reads selected text; observation paths
/// are relative to the distribution's license root.
pub(super) fn read(
    bytes: &[u8],
    format: ArtifactFormat,
    declared: &[String],
    root: &str,
) -> Result<Vec<LicenseFile>, Error> {
    // Every u64 fits usize on the supported 64-bit targets; retain the guard for narrower ports.
    #[cfg_attr(all(coverage_nightly, target_pointer_width = "64"), coverage(off))]
    fn invalid_size(_: std::num::TryFromIntError) -> Error {
        failure()
    }
    if declared.len() > MAX_LICENSE_FILES {
        return Err(failure());
    }
    let mut wanted = BTreeSet::new();
    for path in declared {
        if path.len() > MAX_LICENSE_PATH_BYTES || !valid_path(path) || !wanted.insert(path.as_str())
        {
            return Err(failure());
        }
    }
    let root_name = root.strip_suffix('/').ok_or_else(failure)?;
    if !valid_path(root_name) || root_name.contains('/') {
        return Err(failure());
    }
    let metadata_name = match format {
        ArtifactFormat::Wheel => "METADATA",
        ArtifactFormat::Sdist => "PKG-INFO",
    };
    let mut metadata_seen = false;
    let mut files = Vec::new();
    let mut seen = BTreeSet::new();
    let mut total = 0;
    scan(bytes, format, |name, regular, size, reader| {
        if super::member_path(name, format) {
            if metadata_seen
                || !regular
                || !valid_path(name)
                || name.strip_prefix(root) != Some(metadata_name)
            {
                return Err(failure());
            }
            metadata_seen = true;
        }
        let Some(relative) = name.strip_prefix(root) else {
            return Ok(());
        };
        let modern = (format == ArtifactFormat::Wheel)
            .then(|| relative.strip_prefix("licenses/"))
            .flatten();
        let relative = if declared.is_empty() {
            let candidate = modern.unwrap_or(relative);
            if !conventional(candidate) {
                return Ok(());
            }
            candidate
        } else {
            match (
                modern.filter(|path| wanted.contains(path)),
                wanted.contains(relative),
            ) {
                (Some(_), true) => return Err(failure()),
                (Some(path), false) => path,
                (None, true) => relative,
                (None, false) => return Ok(()),
            }
        };
        if !regular
            || relative.len() > MAX_LICENSE_PATH_BYTES
            || !valid_path(relative)
            || !seen.insert(relative.to_owned())
            || files.len() >= MAX_LICENSE_FILES
        {
            return Err(failure());
        }
        let size = usize::try_from(size).map_err(invalid_size)?;
        if size > MAX_LICENSE_BYTES || size > MAX_LICENSE_TOTAL - total {
            return Err(failure());
        }
        // The advertised size is bounded before allocation. Probe once after
        // reading it to force ZIP integrity checks and reject a size mismatch.
        let text = read_sized_text(reader, size)?;
        total += size;
        let text = String::from_utf8(text).map_err(|_| failure())?;
        files.push(LicenseFile {
            path: relative.to_owned(),
            text,
        });
        Ok(())
    })?;
    if !metadata_seen || (!declared.is_empty() && files.len() != declared.len()) {
        return Err(failure());
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::io::{Cursor, Write};

    fn read(
        bytes: &[u8],
        format: ArtifactFormat,
        declared: &[String],
    ) -> Result<Vec<LicenseFile>, Error> {
        let root = match format {
            ArtifactFormat::Wheel => "example.dist-info/",
            ArtifactFormat::Sdist => "example/",
        };
        super::read(bytes, format, declared, root)
    }

    #[rstest]
    #[case::foreign("other/")]
    #[case::traversal("../example/")]
    #[case::nested("nested/example/")]
    #[case::missing_slash("example")]
    #[case::empty("")]
    fn rejects_invalid_supplied_root(#[case] root: &str) {
        // Arrange
        let bytes = archive(
            ArtifactFormat::Sdist,
            &[("example/LICENSE", b"text", false)],
        );
        // Act
        let result = super::read(&bytes, ArtifactFormat::Sdist, &["LICENSE".into()], root);
        // Assert
        assert!(result.is_err());
    }

    fn archive(format: ArtifactFormat, entries: &[(&str, &[u8], bool)]) -> Vec<u8> {
        let metadata = match format {
            ArtifactFormat::Wheel => "example.dist-info/METADATA",
            ArtifactFormat::Sdist => "example/PKG-INFO",
        };
        let mut members = vec![(metadata, b"Metadata-Version: 2.4\n".as_slice(), false)];
        members.extend_from_slice(entries);
        raw_archive(format, &members)
    }

    fn raw_archive(format: ArtifactFormat, entries: &[(&str, &[u8], bool)]) -> Vec<u8> {
        match format {
            ArtifactFormat::Wheel => {
                let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
                for (path, bytes, link) in entries {
                    if *link {
                        writer
                            .add_symlink(
                                *path,
                                "elsewhere",
                                zip::write::SimpleFileOptions::default(),
                            )
                            .unwrap();
                    } else {
                        writer
                            .start_file(*path, zip::write::SimpleFileOptions::default())
                            .unwrap();
                        writer.write_all(bytes).unwrap();
                    }
                }
                writer.finish().unwrap().into_inner()
            }
            ArtifactFormat::Sdist => {
                let encoder =
                    flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
                let mut writer = tar::Builder::new(encoder);
                for &(path, bytes, link) in entries {
                    let mut header = tar::Header::new_gnu();
                    header.set_size(if link { 0 } else { bytes.len() as u64 });
                    header.set_mode(0o644);
                    if link {
                        header.set_entry_type(tar::EntryType::Symlink);
                        header.set_link_name("elsewhere").unwrap();
                    }
                    header.set_cksum();
                    writer
                        .append_data(
                            &mut header,
                            path,
                            Cursor::new(if link { &[] } else { bytes }),
                        )
                        .unwrap();
                }
                writer.into_inner().unwrap().finish().unwrap()
            }
        }
    }

    #[rstest]
    #[case::wheel(ArtifactFormat::Wheel, "example.dist-info/licenses/LICENSE")]
    #[case::sdist(ArtifactFormat::Sdist, "example/LICENSE")]
    fn rejects_missing_metadata_member(#[case] format: ArtifactFormat, #[case] path: &str) {
        // Arrange
        let bytes = raw_archive(format, &[(path, b"text", false)]);
        // Act
        let result = read(&bytes, format, &["LICENSE".into()]);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn rejects_duplicate_metadata_member() {
        // Arrange
        let bytes = archive(
            ArtifactFormat::Sdist,
            &[("example/PKG-INFO", b"metadata", false)],
        );
        // Act
        let result = read(&bytes, ArtifactFormat::Sdist, &[]);
        // Assert
        assert!(result.is_err());
    }

    #[rstest]
    #[case::wheel(ArtifactFormat::Wheel, "example.dist-info/licenses/legal/custom.txt")]
    #[case::sdist(ArtifactFormat::Sdist, "example/legal/custom.txt")]
    fn reads_exact_declared_text(#[case] format: ArtifactFormat, #[case] path: &str) {
        // Arrange
        let bytes = archive(format, &[(path, b"license text", false)]);
        // Act
        let files = read(&bytes, format, &["legal/custom.txt".into()]).unwrap();
        // Assert
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "legal/custom.txt");
        assert_eq!(files[0].text, "license text");
    }
    #[rstest]
    #[case::wheel(ArtifactFormat::Wheel, "example.dist-info/licenses/LICENSE")]
    #[case::sdist(ArtifactFormat::Sdist, "example/LICENSE")]
    fn discovers_conventional_root(#[case] format: ArtifactFormat, #[case] path: &str) {
        // Arrange
        let bytes = archive(format, &[(path, b"license", false)]);
        // Act
        let files = read(&bytes, format, &[]).unwrap();
        // Assert
        assert_eq!(files[0].text, "license");
    }

    #[rstest]
    #[case::empty("")]
    #[case::parent("../LICENSE")]
    #[case::embedded_parent("legal/../LICENSE")]
    #[case::absolute("/LICENSE")]
    #[case::backslash(r"legal\LICENSE")]
    #[case::dot("./LICENSE")]
    #[case::empty_segment("legal//LICENSE")]
    #[case::control("LICENSE\n")]
    #[case::drive("C:/LICENSE")]
    fn rejects_unsafe_declarations(#[case] path: &str) {
        // Arrange
        let bytes = archive(ArtifactFormat::Wheel, &[]);
        // Act
        let result = read(&bytes, ArtifactFormat::Wheel, &[path.into()]);
        // Assert
        assert!(result.is_err());
    }

    #[rstest]
    #[case::wheel(ArtifactFormat::Wheel, "example.dist-info/licenses/LICENSE")]
    #[case::sdist(ArtifactFormat::Sdist, "example/LICENSE")]
    fn rejects_selected_symlink(#[case] format: ArtifactFormat, #[case] path: &str) {
        // Arrange
        let bytes = archive(format, &[(path, b"", true)]);
        // Act
        let result = read(&bytes, format, &["LICENSE".into()]);
        // Assert
        assert!(result.is_err());
    }

    #[rstest]
    #[case::wheel(ArtifactFormat::Wheel)]
    #[case::sdist(ArtifactFormat::Sdist)]
    fn rejects_missing_declaration(#[case] format: ArtifactFormat) {
        // Arrange
        let bytes = archive(format, &[]);
        // Act
        let result = read(&bytes, format, &["LICENSE".into()]);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn rejects_duplicate_selected_tar_member() {
        // Arrange
        let bytes = archive(
            ArtifactFormat::Sdist,
            &[
                ("example/LICENSE", b"a", false),
                ("example/LICENSE", b"b", false),
            ],
        );
        // Act
        let result = read(&bytes, ArtifactFormat::Sdist, &["LICENSE".into()]);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn rejects_modern_legacy_ambiguity() {
        // Arrange
        let bytes = archive(
            ArtifactFormat::Wheel,
            &[
                ("example.dist-info/licenses/LICENSE", b"a", false),
                ("example.dist-info/LICENSE", b"b", false),
            ],
        );
        // Act
        let result = read(&bytes, ArtifactFormat::Wheel, &["LICENSE".into()]);
        // Assert
        assert!(result.is_err());
    }

    #[rstest]
    #[case::wheel(ArtifactFormat::Wheel, "example.dist-info/licenses/LICENSE")]
    #[case::sdist(ArtifactFormat::Sdist, "example/LICENSE")]
    fn rejects_oversize_text(#[case] format: ArtifactFormat, #[case] path: &str) {
        // Arrange
        let text = vec![b'a'; 256 * 1024 + 1];
        let bytes = archive(format, &[(path, &text, false)]);
        // Act
        let result = read(&bytes, format, &["LICENSE".into()]);
        // Assert
        assert!(result.is_err());
    }

    #[rstest]
    #[case::wheel(ArtifactFormat::Wheel, "example.dist-info/licenses/LICENSE")]
    #[case::sdist(ArtifactFormat::Sdist, "example/LICENSE")]
    fn ignores_unselected_source(#[case] format: ArtifactFormat, #[case] path: &str) {
        // Arrange
        let bytes = archive(
            format,
            &[
                (path, b"license", false),
                ("example/src/LICENSE", &[255], false),
            ],
        );
        // Act
        let files = read(&bytes, format, &[]).unwrap();
        // Assert
        assert_eq!(files.len(), 1);
    }

    #[rstest]
    #[case::legacy("example.dist-info/LICENSE", "LICENSE")]
    #[case::nested_legacy("example.dist-info/licenses/custom.txt", "licenses/custom.txt")]
    #[case::authors("example.dist-info/licenses/AUTHORS", "AUTHORS")]
    fn reads_legacy_or_attribution_paths(#[case] path: &str, #[case] declared: &str) {
        // Arrange
        let bytes = archive(ArtifactFormat::Wheel, &[(path, b"text", false)]);
        // Act
        let files = read(&bytes, ArtifactFormat::Wheel, &[declared.into()]).unwrap();
        // Assert
        assert_eq!(files[0].path, declared);
    }

    #[rstest]
    #[case::at_limit(4096, true)]
    #[case::over_limit(4097, false)]
    fn relative_path_byte_bound(#[case] length: usize, #[case] accepted: bool) {
        // Arrange
        let declared = "a".repeat(length);
        let member = format!("example/{declared}");
        let bytes = archive(ArtifactFormat::Sdist, &[(member.as_str(), b"text", false)]);
        // Act
        let result = read(
            &bytes,
            ArtifactFormat::Sdist,
            std::slice::from_ref(&declared),
        );
        // Assert
        assert_eq!(result.is_ok(), accepted);
        if accepted {
            assert_eq!(result.unwrap()[0].path, declared);
        } else {
            assert_eq!(result.unwrap_err().code, Code::UnknownLicense);
        }
    }

    #[rstest]
    #[case::at_limit(256 * 1024, true)]
    #[case::over_limit(256 * 1024 + 1, false)]
    fn per_file_bound(#[case] length: usize, #[case] accepted: bool) {
        // Arrange
        let text = vec![b'a'; length];
        let bytes = archive(ArtifactFormat::Sdist, &[("example/LICENSE", &text, false)]);
        // Act
        let result = read(&bytes, ArtifactFormat::Sdist, &["LICENSE".into()]);
        // Assert
        assert_eq!(result.is_ok(), accepted);
    }

    #[rstest]
    #[case::at_limit(4, true)]
    #[case::over_limit(5, false)]
    fn aggregate_bound(#[case] count: usize, #[case] accepted: bool) {
        // Arrange
        let names: Vec<_> = (0..count).map(|i| format!("license{i}")).collect();
        let paths: Vec<_> = names.iter().map(|name| format!("example/{name}")).collect();
        let text = vec![b'a'; 256 * 1024];
        let entries: Vec<_> = paths
            .iter()
            .map(|path| (path.as_str(), text.as_slice(), false))
            .collect();
        let bytes = archive(ArtifactFormat::Sdist, &entries);
        // Act
        let result = read(&bytes, ArtifactFormat::Sdist, &names);
        // Assert
        assert_eq!(result.is_ok(), accepted);
    }

    #[rstest]
    #[case::at_limit(32, true)]
    #[case::over_limit(33, false)]
    fn count_bound(#[case] count: usize, #[case] accepted: bool) {
        // Arrange
        let names: Vec<_> = (0..count).map(|i| format!("license{i}")).collect();
        let paths: Vec<_> = names.iter().map(|name| format!("example/{name}")).collect();
        let entries: Vec<_> = paths
            .iter()
            .map(|path| (path.as_str(), b"text".as_slice(), false))
            .collect();
        let bytes = archive(ArtifactFormat::Sdist, &entries);
        // Act
        let result = read(&bytes, ArtifactFormat::Sdist, &names);
        // Assert
        assert_eq!(result.is_ok(), accepted);
    }

    #[test]
    fn rejects_duplicate_declarations() {
        // Arrange
        let bytes = archive(
            ArtifactFormat::Sdist,
            &[("example/LICENSE", b"text", false)],
        );
        // Act
        let result = read(
            &bytes,
            ArtifactFormat::Sdist,
            &["LICENSE".into(), "LICENSE".into()],
        );
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn rejects_non_utf8_text() {
        // Arrange
        let bytes = archive(ArtifactFormat::Sdist, &[("example/LICENSE", &[255], false)]);
        // Act
        let result = read(&bytes, ArtifactFormat::Sdist, &["LICENSE".into()]);
        // Assert
        assert!(result.is_err());
    }

    #[rstest]
    #[case::wheel(ArtifactFormat::Wheel, "other.dist-info/METADATA")]
    #[case::sdist(ArtifactFormat::Sdist, "other/PKG-INFO")]
    fn rejects_ambiguous_metadata_root(#[case] format: ArtifactFormat, #[case] path: &str) {
        // Arrange
        let bytes = archive(format, &[(path, b"metadata", false)]);
        // Act
        let result = read(&bytes, format, &[]);
        // Assert
        assert!(result.is_err());
    }

    proptest::proptest! {
        #[test]
        fn preserves_generated_utf8_text(
            text in ".{0,1024}",
            (format, path) in proptest::sample::select(vec![
                (ArtifactFormat::Wheel, "example.dist-info/licenses/LICENSE"),
                (ArtifactFormat::Sdist, "example/LICENSE"),
            ]),
        ) {
            // Arrange
            let bytes = archive(format, &[(path, text.as_bytes(), false)]);
            // Act
            let files = read(&bytes, format, &["LICENSE".into()]).unwrap();
            // Assert
            proptest::prop_assert_eq!(&files[0].text, &text);
        }
    }
    #[rstest]
    #[case::wheel(ArtifactFormat::Wheel)]
    #[case::sdist(ArtifactFormat::Sdist)]
    fn malformed_archive_cannot_provide_license_text(#[case] format: ArtifactFormat) {
        // Arrange / Act
        let result = read(b"not an archive", format, &["LICENSE".into()]);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn declarations_cannot_select_ambiguous_modern_and_legacy_name() {
        // Arrange
        let bytes = archive(
            ArtifactFormat::Wheel,
            &[("example.dist-info/licenses/LICENSE", b"text", false)],
        );
        // Act
        let result = read(
            &bytes,
            ArtifactFormat::Wheel,
            &["LICENSE".into(), "licenses/LICENSE".into()],
        );
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn corrupted_selected_zip_payload_is_rejected() {
        // Arrange
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        writer
            .start_file("example.dist-info/METADATA", options)
            .unwrap();
        writer.write_all(b"Metadata-Version: 2.4\n").unwrap();
        writer
            .start_file("example.dist-info/LICENSE", options)
            .unwrap();
        writer.write_all(b"unique license payload").unwrap();
        let mut bytes = writer.finish().unwrap().into_inner();
        let offset = bytes
            .windows(b"unique license payload".len())
            .position(|part| part == b"unique license payload")
            .unwrap();
        bytes[offset] ^= 1;
        // Act
        let result = read(&bytes, ArtifactFormat::Wheel, &["LICENSE".into()]);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn non_utf8_tar_member_name_is_rejected() {
        // Arrange
        let mut header = tar::Header::new_gnu();
        header.set_size(0);
        header.set_mode(0o644);
        header.as_mut_bytes()[0] = 255;
        header.set_cksum();
        let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        let mut writer = tar::Builder::new(encoder);
        writer.append(&header, std::io::empty()).unwrap();
        let bytes = writer.into_inner().unwrap().finish().unwrap();
        // Act
        let result = read(&bytes, ArtifactFormat::Sdist, &[]);
        // Assert
        assert!(result.is_err());
    }
    #[test]
    fn padding_cannot_bypass_license_scan_expansion_limit() {
        // Arrange
        let bytes = super::super::tests::expansion_limit_bytes();
        // Act
        let result = read(bytes, ArtifactFormat::Sdist, &[]);
        // Assert
        assert!(result.is_err());
    }
    #[rstest]
    #[case::truncated(b"".as_slice())]
    #[case::excess(b"ab".as_slice())]
    fn inconsistent_advertised_size_is_rejected(#[case] bytes: &[u8]) {
        // Arrange
        let mut reader = Cursor::new(bytes);
        // Act
        let result = read_sized_text(&mut reader, 1);
        // Assert
        assert!(result.is_err());
    }
}
