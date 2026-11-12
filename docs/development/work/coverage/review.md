# Source and test coverage verification

Measured on macOS arm64 on 2026-11-12, in the working tree based on `b004dd3`.
These results describe this uncommitted change, not the earlier baseline or a
hosted CI run. Commands and prerequisites belong to the
[development guide](../../index.md#coverage).

| Scope | Executed / mapped source lines | Coverage |
| --- | --- | --- |
| Rust workspace source | 5,348 / 5,348 | 100% |
| Rust test files and shared fixtures | 5,268 / 5,268 | 100% |
| Python tooling and tests | 2,047 / 2,047 | 100% |

Rust's separate native function summary is 1,155 / 1,155 (100%). All 1,432 Rust
test cases passed, including ignored measurements and the loopback TLS test.
Python's 242 tests passed, including child-process measurement. Rust used
cargo-llvm-cov 0.9.1 and nightly-2026-10-08; Python used the frozen pytest-cov
7.1.0 and coverage 7.17.2 environment.

## Metric discrepancy retained

The unmodified LLVM summary reports 10,779 / 10,821 lines (99.61%), while all
10,616 actual LCOV `DA` records have positive execution counts and the merged
source view lists no missing lines. These are different aggregation metrics.
Do not rewrite the raw `LF`/`LH` totals or describe the native summary as 100%.

[LLVM issue 119299](https://github.com/llvm/llvm-project/issues/119299)
describes the summary's max(covered)/max(total) strategy and its disagreement
with merged source views. LLVM's
[LCOV exporter](https://github.com/llvm/llvm-project/blob/main/llvm/tools/llvm-cov/CoverageExporterLcov.cpp)
exports line records and summary totals through separate paths. The developer
command gates actual mapped source lines and native function coverage. It does
not claim every monomorphized instance or every branch is covered.

## Behavioral checks and exclusions

Added expectations cover rejected inputs and identity binding; corrupted,
truncated, encrypted and oversized archives; signature verification and original
artifact preservation; publication/recovery refusal; cache replacement and
filesystem failures; terminal progress and closed stdout; sanitized diagnostics;
and real CLI module entrypoints. Tests assert outputs, errors, retained bytes,
request counts or state transitions rather than merely invoking code.

Small internal readers, error converters and rendering helpers expose meaningful
boundaries without adding test-only runtime switches. Shared fixtures use one
canonical include path so the same source is not reported under two filenames.
The concurrent cache test now starts with a published record and requires every
read during replacement to find a complete record.

Rust inline exclusions are restricted to fixed valid format parsing, validated
hexadecimal/ASCII/URL conversion, exactly-one-entry invariants, total ZIP name
decoding, fresh tar cursor initialization, primitive-record serialization, and
process-wide HTTPS client initialization failures. The u64-to-usize guard is
excluded only on 64-bit targets. Reasons accompany each attribute; ordinary
builds retain the defensive code. Python explicitly excludes the impossible
inventory page-type guard and a failure-only test callback; type-checking-only
imports are also outside runtime measurement. No file-wide coverage exclusions
were added for maintained source or tests.

## Other verification

Rust formatting, all-target/all-feature Clippy with warnings denied, release
build, and documentation generation with warnings denied passed. Python Ruff
format/check, strict ty, and repository documentation validation passed.
An isolated manual mutation reversing the documentation source-mapping guard
was rejected by 12 tests. This is one mutation check, not a mutation score.
The installed hoimin verification attempt did not complete successfully; it
encountered workspace lease issues, and no hoimin success is claimed.

User-facing behavior is unchanged, so no user-guide update was needed. The
existing verification decision records coverage scope and exclusion policy.
