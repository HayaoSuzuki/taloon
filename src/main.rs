#[cfg(test)]
mod acquisition;
#[cfg(test)]
#[path = "../tests/common/mod.rs"]
mod testfixtures;
fn main() -> std::process::ExitCode {
    eprintln!("taloon check is not available in this implementation stage");
    std::process::ExitCode::from(2)
}
