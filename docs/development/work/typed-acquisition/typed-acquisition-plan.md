# Typed acquisition implementation plan

> **For agentic workers:** Use superpowers:executing-plans inline; one fresh whole-PR review.

**Goal:** Typed grouping keys and direct typed JSON index decoding.
**Architecture:** Binary-private WorkKey and serde protocol records/advertisement
Enum; preserve the core request/result and proof boundary.
**Stack:** Existing Rust/std/serde/rstest/proptest, no dependency change.
**Spec:** [design](design.md).

## Constraints and review focus

Backward compatibility is not required. Preserve inventory, hash/identity/source
validation and archive/index proof distinction. Synthetic inputs only. Review
empty scope, cross-format/hashless grouping, metadata presence versus availability,
malformed/duplicate known fields and unknown extensions. Tests below own each.
Five self-reviews per stage; user docs explain stricter malformed-index fallback.

## Task 1: typed worker grouping

- [x] Add partition_tests.rs ignored 1k/10k measurements; record baseline samples.
- [x] Add named worker-bound/empty/group cases and 256-case integer-label grouping
  oracle. Expected RED: empty scope currently creates one partition, expected zero.
- [x] Replace String map keys with private WorkKey; derive ArtifactFormat ordering;
  return no partition for empty scope. Expected GREEN including progress tests.
- [x] Measure identical release harness, review implementation five times.

## Task 2: typed JSON protocol

- [x] Add index/json_tests.rs ignored 1k/10k history benchmarks and record baseline.
- [x] Add malformed/duplicate known-field cases (runtime RED), valid availability/
  hash/legacy/extension cases and 256-case independent advertisement selection.
- [x] Add private typed records and serde untagged advertisement Enum. Explicit
  present-field decoding must reject null; modern false must not select legacy.
  Existing exact URL/hash checks and per-target ambiguity remain unchanged.
- [x] Measure identical fixture; review implementation five times. Run full stable,
  generated/seed12345 tests, Clippy/rustdoc, existing Lean/core fuzz/Python/docs.
- [x] Update contracts/user guidance with actual hashes, five final reviews and
  fresh whole-PR assessment. Commit, push to still-open PR10, observe actual CI.


## Completion audit: content-only clarification API

The goal continuation revalidated clean `7416ba5`, open PR #10 and all 13
executed checks succeeding. Task 2's last checkbox is now reconciled with those
observed results. The audit found one unnecessary public coupling: clarification
lookup accepted an unused ArtifactRequest despite being content-hash keyed.

- [x] Add two named AAA cases calling clarification with only a ContentHash;
  observe E0061 against the previous signature, then remove the unused argument
  and update its sole inspection caller. No compatibility overload remains.
- [x] Verify the complete stable/shuffled suites, Clippy/rustdoc, final design
  audit and documentation/source attribution. Existing proof/evidence semantics
  and unrelated tool/runtime prerequisites remain unchanged.
External completion gate: commit/push this final API cleanup and inspect CI for
that resulting SHA. The live PR and active goal status own that post-commit gate;
this source document records evidence available before committing its own bytes.
