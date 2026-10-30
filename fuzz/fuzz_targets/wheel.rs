#![no_main]
mod support;
use libfuzzer_sys::fuzz_target;
use taloon::model::ArtifactFormat;
fuzz_target!(|bytes: &[u8]| {
    support::inspect(bytes, ArtifactFormat::Wheel);
    support::structured(bytes, ArtifactFormat::Wheel);
});
