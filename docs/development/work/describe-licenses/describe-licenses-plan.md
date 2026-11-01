# Locked license description implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans inline, with TDD and five self-review passes per task.

**Goal:** Describe selected locked licenses without policy evaluation.

**Architecture:** Resolution owns private typed selection/graph traversal. Public
Description owns selected evidence requests and policy-free complete/incomplete
reports. The CLI shares existing acquisition, credentials, cache and progress.

**Tech Stack:** Existing Rust/Clap/serde/proptest, Lean, existing fuzz workspace.

**Spec:** [Approved design](design.md), approved by the owner to implement in this session.

## Global constraints

uv.lock only; no policy/pyproject prerequisite; all versions/sources; external
normal+all-extra+all-environment closure without dev; workspace-name selects
all external inventory; exclude workspace license observations. All artifacts,
exact identity/proof, unknown/different declarations preserved; no new dependency,
cache schema or check policy mode. Five reviews per task, fresh final reviewer,
commit/push/PR and observed CI. Synthetic fixtures only; no private examples.

## Review focus

- Workspace-name/external-name collisions: AllExternal wins and keeps all roots.
- Workspace intermediary/cycles/diamonds: no lost descendants or exponential tree.
- Different artifact declarations and invalid/missing/foreign evidence: never
  conflate, promote proof or present partial observations as complete.
- Malformed/unreadable/missing pyproject and startup JSON errors: describe must
  stay policy-free; check/cache behavior must remain unchanged.
- Unrelated artifacts and secret-bearing sources/markers: select before I/O,
  preserve conditional context and redact/escape output.

### Task 1: Pure selection, descriptions and correspondence

**Files:** src/resolution/mod.rs (selection in a private submodule), new
src/description/mod.rs, src/lib.rs, minimal crate-private AcquisitionResult access
in src/inspection/mod.rs, tests/description.rs, Lean scope model/generator and
corpus/adapter; CI formal freshness command and formal verification docs.

**Interfaces:** Description::from_uv_lock(lock: &str, name: &str) -> Result<Self, Error>;
requests(&self) -> &[ArtifactRequest]; evaluate(Vec<AcquisitionResult>) -> DescriptionReport.
DescriptionReport has typed complete/incomplete status, getters, from_input_error,
with_warnings, with_acquisition_summary and exit_code() -> u8. Graph selection
is crate-private; public serializable enums describe scope and dependency kind.

- [ ] Write and observe failing tests for normalized/all-identity selection,
  all-external equivalence/collision, all extras/markers/no-dev, workspace routing,
  unsupported targets, graph cycles and exact request subset.
- [ ] Implement selected graph with roots retained, deterministic context and
  request slices; add policy-free per-artifact identity/result accounting,
  exact-text grouping and explicit unknowns. Keep check evaluation unchanged.
- [ ] Test differing licenses, malformed/unknown/custom declarations, missing/
  duplicate/foreign/substituted observations and order invariance. Add bounded
  generated graph cases against an independent reachability algorithm.
- [ ] Add small Lean selection model/proofs plus generated finite graph cases,
  sensitivity witnesses and strict public Description adapter. State canonical
  identity/graph premises; do not claim general Rust/HTTP correctness.
- [ ] Run full Rust suite, strict formatting/Clippy/rustdoc and formal freshness;
  review scope/graph, evidence, errors, ownership and oracle correspondence five
  times; record results and commit.

### Task 2: CLI, shared acquisition and user guidance

**Files:** src/cli.rs with a focused describe renderer/runtime submodule if needed,
tests/cli.rs, src/acquisition/http_tests.rs, fuzz/fuzz_targets/inspection.rs,
docs/user/usage.md and owning OKF contracts/decisions; work review record.

**Interfaces:** taloon describe <package> [--project DIR] [--format text|json]
[--offline]. Reuse Description from Task 1, existing Acquirer and AcquisitionSummary.
No --deny-warnings or policy/clarification evaluation. Exit0 complete,2 incomplete.

- [ ] Observe new process tests fail before command wiring. Cover lock-only input,
  ignored broken policy, all-external/project/empty scope, text/JSON/input errors,
  shared cache/offline, all identity variants and unchanged check/cache behavior.
- [ ] Extract common evidence fulfillment/progress from check into runtime-only
  helper; select requests before fetching; wire describe and separate typed report.
  Render unique package lists, license groups/unknown artifacts, annotated edges
  and workspace/origin context; safe sources and terminal control escaping.
- [ ] Verify synthetic transport receives no unrelated requests. Extend existing
  structured inspection fuzz target with description parsing/selection/results and
  bounded smoke; preserve all existing campaigns and limits.
- [ ] Update implemented user guidance and existing owning OKF topics/formal
  reproduction links. Review scope, output/security, sharing, errors and docs five
  times. Run full Rust tests, strict lints/docs, Python tooling and workflow checks.
- [ ] Commit implementation and request one fresh whole-branch review. Reproduce
  important findings RED→GREEN, fix in one pass and run the full suite.

Push/create a PR via gh, observe hosted checks and record results before handoff.
