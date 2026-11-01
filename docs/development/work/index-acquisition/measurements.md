# Controlled initial acquisition measurements

## Method

Recorded on 2026-11-01 on macOS arm64 with Rust 1.99.1, debug profile. The
opt-in `cold_index_acquisition_benchmark` exercises the actual acquisition/HTTP
path through a synthetic Transport, not the internet or TLS implementation.
It serves one package with 48 distinct locked archive digests and matching
metadata sidecars. Every call delays 10 ms and uses fixed response bytes; eight
acquisition workers and a new evidence cache are used per trial. The index body
is 10,878 bytes; metadata bodies total 4,838 bytes. All 48 artifact results are
accounted for and yield the same compliant result. No archive is fetched.

Request counts, response bytes, parses and metrics come from actual execution,
not a multiplication substituted for observations. Whole-run elapsed time and
overlapping request-duration sums are reported separately. Filesystem/cache
publication is included in elapsed acquisition time. This is not a production
network performance guarantee or a new initial-run latency target.

Run with:

```sh
cargo test --offline --locked --bin taloon cold_index_acquisition_benchmark -- --ignored --nocapture
```

## Before index reuse

Measured in the task-1 working tree based on `95ebd45`, with accounting added
but the original per-artifact index fetch/parse behavior preserved.

| Trial | Wall us | Index requests/parses | Metadata requests | Total bytes | Index bytes | Sum of index/metadata request us |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 275,212 | 48/48 | 48 | 526,982 | 522,144 | 590,278 / 578,433 |
| 2 | 232,973 | 48/48 | 48 | 526,982 | 522,144 | 570,959 / 574,135 |
| 3 | 266,770 | 48/48 | 48 | 526,982 | 522,144 | 588,200 / 585,080 |

Each trial used 96 HTTP attempts, no retries, no archive extraction and no cache
hits. Index parse us were 15,624, 22,447 and 19,456 respectively. In-run index
reuse/wait counters are zero before sharing is introduced.

## Stage 1 review

The metric-interface regression first failed on missing fields/aggregation.
The worker-panic regression then reproduced discarded counts (0 instead of 1),
before summaries were retained independently of worker return. Both now pass.

Five self-review passes:

1. Actual accounting: purpose counts include redirects, retry attempts, partial
   bytes and terminal failures; sleep and transport time are separate.
2. Unwind: attempt timing finishes on drop, worker summaries outlive return
   values, and a poisoned worker-local lock is recovered only after join.
3. Aggregation: one pure merge operation saturates sums and excludes wall time;
   a 256-case generated property verifies order independence against u128 sums.
4. Ownership/output: clocks stay in the binary, values/aggregation stay pure,
   and aggregate reports expose no source identities or URLs. Text formatting
   and partition distribution have named private owners instead of larger callers.
5. Baseline: three actual trials assert all 48 outcomes, distinct content,
   48 index parses/fetches, 48 metadata fetches and exactly 96 transport calls.

Strict Clippy, formatting, normal Rust tests and document validation pass at
this stage. Sharing and before/after equivalence remain the next stage.

## After index reuse

Measured with the same fixture and command in the task-2 working tree based on
`b19343b`, after binding the catalog to one HTTP context and sealing its endpoint
map. The benchmark validates all 48 results and compliant exit status as before.

| Trial | Wall us | Index requests/parses | Metadata requests | Total bytes | Index bytes | Sum of index/metadata request us |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 233,195 | 1/1 | 48 | 15,716 | 10,878 | 15,048 / 677,841 |
| 2 | 217,527 | 1/1 | 48 | 15,716 | 10,878 | 15,025 / 689,833 |
| 3 | 198,870 | 1/1 | 48 | 15,716 | 10,878 | 11,542 / 631,442 |

Each trial used 49 attempts, 47 in-run index hits, zero archive requests,
extractions, persistent evidence-cache hits or retries. Index parse us were
652, 583 and 664; coordination wait sums were 110,310, 109,267 and 85,345 us.
Ready hits add no wait; coordination excludes the owner's fetch and parse.
The earlier implementation trial series also observed exactly 49 requests,
15,716 bytes and one parse; its wall times were 236,351, 214,841 and 197,238 us.

Deterministic savings are 47 attempts and 511,266 received bytes (47 index
bodies), about 49% and 97% respectively. Median wall time in the final series
was 217,527 us versus 266,770 us before, about 18.5% lower. Scheduling, debug
instrumentation and filesystem work affect these small samples; overlapping
transport duration sums do not predict wall time. These synthetic results do
not establish internet or TLS performance.

## Warm process regression

The existing release-profile process test used 1,000 packages and 10,000
unique local archives (2,587,317 bytes). Initial population took 40.687406 s;
five warm processes took 0.677529, 0.546956, 0.551057, 0.537607 and 0.539152 s,
all below the existing two-second criterion. Each warm run reported 10,000
cache hits, zero HTTP/received bytes, zero archive extractions and zero index
parses. This tests cached local evidence rather than cold network throughput.

The measured executable SHA-256 was
`15b15d29a94aca2a237a105941683d8a494975d17bcd0ecd8365d0c4233369d5`.
It was built from the task-2 working tree before the final private constructor
refinement binding Http to the catalog lifetime. The lookup algorithm was
unchanged; this is evidence for that measured executable, not a timing claim
about a later source revision.

## Stage 2 review

The concurrent scope test first failed with 16 index requests instead of one.
It now verifies all 16 distinct locked hashes, one fetch/parse and 15 hits.
Five self-review passes covered:

1. Scope and identity: collect every endpoint's locked URL set before workers;
   preserve origin/path/query and validate each request's own locked digest.
   Generated JSON/HTML properties test URL queries and unrelated malformed links.
2. Synchronization and unwind: OnceLock shares terminal results; an initializing
   panic releases waiters for retry, retains already observed traffic and cannot
   silently remove failed targets. The map is immutable and needs no global lock.
3. Bounds and failures: retained associations are limited to locked URLs; duplicate
   links collapse to an ambiguity enum. Per-URL hash/duplicate failures preserve
   unrelated good associations. Shared terminal HTTP failure still allows each
   artifact's archive fallback. New acquisition calls get fresh catalogs.
4. Accounting and regression: actual benchmark counts/bytes match saved index
   bodies; timings overlap and are qualified. Sufficient warm evidence and offline
   execution perform zero HTTP/index parsing. Report shape remains additive.
5. Proof and architecture: the small Lean model proves filtering/lookup properties
   under canonical-key premises; 216 generated cases exercise both real parsers
   and sidecar proof strength. Runtime I/O/clocks stay outside the pure core.

Implementation rulings: continue in the authorized feature checkout rather than
create a second worktree (filesystem isolation is weaker); retain worker-local
summary locks outside worker returns (parent must recover poison after joining);
bind the catalog to one immutable Http at construction and seal all endpoint keys
(auth context cannot vary by lookup; dynamically adding endpoints needs redesign).
Terminal failures are shared only for this call, with conservative independent
archive fallback; there is no disk index freshness policy.

Software-design-philosophy assessment: seven of eight diagnostics supported,
approximately 8.75/10. The catalog hides endpoint/media/parse/synchronization and
lookup rules behind construction and sidecar lookup; interface comments describe
scope, panic and proof guarantees. Review includes design depth and ownership.
The remaining diagnostic is a measured 10–20% design-effort fraction: time was
not tracked, so it cannot be claimed. Record that fraction in a future task to
substantiate all eight diagnostics; no runtime change is needed for this row.
