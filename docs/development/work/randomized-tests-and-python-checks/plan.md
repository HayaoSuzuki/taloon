# Randomized test order

The owner requested hoimin's randomized Rust testing and pytest-randomly for
Python. This is a configuration change with the execution method already chosen:
continue inline, commit and open a separate PR. Initial base was main `4c481d4`; after the owner merged describe PR #8,
work was rebased onto `ae0c299` before continuing. Existing changes were retained;
the verification-decision documentation conflict was combined, preserving both
the describe correspondence and new test-order sources.

## Scope and approach

Retain stable Rust checks and add a pinned nightly-2026-10-08 libtest --shuffle
CI job, including the existing local-hook prerequisite. Native harness output
records each executable's seed; --shuffle-seed replays that harness order, not
arbitrary OS thread scheduling. Default ignored tests stay ignored; stable TLS
and other explicit checks remain where they are.

Add pytest-randomly>=5,<6 to the docs/test dependency group and require plugin
loading in pytest configuration so a missing plugin cannot silently revert to
ordered tests. Resolve uv.lock and synchronize docs/release groups. Ordinary
pytest commands randomize and seed Python RNGs by default; session output shows
the seed and --randomly-seed supports reproduction. No production Rust logic, runtime dependency or permission/evidence rule changes.
The later Python quality extension below permits behavior-preserving tooling refactors.

hoimin source read: `39d35f3980c428ab15258b519a233b86f1551ffa`,
.github/workflows/ci.yml rust-shuffle, pyproject.toml dev dependencies, and
its development guide's randomized-test sections. Use existing taloon pinned
Actions and uv conventions; no custom shuffle implementation or test-runner swap.

## Execution and verification

- [x] Confirm pytest-randomly is unavailable before adding the dependency.
- [x] Add the separate Rust shuffle job and required Python plugin; resolve lock.
- [x] Run Python with default randomness, a fixed seed and a replay; run nightly
  Rust with generated seeds and a fixed seed. Confirm actual output seeds.
- [x] Update the developer guide and verification decision with reproduction
  commands and limits. Review scope, prerequisites, seeds, CI isolation and
  documentation five times. Run workflow/docs checks and inspect lock changes.
- [x] Request a fresh read-only review and fix findings.

Final delivery: commit, push, open a separate PR and observe CI. Do not merge
or publish. GitHub evidence is attached to the PR rather than presumed here.

No implementation-mirroring test or new formal model is needed: execute the
actual test runners to establish shuffle/plugin behavior. The Python quality extension adds focused mutation verification for changed
production tooling.

## Scope extension from the owner

During implementation the owner additionally requested Ruff formatting/lint and
ty type checking with constraints as strict as practical. Extend this change to
all maintained tools/ and tests/ Python code, retain runtime behavior and empty
runtime dependency metadata, enable Ruff ALL plus formatter-compatible rules and
narrow justified test exceptions, and ty all-error/strict analysis. Add CI and
local hook gates, repair existing diagnostics, and verify behavior under randomized
pytest. Production Python refactoring requires focused mutation-test assessment.

## Test design requirement from the owner

Use Arrange/Act/Assert consistently. One function describes one behavior/case;
input variants must be independent parametrized cases rather than loops inside a
test. Generalize the relevant parametrized domain to property-based tests with
independent expectations, retaining readable fixed regressions. Apply this to
new/rewritten Rust and Python tests; record it in the developer guide and agent entrypoint.
