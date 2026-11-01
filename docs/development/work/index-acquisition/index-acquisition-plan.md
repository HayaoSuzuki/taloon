# Measured index acquisition implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans inline, task by task, with TDD and five self-review passes per stage.

**Goal:** Measure acquisition costs and remove repeated per-run index fetch/parse work.

**Architecture:** Binary HTTP owns purpose-aware attempt timing; pure report
values own aggregation. A per-run index catalog hides endpoint construction,
single initialization and bounded exact-URL parsing from acquisition callers.

**Tech Stack:** Existing Rust toolchain/dependencies, std OnceLock/Mutex,
controlled Transport fixtures, proptest and existing Lean model tooling.

**Spec:** [Approved design](design.md).

## Global constraints

- Preserve locked scope, policy, proof strength, auth/redirect/retry/timeout rules.
- No index persistence, metadata-body cache redesign, new dependency or flag.
- Keep original counters; add named microsecond metrics without source details.
- Count attempts/received bytes after worker panics; no elapsed-time summing as wall time.
- Retain only complete locked-URL scope; keep ambiguity/hash failures per URL.
- Five reviews per stage, one fresh final review, commit/push/create PR; no merge.

## Review focus

- Initializer panic with blocked waiters must terminate and retain attempt stats
  (Tasks 1 panic meter and 2 concurrent initialization tests).
- An early artifact must not narrow the later snapshot's URL scope (Task 2).
- Different source/query/auth contexts must not share entries (Task 2 endpoint tests).
- Retried/redirected/partial responses must count actual attempts/bytes (Task 1).
- Warm/offline runs must not contact or parse an index; bad unrelated links must
  not invalidate good associations (Task 2, including generated permutations).

### Task 1: Account acquisition costs and establish a baseline

**Files:** `src/report.rs`, `src/acquisition/metrics.rs`, `http.rs`, `mod.rs`,
`http_tests.rs`, `src/cli.rs`, and work evidence `measurements.md`.

**Interfaces:** Pure `TransferSummary` and `HttpSummary` inside `AcquisitionSummary`;
`merge(&mut self, other: &Self)` saturates counters and excludes wall elapsed.
Binary `RequestPurpose::{Index, Metadata, Archive}` selects classification;
`Attempt::begin(summary, purpose)` records panic-safe request accounting;
`timed(&mut u64, FnOnce() -> T)` records monotonic microseconds, including unwind.

- [ ] Write failing request/byte/parse/timing, merge saturation and panic-retention
  tests. Assert original totals equal per-purpose sums after retries/redirects
  and failed partial reads; wall time is not the sum of worker time.
- [ ] Observe missing metric behavior fail before implementation.
- [ ] Implement pure value aggregation and runtime-only timers; retain per-worker
  summaries outside returned batches and recover poisoned summary locks after join.
- [ ] Add an ignored controlled benchmark with 48 distinct artifacts from one
  synthetic package, index+sidecar routes and 10 ms per transport call. Run three
  fresh-cache trials; report actual calls, bytes, parses and times. Expected before
  reuse: 48 index fetches/parses plus 48 metadata fetches, no archive acquisition.
- [ ] Run strict lint/tests/docs as relevant; review five times, record baseline
  and commit. Whole-stage test command: `cargo test --offline --locked --workspace --all-features`.

### Task 2: Share a bounded parsed index per run

**Files:** `src/acquisition/index.rs`, `mod.rs`, `http_tests.rs`, owning OKF/user
guidance, work measurements/reviews, and a small Lean lookup model if useful.

**Interfaces:** `IndexCatalog::new(requests: &[ArtifactRequest]) -> Self` collects
complete scope. `sidecar(&self, http: &Http, request: &ArtifactRequest,
stats: &mut AcquisitionSummary) -> Result<Option<(Url, SidecarAssociation)>, Error>`
hides HTTP endpoint/parsing/synchronization. Each endpoint contains a shared
once-initialized parsed result; exact artifact URL/hash lookup remains separate.

- [ ] Write failing concurrent fetch-once, complete-scope, per-URL ambiguity/hash,
  source/query isolation, per-call freshness, failure/fallback and initializer
  panic/waiter tests. Add generated URL/permutation association invariants.
- [ ] Implement per-run catalog and filtered parsed snapshots; do not hold the
  entry-map lock during transport. Retain bounded terminal failures for this run.
- [ ] Rerun the same ignored benchmark, three trials. Expected: 1 index fetch/parse,
  48 metadata fetches, 47 index hits, identical 48 outcomes/decisions, and exactly
  47 index response bodies saved. Record timing instead of an internet threshold.
- [ ] Check cached/offline zero-HTTP/parse behavior and warm 10,000-artifact
  process regression. Run relevant normal tests, strict lints/docs and model checks.
- [ ] Review five times and request one fresh whole-branch review. Fix reproduced
  important findings; commit/push/create PR and verify hosted checks. Record actual
  scope/results without publishing private inputs or their validation results.
