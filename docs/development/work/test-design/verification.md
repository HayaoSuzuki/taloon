# Final test-design audit verification

Verified on macOS arm64, 2026-11-14, in the uncommitted working tree based on
`f8e3127`. No commits were created. All changes affect tests, test support or
these development records; production behavior and coverage exclusions are
unchanged. Consequently no user guide or OKF contract amendment was needed.

## Audit scope and review

All 61 Rust files containing tests and all 11 Python test files have a recorded
outcome. Shared Rust fixtures were also inspected. Findings and deliberate
fixture, oracle and benchmark loops are recorded by scope:

- [Description, evidence and inspection](root-review.md)
- [Other Rust integration tests](integration-rust-review.md)
- [Rust inline tests and decision crate](inline-rust-review.md)
- [Python tests](python-review.md)

The audit separates independent scenarios and names parameter variants. Generated
tests use explicit invariants or independent oracles. Finite Boolean domains
may be exhaustively parameterized; a benchmark sample loop, fixture constructor
or graph oracle is not an independent example loop. Multiple assertions may
inspect one result; paired executions may define one relational property.

Cross-review restored scalar cache inputs and exact invalid-declaration errors,
moved a remaining evaluation before assertions, restored CLI discovery cardinality
and redaction JSON validation, and made multi-wheel provenance verification
observable. HTTP tests retain warmed-cache rejection and a script link that
would match the requested artifact if parsed incorrectly. The initial coverage
run found four missed helper lines: valid scoped URLs were added alongside
adversarial generated inputs, and string cache-damage modes became an exhaustive
enum. No exclusions were added to resolve these findings.

## Final results

| Check | Result |
| --- | --- |
| Rust stable workspace/all-features | 3,371 passed, 0 failed, 19 ignored |
| Rust nightly coverage, including ignored | 3,390 passed, 0 failed, 0 ignored |
| Rust actual LCOV mapped source/test lines | 10,758 / 10,758, 100% |
| Rust native function totals | 1,222 / 1,222, 100% |
| Python tooling/tests, including child processes | 269 passed; 2,264 / 2,264 statements, 100% |
| Rust formatting and all-target/all-feature Clippy, warnings denied | Passed |
| Python Ruff formatting/check and strict ty | Passed |
| Documentation links, OKF structure, citations and navigation | Passed |

Python final random-order seed was `1769025539`.

Rust's raw LCOV summary is **10,915 / 10,957 lines (99.62%)**. As documented in
the [existing coverage record](../coverage/review.md#metric-discrepancy-retained),
this aggregation differs from merged `DA` line records. Neither total was
rewritten. The full fresh report is the local generated artifact
`target/coverage.WkE6JB/lcov.info`. A comparison of SHA-256 fingerprints for 106
Rust/Python source and support files confirmed no changes during the final run.

Commands follow the [development guide](../../index.md#coverage), using
nightly-2026-10-08, cargo-llvm-cov 0.9.1 and the existing release benchmark binary.
The final coverage run used a fresh directory for both target and build output;
all mapped line counts were positive and the 100% native-function gate passed.
Python used the frozen environment and a temporary uv cache because the normal
cache is sandbox restricted. Local sockets were enabled for Rust HTTP/TLS tests.

These results establish execution and the reviewed test organization on this
platform. They do not establish branch coverage, exhaustive generated domains,
a new mutation score, new formal proofs, or unobserved hosted CI results.

## PR preparation after case-name revisions

Re-ran the complete stable Rust workspace/all-features suite: 3371 passed,
0 failed, 19 ignored. Python: 269 passed, 2,264/2,264 statements covered.
Rust formatting, full Clippy, Ruff formatting/check, strict ty and documentation
validation passed. The earlier fresh nightly coverage numbers above describe
the snapshot before the subsequent oracle case-name/ID-lookup correction; no
new full Rust coverage measurement is claimed for that correction.
