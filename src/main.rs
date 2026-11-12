#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod acquisition;
mod cli;
fn main() -> std::process::ExitCode {
    std::process::ExitCode::from(cli::run())
}

#[cfg(test)]
#[allow(dead_code)]
mod testfixtures {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/common/fixtures.rs"
    ));
}
