# Test design audit implementation plan

**Goal:** Audit every maintained Rust and Python test for one scenario or property per function, explicit case parameterization, meaningful generated coverage, and Arrange/Act/Assert separation.

**Architecture:** Preserve production behavior and existing coverage. Review integration tests, inline Rust tests, workspace crate tests and shared fixtures. Distinguish independent examples from fixture construction, independent oracles and benchmark sampling; the latter may legitimately use loops.

**Tools:** rstest, proptest, pytest parametrization, Hypothesis, existing coverage and quality commands in the development guide.

## Tasks

- [x] Inventory all test functions in `tests/`, `src/` and `crates/taloon-decisions/`. Record per-file audit outcomes and specific exceptions in a review record beside this plan.
- [x] Convert independent example loops in Rust integration tests, starting with `tests/evidence.rs`, `tests/inspection.rs`, and `tests/description.rs`, to named rstest cases. Separate different operations or contexts into separate functions; retain fixture and independent-oracle loops.
- [x] Review Rust inline tests, including acquisition/cache, HTTP, evidence, policy and reporting, and decision-crate tests using the same criteria. Keep multi-step state transitions together only when they define the single property under test.
- [x] Review all Python test modules. Separate setup, the operation, and observable assertions. Exception matchers may combine Act and Assert. Replace independently executed example variants with named pytest parameters. Do not treat comprehensions or fixture-building loops as violations without examining their role.
- [x] Map generated tests to meaningful variable domains and independent expectations. Extend missing domains with proptest/Hypothesis, retaining readable boundary regressions. Do not generate arbitrary inputs for fixed environment or process checks solely to increase property-test counts.
- [x] Run affected tests after each change, then full Rust/Python suites and quality gates. Re-measure mapped Rust lines/functions and Python lines with existing exclusions unchanged.
- [x] Review the final diff against all four requirements. Record remaining platform limits and justified patterns. User behavior and design contracts remain unchanged, so no user-guide or OKF contract change is required unless the audit discovers a contract discrepancy.

## Review focus

Preserve case isolation when parameterizing archive/metadata failures. Preserve request counts and state assertions when splitting acquisition tests. Preserve exception checks when separating AAA. Keep independent graph and policy oracles independent of production code. Do not confuse a sequence defining a state-machine property with unrelated scenarios.

## Acceptance

Every inventoried file has a recorded review outcome; independent variants run as separate cases; each function tests a named scenario or property; generated domains have meaningful oracles; AAA is structurally visible. Full quality and coverage gates pass. Coverage alone does not satisfy this audit.

Final evidence: [verification record](verification.md).
