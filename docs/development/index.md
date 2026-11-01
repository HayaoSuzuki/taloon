# Development

This is the entry point for developers and AI agents changing taloon.
For product usage, start with [the user guide](../user/index.md).

| Task | Start here |
| --- | --- |
| Understand decisions and implementation boundaries | [OKF catalog](knowledge/index.md) |
| Build and validate Rust changes | Setup and local checks below |
| Build Python wheels or publish/recover a release | [Release procedure](releases.md) |
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
Commit `Cargo.lock` whenever dependencies change. The package provides a pure
inspection/description library and the `check`, `describe` and `cache clear` commands; crates.io publishing is disabled. Follow the
[user guide](../user/index.md) for installation, runtime options, and supported
formats, or the [baseline](knowledge/project-baseline.md) for verification scope.

Install [prek](https://prek.j178.dev/) 0.5.7 (also required by the workflow
integration test) and enable the repository hooks:

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
the initial Lean corpus through public APIs. Binary and process tests additionally exercise acquisition, caching, and CLI
behavior.
The Python tooling tests exercise documentation validation. Run those and the
strict user-site build using the [documentation procedure](documentation.md).

## Python quality

Install the frozen docs and release groups using the
[documentation prerequisites](documentation.md#validate-and-preview), then run:

```sh
uv run --frozen --no-sync ruff format --check .
uv run --frozen --no-sync ruff check --no-fix .
uv run --frozen --no-sync ty check
uv run --frozen --no-sync pytest
```

Use `ruff format .` to apply formatting. CI and local prek hooks check formatting,
Ruff's `ALL` rule selection, and ty's all-error strict analysis. Type-ignore
comments cannot suppress ty diagnostics. Ruff exceptions are limited to two
formatter conflicts, standalone test conventions, and individually explained
boundary operations. Generated output and synthetic fixture data are excluded;
maintained tooling and tests are checked. The frozen lock pins the tool versions;
review new diagnostics when updating them.

## Test design

Apply these requirements to Rust and Python tests:

- Separate Arrange, Act and Assert. Keep setup before the operation being tested
  and inspect its observable result afterward; expected failures may combine Act
  and Assert in an exception/panic matcher.
- Give each test function one scenario or property. Multiple assertions may
  describe that single outcome; split unrelated behavior and repeated operations
  into separate tests. Shared setup belongs in fixtures/helpers.
- Use named `rstest` cases in Rust and `pytest.mark.parametrize` cases in Python
  for input variants. Each case must run independently; do not hide cases inside
  a loop in a test body. Fixture construction loops remain appropriate.
- Generalize meaningful fixed case domains with `proptest` or Hypothesis and an
  independent oracle or invariant. Keep readable boundary regressions. Do not
  replace expected values with a duplicate of the implementation.

The policy tests demonstrate fixed permission/expiry cases and generated Boolean
permission semantics. Python release tests combine version boundary examples
with a generated numeric oracle. These examples guide new and modified tests;
this change does not claim to rewrite every pre-existing Rust test. Test ordering
checks below complement behavioral and generated coverage.

## Randomized test order

CI adds a separate pinned nightly job alongside the stable Rust checks, following
[hoimin's shuffle job](https://github.com/HayaoSuzuki/hoimin/blob/39d35f3980c428ab15258b519a233b86f1551ffa/.github/workflows/ci.yml).
The standard harness shuffles tests within each executable and prints its seed:

```sh
rustup toolchain install nightly-2026-10-08 --profile minimal
cargo +nightly-2026-10-08 test --locked --workspace --all-features -- -Z unstable-options --shuffle
```

Replay a reported seed with the same nightly, revision and test selection:

```sh
cargo +nightly-2026-10-08 test --locked --workspace --all-features -- -Z unstable-options --shuffle-seed 12345
```

Different executables can report different generated seeds. Narrow a reproduction
with Cargo's `--test NAME` or `--bin NAME` when needed. A seed reproduces harness
ordering, not arbitrary parallel completion/interleaving; append
`--test-threads=1` when investigating a serial ordering dependency. Cargo's order
of test executables is not shuffled. Ignored benchmarks/TLS fixtures remain
ignored here; their existing explicit checks are unchanged.

The docs/test dependency group includes `pytest-randomly`, which pytest requires
to load. Ordinary pytest commands randomize collection and reseed Python's random
generator automatically. Use normal verbosity to see the session seed; install
the [documentation prerequisites](documentation.md#validate-and-preview) first.

```sh
uv run --frozen --no-sync pytest
uv run --frozen --no-sync pytest --randomly-seed=12345
uv run --frozen --no-sync pytest --randomly-seed=last
```

`last` uses pytest's local cache; CI has no cross-run last-seed guarantee. Keep the
seed printed in the failing CI session and pass that number explicitly to replay.
These tools are development-only and do not change the installed taloon command.

For workflow checks, install uv and the docs environment using the
[documentation procedure](documentation.md), then run:

```sh
tools/check_workflows.sh
uvx zizmor==1.30.1 --no-online-audits --min-severity low .github/workflows
```

For dependency checks:

```sh
cargo install cargo-audit --version 0.22.2 --locked
cargo install cargo-deny --version 0.20.2 --locked
cargo audit --file Cargo.lock
cargo deny --workspace --all-features --locked --config deny.toml check --deny warnings licenses sources
cargo fetch --locked --manifest-path fuzz/Cargo.toml
tools/check_fuzz_licenses.sh
```

`deny.toml` permits Elastic-2.0 for taloon itself and checks third-party licenses
separately. Review the policy when adding dependencies or changing versions.
The fuzz policy helper runs offline. Fetch its separate locked dependency graph
first; the dependency-audit CI job performs this preparation on a fresh runner.

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

CI is configured for Ubuntu. The local final verification used macOS arm64;
configuration and local checks are not evidence of an unobserved GitHub run.
The [stage reviews](work/initial-design/implementation-reviews.md) and
[performance record](work/initial-design/performance.md) state the verified scope.
Lean checks corpus freshness; Rust tests execute its public-API adapter. A pinned
nightly runs all six bounded fuzz targets, and dependency checks cover the
shipping and separate fuzz graphs. Neither Lean nor fuzz tools are runtime
requirements.

The separate Documentation workflow validates OKF, repository document links,
and the user-site build, and retains the generated HTML for review.

The [release procedure](releases.md) describes the Python wheel pipeline and
required PyPI/TestPyPI publishers. crates.io publishing stays disabled. Branch
protection is separate repository administration.
