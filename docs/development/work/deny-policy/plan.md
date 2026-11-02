# Deny policy implementation plan

> **For agentic workers:** Use superpowers:executing-plans for task-by-task delivery.

**Goal:** Implement Issue #23's schema-1 allow/deny choice without weakening evidence or custom-reference review.

**Architecture:** Policy owns an explicit Allow/Deny mode and one validated atomic-unit set. SPDX Boolean evaluation remains shared; deny flips standard atomic membership before expression evaluation, while explicit denials win over scoped exceptions. Hash-scoped custom permissions and proof checks retain their existing owner.

**Tech Stack:** Rust, existing spdx/TOML/serde; rstest and proptest. No runtime dependencies added.

**Spec:** [License inspection contract](../../knowledge/license-inspection.md), revised by Issue #23 and the owner instructions recorded in this plan.

## Constraints and alternatives

Exactly one of `allow` and `deny` is mandatory, including empty arrays. Allow retains existing semantics. Deny permits recognized standard units absent from its set, denies exact listed atomic/WITH units, and never implicitly permits custom references. Exceptions remain available for hash-scoped custom permissions; deny wins over matching exceptions. Clarification changes a declaration and grants no permission. Missing/invalid/unknown declarations and inadequate proof remain incomplete. OR requires any permitted term and AND requires every term.

Rejected alternatives: two sets permit contradictory configuration; a Boolean mode obscures intent; complementing the final expression reverses AND/OR meaning; banning exceptions prevents reviewed custom references; allowing exceptions to override deny weakens a workspace prohibition. Existing documentation requires allow and describes only additive permissions; record this as the deliberate Issue #23 contract change before implementation.

## Task 1: Policy semantics

Files: `src/policy/mod.rs`, `tests/policy.rs`, `tests/policy_index.rs`.

- [x] Add named configuration, Boolean, exact-unit, custom-reference, exception-precedence and proof regressions; add independent numeric Boolean properties.
- [x] Run focused tests and record expected RED from deny being unsupported.
- [x] Implement mode/set representation, validation, shared evaluation and distinct unused-deny warnings.
- [x] Run focused tests then full Rust suite; review five times; commit policy target.

## Task 2: Report and documentation

Files: `src/report.rs`, `src/inspection/mod.rs`, `src/cli.rs`, process tests, existing user configuration/usage and OKF owners.

- [x] Add and run RED report/process tests for selected mode on complete/incomplete and empty-target runs, plus policy-free describe coverage.
- [x] Expose optional report mode (absent when input policy never parsed), mode on decisions and text, without serializing configuration.
- [x] Update current user guidance and contract/decision owners with actual source snapshot hashes.
- [x] Run Rust format/Clippy/tests/rustdoc, nightly shuffled tests, relevant bounded fuzz, docs checks; review five times; commit reporting/docs target.

## Review focus

Custom SPDX classification must come from parsed variants; custom WITH units need exact identity. Do not short-circuit observed OR branches for warnings. Do not change evidence, acquisition, discovery, source redaction, describe or exit precedence. JSON startup failures may lack mode. All checks use this isolated worktree and synthetic public fixtures.

## Design reviews before implementation

1. Requirement pass: allow XOR deny preserves empty-vs-absent distinction; schema stays 1.
2. Security pass: deny wins over exceptions; custom references still need archive-verified, hash-scoped explicit permission.
3. Algebra pass: membership is inverted per standard atom; AST operators remain unchanged.
4. Boundary pass: policy remains Sans I/O and reporting never exposes the configured set.
5. Complexity pass: one enum and one set centralize selection; no parallel evaluators or new dependencies. Design score 10/10; all eight software-design diagnostics pass at design stage. Implementation review must reassess.

## Risks and verification limits

Default-permit applies only to recognized standard SPDX requirements. Existing allow-based Lean corpus covers allow only; deny receives independent generated Boolean expectations. Local macOS verification cannot establish GitHub CI results. No private dependency sample is accessed.

## Task 1 implementation evidence and five reviews

