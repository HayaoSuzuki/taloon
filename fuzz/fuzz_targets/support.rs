#![allow(dead_code)]
use std::io::{Cursor, Write};
use taloon::{
    evidence::Evidence,
    model::{
        ArtifactFormat, ArtifactLocator, ArtifactRequest, ContentHash, EvidenceRequirement,
        PackageId, SourceId,
    },
};
pub fn request(bytes: &[u8], format: ArtifactFormat) -> ArtifactRequest {
    ArtifactRequest::new(
        PackageId::parse(
            "example",
            "1.0",
            SourceId::parse("registry", "https://example.test/simple").unwrap(),
        )
        .unwrap(),
        ArtifactLocator::registry(
            "https://example.test/simple",
            "https://example.test/example.whl",
        )
        .unwrap(),
        format,
        Some(ContentHash::digest(bytes)),
    )
    .unwrap()
}
pub fn roundtrip(request: &ArtifactRequest, evidence: &Evidence) {
    let replay = Evidence::from_record(request, evidence.to_record()).unwrap();
    assert_eq!(
        replay.license().map(|l| l.text().to_owned()),
        evidence.license().map(|l| l.text().to_owned())
    );
    assert_eq!(
        replay.requirement_satisfied(EvidenceRequirement::ArchiveVerified),
        evidence.requirement_satisfied(EvidenceRequirement::ArchiveVerified)
    );
}
pub fn inspect(bytes: &[u8], format: ArtifactFormat) {
    let request = request(bytes, format);
    if let Ok(e) = Evidence::from_archive(&request, bytes) {
        roundtrip(&request, &e);
    }
}
pub fn structured(bytes: &[u8], format: ArtifactFormat) {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return;
    };
    let metadata = format!(
        "Metadata-Version: 2.4\nName: example\nVersion: 1.0\nLicense-Expression: {text}\n\n"
    );
    let archive = match format {
        ArtifactFormat::Wheel => {
            let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
            writer
                .start_file(
                    "example-1.0.dist-info/METADATA",
                    zip::write::SimpleFileOptions::default(),
                )
                .unwrap();
            writer.write_all(metadata.as_bytes()).unwrap();
            writer.finish().unwrap().into_inner()
        }
        ArtifactFormat::Sdist => {
            let mut writer = tar::Builder::new(flate2::write::GzEncoder::new(
                Vec::new(),
                flate2::Compression::fast(),
            ));
            let mut header = tar::Header::new_gnu();
            header.set_size(u64::try_from(metadata.len()).unwrap());
            header.set_mode(0o644);
            header.set_cksum();
            writer
                .append_data(&mut header, "example-1.0/PKG-INFO", Cursor::new(metadata))
                .unwrap();
            writer.into_inner().unwrap().finish().unwrap()
        }
    };
    inspect(&archive, format);
}
