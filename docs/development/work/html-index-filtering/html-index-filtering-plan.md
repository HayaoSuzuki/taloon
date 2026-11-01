# HTML index filtering implementation plan

> **For agentic workers:** Use superpowers:executing-plans inline, with one fresh whole-PR reviewer at the end.

**Goal:** Stop retaining all historical HTML tags while preserving associations.
**Architecture:** Interpret relevant tokens in the private HTML TokenSink; retain
Result<state, Error> and Option<explicit base>. Keep PackageIndex API unchanged.
**Tech stack:** Existing Rust, html5ever, rstest, proptest; no dependencies added.
**Spec:** [design](design.md).

## Constraints and review focus

Preserve sans-io, exact URL/hash/proof checks and existing CLI/cache formats.
Use synthetic data only. Five self-reviews per stage. Review raw-text elements,
base encounter order, late errors, duplicate links and metadata attribute
precedence. Every focus has a fixed case below or an existing corpus regression.

## One task: stream private HTML association interpretation

- [x] Add named AAA cases for base-before/after links, first/missing base href,
  late unsupported meta, raw-text fake links, modern/legacy precedence and
  duplicates. Add a 256-case independent base-order/irrelevant-link property.
  Run the old parser first: expected PASS as preservation characterization,
  not a claim of a previously broken behavior. Existing suite is already green;
  this work is the refactoring phase, not a new behavioral feature.
- [x] Add an ignored release benchmark for 1,000/10,000 synthetic historical
  anchors and one locked target. Record five baseline samples per case; no CI
  timing threshold or inferred RSS result.
- [x] Replace retained tag vector with private HtmlIndex TokenSink state in
  src/acquisition/index.rs. Borrow tokenizer-owned attribute values with the old map's last-local-name
  semantics and retain existing raw-text decisions;
  fold base/meta/anchor handling into the sink and leave html_links as the
  tokenization entry point. Expected: all characterization tests and strict
  index model correspondence remain green.
- [x] Record identical after samples. Run full stable and shuffled Rust suites,
  Clippy/rustdoc, existing Lean freshness and metadata/inspection fuzz, Python/
  docs/hooks checks. Write five actual review passes per stage and update owning
  OKF concepts. New proof claims and user-flow changes are out of scope.
- [ ] Obtain fresh whole-PR review, resolve actionable findings, inspect final
  staged diff/hashes, commit, push to still-open PR #10 and observe final CI.
