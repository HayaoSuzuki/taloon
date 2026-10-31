#![allow(dead_code)]
use std::io::{Cursor, Write};
use taloon::model::{
    ArtifactFormat, ArtifactLocator, ArtifactRequest, ContentHash, PackageId, SourceId,
};

pub fn metadata(fields: &str) -> Vec<u8> {
    format!("Metadata-Version: 2.4\nName: example\nVersion: 1.0\n{fields}\n\nDescription.\n")
        .into_bytes()
}
pub fn wheel(metadata: &[u8]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .start_file(
            "example-1.0.dist-info/METADATA",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
    writer.write_all(metadata).unwrap();
    writer.finish().unwrap().into_inner()
}
pub fn wheel_with_two_metadata(metadata: &[u8]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for name in [
        "example-1.0.dist-info/METADATA",
        "another-1.0.dist-info/METADATA",
    ] {
        writer
            .start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(metadata).unwrap();
    }
    writer.finish().unwrap().into_inner()
}
pub fn sdist(metadata: &[u8]) -> Vec<u8> {
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    let mut writer = tar::Builder::new(encoder);
    let mut header = tar::Header::new_gnu();
    header.set_mode(0o644);
    header.set_size(u64::try_from(metadata.len()).unwrap());
    header.set_cksum();
    writer
        .append_data(&mut header, "example-1.0/PKG-INFO", Cursor::new(metadata))
        .unwrap();
    writer.into_inner().unwrap().finish().unwrap()
}
pub fn request(bytes: &[u8], format: ArtifactFormat) -> ArtifactRequest {
    ArtifactRequest::new(
        PackageId::parse(
            "example",
            "1.0",
            SourceId::parse("registry", "https://pypi.org/simple").unwrap(),
        )
        .unwrap(),
        ArtifactLocator::registry(
            "https://pypi.org/simple",
            "https://files.example.test/example.whl",
        )
        .unwrap(),
        format,
        Some(ContentHash::digest(bytes)),
    )
    .unwrap()
}
