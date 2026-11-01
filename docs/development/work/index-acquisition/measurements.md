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
