# Versioned alias review and verification

Reviewed on 2026-11-03 against base `d1e01bc9cb093c217055ec86d9a974e04e36a325`
on macOS arm64. The [plan](plan.md) owns scope and the pinned public reference.
Only synthetic metadata and public upstream naming data were used.

## Stage 1: design, five self-review passes

1. Ownership: the existing private interpreter owns all 39 added aliases; no new
   resolver, configuration, enum or public interface is needed for fixed data.
2. Semantics: inspected the pinned CycloneDX entries individually; versions and
   BSD clause counts remain distinct. GNU additions explicitly grant later-version
   permission; generic Mozilla/BSD/CDDL/EPL and version-only GNU names are excluded.
3. Evidence/I/O: raw metadata is already persisted; no migration, verdict cache,
   archive escalation or external-service dependency is introduced.
4. Coverage: map acceptance criteria to named boundary cases, allow/deny public
   inspection, generated whole-value properties and actual acquisition replay.
5. Documentation: the implementation extends supported aliases within the
   existing contract; user guidance must describe the new names and limits.

## Stage 2: implementation and tests, five self-review passes

1. Scope: production changes consist only of 39 table rows and their source
   comment; strict parsing and policy/decision algorithms are unchanged.
2. Semantics: repeated fields, invalid authoritative expressions, missing versions,
   changed spacing and additional conditions remain separate negative cases.
   Existing named collision tests still reject differing IDs for the same name.
3. Evidence/I/O: real HTTP fixtures check the sidecar fast path and online/offline
   disk replay, including original fields, metadata/artifact hashes, origin and
   index-associated strength. Replay performs zero requests, bytes or extraction.
4. Coverage: AAA and independent rstest cases retain one scenario per invocation.
   Named cases check each added alias. Two new proptests generalize representative
   spellings across every added identity under case/outer-whitespace changes and
   generated prefixed/appended terms. Expectations use literal SPDX IDs.
5. Regression evidence: before production mappings, 10 of 66 public legacy tests,
   eight of 10 cache cases and the new sidecar case failed as unrecognized legacy
   names. All passed after the mappings. Existing negative tests stayed green.
   Broadening the subset added 35 initially failing name cases. The expanded
   property test also failed before the additions; 16 cache cases, two sidecar
   cases and four explicit-later policy cases separately failed before connection.
   Final targeted totals are 109 public legacy, 26 cache and seven sidecar cases,
   all passing. Existing and new ambiguous-name tests remained green.

## Stage 3: final review, five self-review passes

1. API/type review: preserve the existing LicenseExpression/LicenseOrigin types;
   fixed vocabulary additions do not require a new abstraction or enum.
2. Permission review: resolved MPL and GPL-or-later follow normal allow and deny rules; unknowns
   remain incomplete. No additional terms or AND/OR relationships are inferred.
3. Performance review: 39 additional linear comparisons after the pinned
   catalog, with no per-alias allocation or I/O. This is not a wall-time benchmark.
4. Verification review: all commands below succeeded; ignored benchmarks and
   network fixtures remain ignored rather than being counted as passing tests.
5. Documentation/diff review: updated existing user and contract owners with
   source attribution; no unrelated behavior or dependency changes. A separate
   read-only reviewer reported no actionable findings in both the initial and
   expanded subset reviews. All five passes were repeated for the expanded scope.

The scoped design scores 10/10 under software-design-philosophy's eight
diagnostics: coherent ownership, deep interface, replaceable internals,
abstraction comments, design review, hidden naming rules, documented boundaries
and deliberate design effort. No diagnostic failed for this change. This is not
a score for the entire repository or a proof of legal completeness.

## Verification

- Baseline workspace: 789 passed, 18 ignored, zero failed.
- Final `cargo test --locked --workspace --all-features`: 882 passed,
  18 ignored, zero failed. Includes existing formal-oracle correspondence tests.
- `cargo fmt --all -- --check`, strict all-target/all-feature Clippy and
  warnings-denied Rustdoc: passed.
- Legacy tests under nightly-2026-10-08, `--shuffle-seed 2603`: 109 passed.
- Existing metadata fuzz target: 1,000 runs, `-max_len=4096`, no failures;
  corpus includes new MPL/GPL alias, extra-condition and GNU version-only seeds. Mutated corpus was
  kept outside the repository. This is a bounded smoke test, not exhaustive.
- Documentation/OKF/link checker and strict MkDocs build: passed.

No new formal model is needed: the change is reviewed naming data, not a change
to decision semantics or a newly proved licensing identity. Existing formal
claims do not prove the correctness of alias-to-license assignments.
