# Actionable diagnostics review

## Decision and sources

The owner delegated evaluation of open Issues #13, #14, #16, #17, #18 and #19.
The [plan and assessment](plan.md) records why #13 is the immediate improvement,
why #14/#18 remain useful follow-ups and why purpose/distribution-dependent
policy and fulfillment functions need a separate scope. No deferred issue is
closed by this change. Source issue text was read with gh on 2026-11-02;
implementation starts from 9a07aa7a6689ed57c4ec23c139cd7ecf75cea02a.

The earlier contract groups failures by package/artifact but the implementation
uses one UnknownLicense diagnostic for missing/repeated declarations and prints
one block per artifact. This change refines diagnostics and presentation, without
changing permission, completeness, selection, proof or acquisition decisions.

## Typed causes: five self-review passes

1. Domains: eight typed causes distinguish missing/repeated declarations,
   invalid expressions, unrecognized legacy, unavailable files, unsupported text,
   conflicting files and family mismatch. Invalid authoritative expressions never
   use valid legacy fields; repetitions are not confused with syntax errors.
2. Connection: causes are assigned where evidence interprets fields or observed
   files, not guessed from prose by renderers. The broad UnknownLicense code
   remains. Other failures, including acquisition, retain separate categories.
3. Storage/types: optional LicenseProblem uses snake_case serialization and
   defaults to None for older Error representations. Raw observation/checksum
   formats and proof requirements are unchanged; completed negative scans retain
   their behavior. No new dependency or transport fallback exists.
4. Tests: named real sidecar/archive cases cover all eight causes and raw-record
   replay. A generated cardinality property requires repeated-declaration for
   arbitrary repeated expression values rather than recreating syntax logic.
5. RED/GREEN: eleven initial cases lacked structured causes. Review corrected two
   synthetic archive fixtures whose leading blank header line accidentally hid
   License-File declarations. The corrected 11-case executable passes. The
   intermediate whole-suite failure was those same two fixtures, not ignored.

## Presentation: five self-review passes

1. Domains: both commands share a borrowed artifact view and renderer. Default
   presentation indexes groups by displayed result plus untruncated declarations
   and context; verbose displays each artifact. JSON remains per artifact.
2. Connection: CLI flags reach both production emitters, including safe invalid
   input paths. Normal check still suppresses ordinary compliant packages, while
   supplementation and applied overrides remain visible. A new RED test exposed
   hidden successful clarifications; the skip condition now retains them.
3. Identity/performance: grouping happens only within a package/source. Status,
   proof strength, origin, interpretation, severity/diagnostics and policy
   units/overrides remain discriminants; hashes and locators identify individual
   members available in verbose/JSON. BTreeMap indexing avoids pairwise scans.
   Exact full *redacted* declarations are borrowed, not cloned into each key;
   bodies/previews allocate only presentation text. No I/O is added.
4. Adversarial cases: 20 independent real-process cases/properties cover both
   commands, equal/different causes and declarations, JSON/verbose, mixed local
   acquisition failures, different hash-scoped clarifications, credentials,
   terminal controls and declarations differing beyond the preview boundary.
   Four unit cases additionally pin proof and status grouping. Previews use at
   most 200 Unicode scalars before escaping. Different original URL credentials,
   query data or fragments may intentionally merge after existing redaction.
5. Reproduction: initial CLI run had 13 intended missing-group/verbose failures;
   the two ordinary JSON cases already worked with typed causes. All new cases
   now pass. Existing process regressions continue to check permission unit
   reporting and supplementation origin separately from policy clarification.

## Documentation and completion: five self-review passes

1. Ownership: command usage and troubleshooting update their existing user
   owners. The inspection, sans-I/O and verification concepts cite this evidence;
   the assessment stays in this work record rather than duplicating user guidance.
2. Contract: schema 1 gains an optional license_problem field; existing fields
   and broad codes remain. Wording is not a machine interface. Incompatible
   future layouts require a new version. Grouping never reduces evaluation scope.
3. Safety/limits: full declarations means existing redacted report values, not
   unredacted retained metadata. Individual observations remain in JSON/verbose.
   No license permissions, configuration writes, legal compatibility inference,
   classifier guessing or extra acquisition is introduced.
4. Verification: on macOS arm64, the baseline passed 704 tests. Final stable and
   pinned-nightly randomized-order suites each pass 739 tests, with 18 intentional
   ignores. Strict Clippy, workspace fmt and warning-free rustdoc pass. Initial
   test-helper allocation lints were corrected rather than suppressed.
5. Independent review: a fresh-context reviewer completed five passes over code
   and tests, independently passing 31 focused tests with no Critical/Important
   findings. One Minor documentation precision issue (full redacted declarations)
   was corrected in the comment/plan/user guide. This review does not establish
   new formal verification of parsing, diagnostic classification or rendering.

The existing formal decision crate is unchanged. Existing Rust/Lean corpus
adapters ran with the full suite; no new Lean/Creusot guarantee is claimed.
Metadata, wheel and sdist fuzz each run 1,000 bounded executions against current
error/replay behavior. Follow the [developer guide](../../index.md) for commands.
Fuzzing is a crash/invariant smoke check rather than exhaustive validation.

Design assessment: 9/10 for this slice. Both CLI callers use one presentation
boundary; eligibility/causes remain evidence-owned. A presentation body is part
of the grouping key, intentionally tying grouping to all displayed facts. This
keeps two commands consistent but means presentation and grouping semantics
change together. Reconsider a separate structural grouping key only when another
consumer or profiling demonstrates the need; avoid speculative schema layers.

## Public offline presentation observation

The public hoimin lock snapshot SHA-256 was `7ad78d6e68c655115654c170d6b4c317d038520d5844abb2ca2100fec7171de8` on 2026-11-02.
Current `describe` in whole-inventory mode covered 84 external packages and 921
artifacts, reusing 921 cache observations with zero requests. Normal text had 34
artifact problem/result groups, 677 lines and 36,927 bytes; `--verbose` had 6,974
lines and 622,316 bytes. JSON retained all 921 artifact records. Exit 2 remained
because the snapshot has incomplete license observations. This compares current
summary and verbose modes, **not** old-main output or a cold-network benchmark.
Graph/context lines remain in both modes; the counts are observations for this
lock/cache/environment, not a fixed reduction guarantee.

OKF structure/link/citation checks and the strict user-site build passed with
the frozen Python environment and a writable temporary UV_CACHE_DIR. Python
source and dependency graphs are unchanged. The independent review predates final
document metadata validation; those checks and source hashes are parent-owned.
