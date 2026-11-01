# Selective cache clear implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans inline with TDD and five self-review passes.

**Goal:** Add taloon cache clear without deleting unrelated data.

**Architecture:** The existing binary cache module owns record recognition and
selective deletion. CLI reuses directory resolution and renders the count/error.

**Tech Stack:** Rust std filesystem, existing Clap and proptest; no dependencies.

**Spec:** [Confirmed scope and design](design.md).

## Global constraints

Only taloon cache; preserve root/unrelated files/directories/symlinks; no recursive
remove, network, project discovery or library I/O. Missing cache succeeds. Use
the current location precedence. Commit and create a PR; do not merge/publish.

## Review focus

- Arbitrary configured directory and damaged records: preserve unrelated data.
- Matching-name directories and symlinks: never recurse or follow entries.
- Missing/root-file/root-symlink locations: deterministic success/error.
- Partial failure and concurrent publication: no atomic-clear claim.
- Existing cache filenames and non-UTF-8 names: recognize only complete grammar.

### Task 1: Selectively clear cached evidence

**Files:** Modify src/acquisition/cache.rs, src/acquisition/mod.rs, src/cli.rs,
tests/cli.rs, docs/user/usage.md and owning license-inspection contract; add
cache unit properties and work review.

**Interfaces:** Binary-visible EvidenceCache::new(PathBuf) and
EvidenceCache::clear(&self) -> Result<u64, Error>. CLI Command::Cache contains
CacheCommand::Clear. Existing cache_directory resolves the location unchanged.

- [ ] Add process tests: produce real archive and index records, add damaged
  matching records and unrelated files/directories, clear from outside a project;
  assert count, preservation, zero network and subsequent cache miss.
- [ ] Add missing/idempotent/root-file tests and Unix root/entry symlink tests.
  Observe the new command fail before implementation (expected exit 2, wanted 0).
- [ ] Implement complete lowercase 64-hex filename recognition and immediate
  regular-file deletion in cache.rs. Missing entries do not count; other errors
  fail safely. Wire nested CLI command without check initialization.
- [ ] Add bounded generated filename/preservation coverage; verify old archive
  and index producers yield recognized paths. Keep corrupt records removable.
- [ ] Update user guidance and the owning OKF contract. Perform five actual
  self-reviews: scope, identity, symlinks, errors/concurrency and architecture.
- [ ] Run cargo test --offline --locked --workspace --all-features, fmt, strict
  Clippy/rustdoc and docs validation; commit. Request one fresh whole-branch
  review and fix reproduced important findings with RED→GREEN tests.

After implementation, push/create the PR and observe hosted CI; record results
in the PR and preserve all implementation rulings in the review and final handoff.
