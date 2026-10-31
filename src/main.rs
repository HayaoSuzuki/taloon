mod acquisition;
mod cli;
#[cfg(test)]
#[path = "../tests/common/mod.rs"]
mod testfixtures;
fn main() -> std::process::ExitCode {
    std::process::ExitCode::from(cli::run())
}
