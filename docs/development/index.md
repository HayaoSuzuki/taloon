# Development

This is the entry point for developers and AI agents changing taloon.
For product usage, start with [the user guide](../user/index.md).

| Task | Start here |
| --- | --- |
| Understand decisions and implementation boundaries | [OKF catalog](knowledge/index.md) |
| Build and validate Rust changes | Setup and local checks below |
| Write, reorganize, or preview documentation | [Documentation procedure](documentation.md) |

Read relevant OKF concepts and their cited sources before implementation.
Update the owning concept and user guidance when behavior or design decisions
change. See [AGENTS.md](../../AGENTS.md) for repository documentation rules.

The development configuration follows [hoimin](https://github.com/HayaoSuzuki/hoimin):
Rust 1.99.1, edition 2024, strict Clippy checks, prek hooks, SHA-pinned GitHub
Actions, and separate vulnerability and license/source checks.

## Setup

Install Rust with rustup, then run from the repository root:

```sh
rustup toolchain install
cargo build --locked --workspace
```

`rust-toolchain.toml` selects the same compiler for local development and CI.
Commit `Cargo.lock` whenever dependencies change. The package is a
library with publishing disabled. The implemented library provides
validated identities, pure policy evaluation, and version-1/revision-5 lock
inspection, verified archive/sidecar metadata, and complete report aggregation; it does not provide
a usable executable yet. See the [current baseline](knowledge/project-baseline.md).

Install [prek](https://prek.j178.dev/) and enable the local hooks:

```sh
prek install
prek run --all-files
```

Hooks run formatting and Clippy when Rust files or their configuration change.
They do not run the test suite; run that explicitly before submitting changes.

## Local checks

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
RUSTDOCFLAGS='-D warnings' cargo doc --locked --workspace --all-features --no-deps
```

Rust tests exercise policy, locked inventory/paths, generated properties, and
the initial Lean corpus through public APIs. In-memory metadata interpretation is implemented; acquisition and the CLI
remain implementation tasks.
The Python tooling tests exercise documentation validation. Run those and the
strict user-site build using the [documentation procedure](documentation.md).

For workflow checks, install uv and run:

```sh
uvx --from actionlint-py==1.7.12.25 --with shellcheck-py==0.11.0.1 actionlint
uvx zizmor==1.30.1 --no-online-audits --min-severity low .github/workflows
```

For dependency checks:

```sh
cargo install cargo-audit --version 0.22.2 --locked
cargo install cargo-deny --version 0.20.2 --locked
cargo audit --file Cargo.lock
cargo deny --workspace --all-features --locked --config deny.toml check --deny warnings licenses sources
```

`deny.toml` permits Elastic-2.0 for taloon itself and checks third-party licenses
separately. Review the policy when adding dependencies or changing versions.

## Formal and generated checks

The [verification decision](knowledge/verification-strategy.md) explains model
boundaries and correspondence. Install the pinned Lean toolchain in
`formal/TaloonOracle/lean-toolchain`; follow the
[formal reproduction procedure](../../formal/TaloonOracle/README.md).
The Rust test command above includes the generated-corpus adapter and proptest.

With cargo-fuzz 0.13.2 and nightly-2026-10-08 installed, run the initial smoke target:

```sh
cargo +nightly-2026-10-08 fuzz run policy -- -runs=1000 -max_len=4096
cargo +nightly-2026-10-08 fuzz run uv_lock -- -runs=1000 -max_len=4096
cargo +nightly-2026-10-08 fuzz run metadata -- -runs=1000 -max_len=4096
cargo +nightly-2026-10-08 fuzz run wheel -- -runs=1000 -max_len=4096
cargo +nightly-2026-10-08 fuzz run sdist -- -runs=1000 -max_len=4096
cargo +nightly-2026-10-08 fuzz run inspection -- -runs=1000 -max_len=4096
tools/check_fuzz_licenses.sh
```

The fuzz workspace stays separate from shipping dependencies. Its license check
derives the root policy plus exact development-only NCSA permission for
libfuzzer-sys 0.4.13; upgrading that package requires review. Lean/fuzz tools are
development prerequisites, not requirements for running taloon.

## CI and scope

CI runs on pull requests, main pushes, merge groups, and manual dispatch. It
checks Rust formatting, Clippy, tests, documentation, and workflow definitions.
The dependency workflow also runs daily. Jobs have explicit time limits and
read-only repository permissions.

Ubuntu is the initial CI environment. Passing these scaffold checks does not
establish dependency coverage or correctness of future license decisions.
Validate metadata acquisition, artifact coverage, policy semantics, and caching
during the Issue #1 research before documenting those as supported behavior.

The separate Documentation workflow validates OKF, repository document links,
and the user-site build, and retains the generated HTML for review.

Release automation, package publishing, and deployment are deferred. Branch
protection and repository settings are not changed by these files; configure
required checks after the workflows have run on GitHub.
