# Terminal text implementation plan

> **For agentic workers:** Use superpowers:executing-plans inline; fresh whole-PR review at the end.

**Goal:** Borrow unchanged report text and avoid per-character allocation.
**Architecture:** Private Cow<str> renderer helper with one escape predicate.
**Stack:** Existing Rust/std, rstest/proptest. No new dependency.
**Spec:** [design](design.md).

## Constraints and review focus

Exact existing output; no public/CLI/JSON/proof/cache contract changes. Synthetic
inputs only. Five self-reviews per stage. Check ASCII controls, C1, bidi range
boundaries, ordinary Unicode/quotes, and borrowing versus escaping; each has
fixed/generated coverage below. All Rust/Python test conventions remain in force.

## One task

- [x] Add src/cli/text_tests.rs with named output cases, numeric Unicode oracle
  and ignored three-class release benchmark. Wire test-only child module in
  src/cli.rs. Run old helper first: semantic cases PASS; measure five samples.
- [x] Add borrowing cases requiring Cow::Borrowed for unchanged text. Observe
  old String type mismatch (new private ownership contract, not runtime defect).
- [x] Change safe() in src/cli.rs to Cow<str>, extracting one private control
  predicate. Keep caller code unchanged; expected all old/new cases PASS.
- [x] Compare identical release samples, run full stable/generated/seed12345
  Rust, Clippy/rustdoc, existing Lean/fuzz, Python/docs/hooks checks. Record five
  implementation/final reviews and update owning OKF with actual source hashes.
- [ ] Fresh whole-PR review, final diff/privacy/hashes; commit, push to still-open
  PR #10 and observe CI. Historical timings retain their revision/environment.
