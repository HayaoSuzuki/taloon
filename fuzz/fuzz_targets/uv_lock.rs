#![no_main]
use libfuzzer_sys::fuzz_target;
use taloon::resolution::Resolution;

fuzz_target!(|bytes: &[u8]| {
    if let Ok(text) = std::str::from_utf8(bytes)
        && let Ok(resolution) = Resolution::from_uv_lock(text)
    {
        for target in resolution.targets() {
            let _ = resolution.paths(target.package());
        }
    }
});