The deny-specific RED command (`cargo test --locked --test policy deny --quiet`) failed 19 cases at unsupported policy parsing (4 invalid-configuration cases already passed). The implementation then passed all 55 policy cases. Eight public-inspection cases preserve invalid/missing/unknown outcomes, custom-reference nonpermission, clarification denial, and complete-run/all-OR warning behavior. The full workspace/all-features suite passed before reporting changes (local macOS arm64).

1. Schema review: Option arrays distinguish absent and explicit empty; both arrays always reject. Full validation still covers unmatched overrides.
2. Permission review: hard global denial precedes additive scopes; custom-reference permission remains hash scoped. SPDX LicenseItem classification prevents spelling heuristics.
3. Algebra review: binary independent Boolean oracle exercises all three unit masks with both operators, and readable nested/WITH/GNU cases distinguish atom inversion from AST inversion.
4. Boundary review: no acquisition, evidence parsing, minimum-proof or describe code changes. Synthetic archive integration retains incomplete precedence and suppresses unused warnings on incomplete runs.
5. Complexity review: one global set, one explicit mode, shared AST evaluator and warning scan; deterministic unit reporting retains ordering. Score 10/10, all eight diagnostic rows satisfied. No review findings remain at this stage.


## Task 2 reviews and resolved findings

The report RED command (`cargo test --locked --test cli cli_json_identifies_selected_policy_mode`) failed all four cases on missing `policy_mode`. All four passed after adding optional report mode, decision mode and text mode. Mode remains absent when startup fails before valid policy loading. A policy-free describe regression passes against a denying policy.

1. Report review: mode survives complete, violation and incomplete inspection; the text summary also covers empty inventories. No configuration arrays enter JSON.
2. Regression review: all existing allow cases retain their behavior. Named policy examples replace the old documentation test's hardcoded two-example count after the deny example exposed that test limitation (RED count 3 versus 2, then all three named examples GREEN).
3. Evidence review: sidecar-only evidence cannot satisfy a hash-scoped custom permission; clarification to a prohibited unit yields violation. Invalid/missing/unknown expressions remain incomplete, and custom units without explicit permission remain violations.
4. Documentation review: configuration, report usage, evidence guidance and the existing contract/design owners describe deny precedence, custom reference limits and exact WITH/GNU units. Source snapshots identify actual bytes read. The owner contract records the allow-only text as superseded by Issue #23.
5. Complexity review: the policy module owns mode semantics; report/CLI receive only the selected enum. No acquisition or describe implementation changes and no runtime dependencies. Design score 10/10: all eight software-design diagnostics pass. Remaining verification limits are listed below.

## Verification scope

All new fixtures contain synthetic metadata and archives. The existing Lean policy corpus exercises allow mode; independent binary and nested numeric Boolean proptests cover deny rather than claiming new Lean correspondence. This change leaves the production decision crate and Creusot contracts untouched. Formal CI is separately subject to the external Creusot toolchain download failure tracked by PR #22; it is not a deny runtime defect. No merge or deployment belongs to this change.

## Final local checks (2026-11-02, macOS arm64)

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`: passed.
- `cargo test --locked --workspace --all-features`: 789 passed, 18 ignored, 0 failed across 37 harnesses. The baseline had 739 passing tests.
- `RUSTDOCFLAGS='-D warnings' cargo doc --locked --workspace --all-features --no-deps`: passed.
- `cargo +nightly-2026-10-08 test --locked --workspace --all-features -- -Z unstable-options --shuffle`: 789 passed, 18 ignored, 0 failed. Each executable reported its own seed; shuffle does not prove arbitrary concurrent interleavings.
- `cargo +nightly-2026-10-08 fuzz run policy /private/tmp/taloon-deny-seeded-corpus -- -runs=1000 -max_len=4096`: passed, including the new deny seed and existing allow/expression seeds. The temporary corpus prevents generated mutations from entering maintained fixtures. The earlier smoke run's 158 generated hash-named entries were removed, retaining named seeds.
- Documentation checker and strict MkDocs build: passed using the repository's pinned Python environment.
- `python -m pytest tests/test_check_docs.py`: 25 passed. No Python tooling changed.

The parent agent performed a fresh independent whole-diff review: no Critical or Important findings; its 115 focused checks passed. Its metadata-snapshot and generated-corpus cleanup findings were addressed. This local review does not establish an external GitHub CI result.
