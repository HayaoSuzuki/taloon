#![no_main]
use libfuzzer_sys::fuzz_target;
use taloon::{
    model::EvaluationDate,
    policy::{LicenseExpression, Policy},
};

fuzz_target!(|bytes: &[u8]| {
    if let Ok(text) = std::str::from_utf8(bytes) {
        let _ = Policy::from_pyproject(text, EvaluationDate::parse("2026-10-30").unwrap());
        let _ = LicenseExpression::parse(text);
    }
});
