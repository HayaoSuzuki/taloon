# Constrained legacy supplementation review

## Scope and prior boundary

The owner requested reasonable supplementation of incomplete declarations. The
prior contract admitted bundled files only when both declaration fields were
absent. This change deliberately extends that boundary to a fixed whole-value
set; it does not relax the complete-text matcher or SPDX expression syntax.
The [plan](plan.md) records the selected design. Maintained fixtures use synthetic
artifacts and public SPDX text; this record makes no claim of legal equivalence.

## Core: five self-review passes

1. Acceptance/rejection: UNKNOWN accepts an existing recognized template; BSD
   requires BSD-2-Clause/BSD-3-Clause, Apache requires Apache-2.0. Case/outer
   whitespace are ignored only for the whole eligible value. Explicit expressions,
   repeated fields, valid legacy interpretations and additional terms retain
   priority. NOASSERTION retains its existing strict interpretation without
   acquiring files; it is not a supplementation placeholder.
2. Production connection: `Evidence::file_constraint` owns eligibility for both
   acquisition and interpretation. Fresh archive construction re-evaluates metadata;
   a stale sidecar cannot override a different archive declaration. Complete files
   must agree before the family constraint is checked.
3. Types/storage/performance: a private FileConstraint hides family knowledge from
   I/O; public LicenseOrigin describes evidence before policy clarification.
   Immutable raw records, schema/checksums, proof binding and bounded file readers
   are unchanged. Origins are recomputed, never persisted as verdicts. No new
   dependencies, templates, transport layer or configuration flags.
4. Adversarial tests: wheel and sdist tables cover exact labels and mismatches;
   conflicting/unrecognized second files block inference. Record replay retains
   origin. A generated added-term property uses `BSD; <suffix>` so its domain
   cannot accidentally generate the legitimately eligible `BSD License` label.
5. Evidence: initial public tests compiled and failed on missing supplementation
   and missing origins. The complete bundled-license executable now passes 57
   cases. Cache tests below independently exercise the production query.

## Acquisition: five self-review passes

1. Cold behavior: eight eligible-label cases require exactly one archive fetch and
   extraction after the existing index/sidecar path.
2. Provenance: two cases advertise eligible sidecars but place an authoritative
   expression or additional restrictions in the archive. Only fresh archive
   metadata decides the result, at archive proof strength.
3. Reuse: positive and negative completed scans require zero network requests,
   body bytes and extractions on the next run. Cached sidecars upgrade directly
   without another metadata/index fetch; pending offline observations never fetch.
4. Rejection: invalid/repeated expressions, repeated legacy fields and added
   restrictions do not escalate. Family mismatches and unsupported text remain
   Unknown and are cached as completed observations.
5. Reproduction: 27 independently named HTTP cases use in-memory responses and
   isolated temporary caches. The first eight cases failed on archive count 0
   instead of 1 before implementation; all 27 pass after implementation.

## Reporting: five self-review passes

1. Both check and describe show successful supplementation in JSON and text,
   including compliant artifacts with no diagnostics.
2. Rendering reads typed evidence origin, not the final policy expression. A
   clarification may change that expression without changing observed origin.
3. Labels use existing escaped output paths and perform no I/O. No warning is
   invented just to make a successful supplement visible.
4. Four negative declaration domains run independently for both commands: valid
   expression, already resolved legacy, absent fields and invalid expression.
   None claims legacy supplementation. Clarification regressions retain raw BSD
   declarations while policy evaluates MIT.
5. Six initial real-process cases failed before implementation; the CLI executable
   now passes all 38 cases, including 14 new reporting cases. Synthetic local
   wheels contain public canonical BSD-3-Clause text.

## Independent review

A fresh-context read-only reviewer completed five passes over the working code
and tests: eligibility/priority, call connection/provenance, storage/bounds,
independent cases and requirements correspondence. No Critical, Important or
Minor findings remained. The reviewer independently ran all 57 bundled-license
cases, 14 reporting cases and 27 acquisition cases. This approval covers code
and tests, not final documentation metadata validation.

## Verification and limits

On macOS arm64, the complete locked all-features workspace suite passed 704 tests
with 18 intentional ignores; the pinned nightly shuffled suite independently
passed the same 704 tests. Strict Clippy, workspace formatting and warning-free
rustdoc passed. Existing Rust/Lean corpus adapters ran in the workspace suite;
this change adds no new formal proof and makes no new Creusot/Lean claim about
metadata parsing, template equivalence or legal completeness.

Metadata, wheel and source-archive fuzzing include named UNKNOWN supplementation
seeds. Roundtrip invariants additionally compare eligibility and origin. Each
bounded smoke run uses 1,000 executions and maximum generated length 4,096 bytes;
this is a crash/invariant check, not exhaustive coverage. Follow the
[developer entry point](../../index.md) for reproduction commands.

Documentation review keeps usage in the user evidence guide and updates the
inspection, sans-I/O and verification owners. Sources are attributed to the
bytes actually reviewed; historical reports remain historical. No README, Python,
workflow, dependency or distribution behavior changes are needed for this slice.

Design assessment: 10/10 against the skill's eight diagnostics for this extension.
Eligibility and family decisions are hidden behind existing evidence queries;
callers gain one small provenance enum rather than header/matcher knowledge.
Revisit if new families require broader semantics, matcher performance warrants
measured caching, or reporting needs separate evidence and policy origin types.

## Documentation and validation: five self-review passes

1. Ownership: usage stays in the existing user evidence guide; contracts and
   rationale stay in the three existing knowledge concepts. The work plan/review
   records the deliberate change from absent-only eligibility.
2. Correspondence: exact accepted values, strict expression priority, family
   mismatches, fresh metadata and pre-clarification origin agree with production
   paths and independent cases. No automatic policy permission is implied.
3. Source scope: the review is attributed by its actual SHA-256 in each concept.
   No new formal verification or exhaustive fuzzy/text recognition is claimed.
4. Checks: OKF/link/navigation validation and strict MkDocs build passed using
   the frozen environment with a writable temporary UV_CACHE_DIR. The default
   uv cache was unavailable inside the sandbox on the first attempt.
5. Formatting/limits: root workspace fmt and changed fuzz support rustfmt pass.
   An additional whole-fuzz-workspace fmt check reports pre-existing formatting
   in unchanged fuzz_targets/inspection.rs and uv_lock.rs; those unrelated files
   were not rewritten. All three targeted fuzz runs completed 1,000 executions
   without a crash/invariant failure. Generated corpus files were removed while
   retaining the three named supplementation seeds.
