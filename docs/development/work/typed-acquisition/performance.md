# Typed acquisition measurements

macOS arm64, release profile, 2026-11-01. Baseline
`df637758fa6bb3349981641c68f63bf60f0950eb` plus the same ignored harnesses.
Five local samples per fixture, microseconds. No dependencies changed.

## Worker grouping

Requests have unique locators and share one content digest per pair, all wheels.
Construction/hashing is outside timing. Partition construction is inside;
returned partitions are dropped after timing. Each sample checks full coverage.

| Requests | String/debug key baseline | Typed key |
| --- | --- | --- |
| 1,000 | 1158, 1039, 928, 905, 932 | 94, 50, 49, 38, 30 |
| 10,000 | 4719, 4638, 4620, 4625, 4752 | 969, 912, 896, 976, 844 |

Medians: 932 → 49µs and 4,638 → 912µs. No digest formatting or key String
construction remains. Empty scope now has no partition or worker. Worker
assignment/order can change under the owner's no-compatibility instruction.

## JSON index

Each fixture has 1,000/10,000 historical files and one locked target. Consumed
fields include URLs and hashes; unknown nested extensions deliberately exercise
ignored data. Construction/serialization is outside timing, parsing/filtering and
intermediate-document destruction inside. Returned output is dropped afterward.

| History | Body bytes | Value baseline | Typed object-only records |
| --- | --- | --- | --- |
| 1,000 | 456,012 | 1015, 999, 984, 1002, 1070 | 633, 586, 581, 579, 943 |
| 10,000 | 4,578,012 | 12773, 12579, 13072, 12644, 12775 | 6391, 5859, 5472, 5612, 5083 |

Medians: 1,002 → 586µs and 12,773 → 5,612µs. Typed decoding omits arbitrary
Value/extension trees but retains a complete file vector and consumed Strings.
This is not streaming or constant-memory decoding. No peak RSS or optimized
allocator-call count was measured.

Both measurements exclude HTTP, archives, policy and complete CLI/report work.
Five samples with uncontrolled local scheduling/frequency are limited evidence,
not a general speed guarantee. Stricter malformed-index rejection can require
additional archive downloads; that deliberate behavioral cost is not measured
by valid-index fixtures. Full CLI JSON/offline timings at older revisions remain
historical and do not establish this change's network behavior or total speed.

Reproduce from the repository root:

```sh
cargo test --release --locked --bin taloon worker_grouping_measurement -- --ignored --nocapture --test-threads=1
cargo test --release --locked --bin taloon historical_json_index_parse -- --ignored --nocapture --test-threads=1
```

The initial typed decoder accepted positional arrays, identified by independent
review and corrected before delivery. Its earlier samples were 963, 759, 746,
689, 672µs and 5713, 5450, 4048, 4145, 4187µs. They are superseded by the
object-only samples above; no overhead attribution is made across uncontrolled
local runs. Final shape regressions, not speed, decide decoder acceptance.

## Final CLI warm goal

After the object-only correction, the release executable exercised the existing
1,000-package/10,000-distinct-artifact offline fixture (2,587,317 archive bytes).
Cold local publication took 44.300111s with 10,000 extractions and zero cache hits.
Warm seconds: 0.580776, 0.542736, 0.550191, 0.554233, 0.554000. Each warm run reported 10,000 hits, zero archive
extractions, zero HTTP requests and zero downloaded bytes. Every sample satisfies
the two-second warm goal. This is not a controlled comparison against historical
CLI times; it does not exercise JSON network responses or measure cold HTTP.

```sh
cargo build --release --locked
TALOON_BENCH_BINARY=target/release/taloon cargo test --release --locked --test performance -- --ignored --nocapture
```
