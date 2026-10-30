#![no_main]
mod support;
use libfuzzer_sys::fuzz_target;
use taloon::{
    evidence::{Evidence, SidecarAssociation},
    model::{ArtifactFormat, ContentHash},
};
fuzz_target!(|bytes: &[u8]| {
    let req = support::request(b"archive", ArtifactFormat::Wheel);
    let hash = req.hash().unwrap();
    if let Ok(e) = Evidence::from_sidecar(
        &req,
        SidecarAssociation::new(hash, hash, ContentHash::digest(bytes)).unwrap(),
        bytes,
    ) {
        support::roundtrip(&req, &e);
    }
});
