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

- [ ] Add fixed slot scenarios and a sequence property before implementation.
- [ ] Implement Missing/Single/Duplicate with exact payload/consumption contracts.
- [ ] Delegate production ResultLedger slot transitions; keep BTreeMap ownership.
- [ ] Establish pinned proof environment and fresh obligation checks; prove slots.
- [ ] Exercise compiling duplicate/consumption mutations and restore clean source.
- [ ] Run Rust regression/oracle checks and five self-reviews; update evidence and commit.

## Stage 2: Verdict and exit precedence (second commit)

- [ ] Add parameterized verdict/exit cases and an independent precedence property.
- [ ] Move Status and decision bodies to the same verified crate; delegate reports.
- [ ] Prove exact precedence and bounded exit codes; reject compiling mutations.
- [ ] Run all relevant checks, five self-reviews and independent final review.
- [ ] Update current verification contract, developer procedure and evidence; commit.
- [ ] Push and open a reviewable PR after final checks.
