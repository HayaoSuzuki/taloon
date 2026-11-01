# Production decision contracts implementation plan

**Goal:** Verify actual Rust observation-slot and verdict decisions with Creusot.
**Architecture:** A small production workspace crate with cfg-gated contracts,
called by the existing ledger and reports; existing Lean/testing stays in place.
**Spec:** [Design](design.md).

## Constraints and review focus

Preserve exact payload identity, duplicate absorption before consumption,
consumption followed by missing, foreign-ID diagnostics, error/violation/warning
precedence, JSON spelling and ordinary stable builds without proof tooling.
Use pinned solver/compiler inputs, bounded fresh proofs and compiling negative
witnesses. Review allocation changes and dependency/license scope explicitly.

## Stage 1: Result accounting (first commit)

- [x] Add fixed slot scenarios and a sequence property before implementation.
- [x] Implement Missing/Single/Duplicate with exact payload/consumption contracts.
- [x] Delegate production ResultLedger slot transitions; keep BTreeMap ownership.
- [x] Establish pinned proof environment and fresh obligation checks; prove slots.
- [x] Exercise compiling duplicate/consumption mutations and restore clean source.
- [x] Run Rust regression/oracle checks and five self-reviews; update evidence and commit.

## Stage 2: Verdict and exit precedence (second commit)

- [x] Add parameterized verdict/exit cases and an independent precedence property.
- [x] Move Status and decision bodies to the same verified crate; delegate reports.
- [x] Prove exact precedence and bounded exit codes; reject compiling mutations.
- [x] Run all relevant checks, five self-reviews and independent final review.
- [x] Update current verification contract, developer procedure and evidence; commit.
- [x] Push and open [PR #12](https://github.com/HayaoSuzuki/taloon/pull/12) after final checks.
