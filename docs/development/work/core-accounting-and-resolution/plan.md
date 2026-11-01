# Core accounting and resolution implementation plan

> Execute inline using superpowers:executing-plans, task by task. Obtain a fresh
> read-only whole-change review before delivery.

**Goal:** Remove duplicated result protocol and unnecessary graph work while
preserving public behavior.
**Architecture:** Private typed observation ledger; temporary name index;
parent-index BFS. Pure core remains data-only.
**Tech stack:** Existing Rust, rstest, proptest, Lean corpus and cargo-fuzz.
**Spec:** [Assessment and design](design.md).

## Constraints and review focus

- Preserve CLI/JSON/public entry points, stable diagnostics and source identities.
- Preserve missing/duplicate/foreign/rebound result incompleteness, proof strength,
  hashless source associations, conditional extras and workspace coverage.
- No new dependency, runtime prerequisite or access to private reference projects.
- AAA, one scenario/property per test; parameterize variants and generalize them
  with independent oracles. Five distinct self-reviews per stage.
- Inspect duplicate sequences after failure, same-name source/version alternatives,
  cyclic extras, equal-length path ties and unreachable targets.

## Tasks

1. [x] Establish clean baseline and add opt-in synthetic resolution/path benchmarks;
   record before measurements and design review passes.
2. [x] Add behavioral observation-accounting tests, then implement private ledger
   using an Enum. Preserve existing AcquisitionResult path; use ledger from check
   and describe. Avoid duplicate summary construction. Review five times.
3. [x] Add independent matching/path properties and boundary regressions, then
   index candidates by normalized name and replace prefix copies with parent
   indices. Preserve exact path order and validation. Review five times.
4. [x] Re-measure, update owning OKF decision and evidence, run relevant checks,
   perform five final self-reviews and independent whole-change review. Final delivery is to commit,
   push, open PR and observe CI; leave merging to owner.
