# Immutable payload measurements

macOS arm64, Rust release profile, 2026-11-01. Baseline is
`e7badc3ee22f5b5756ee0c1d0099ff44ab7461a0`; the candidate changes only serde's
rc feature and private EvidenceRecord payload storage. The same uncommitted
`tests/evidence_performance.rs` harness was used on both representations.

## Metadata-heavy evaluation

Each sample evaluates 64 synthetic wheel observations with 256 KiB padding per
metadata header. Construction, archive verification and input cloning are outside
the timer; evaluation, result construction and destruction are inside. The
harness asserts success through public Inspection/Description APIs. Five samples
per consumer, microseconds:

| Consumer | Deep-copy baseline | Shared immutable payload |
| --- | --- | --- |
| check | 62535, 62289, 62940, 63297, 67675 | 66250, 64835, 65291, 64638, 64903 |
| describe | 68129, 70258, 66865, 68475, 66755 | 64015, 64051, 62834, 63000, 65614 |

Medians are check 62940 → 64903 and describe 68129 → 64015. Opposite directions,
small absolute differences and uncontrolled local scheduling do not establish an
overall speedup. The structural result is narrower: rebind/to_record clones no
longer allocate/copy the metadata String. Parsing and checksum serialization
still process the entire header. Peak RSS and allocation counts were not measured.
One retained Arc allocation and atomic increments/decrements are added per shared
record; locked serde also allocates a transient Box<RecordData> during Arc
deserialization. Neither allocation copies the metadata String buffer.

Reproduce from the repository root:

```sh
cargo test --release --locked --test evidence_performance -- --ignored --nocapture --test-threads=1
```

The earlier consuming/borrowing API prototype was also measured, then discarded.
It did not demonstrate a decisive timing advantage worth additional public APIs.
No timing limit is added to regular tests.

## Full CLI goal

The existing opt-in process benchmark creates 1,000 packages and 10,000 distinct
local archives. It exercises acquisition, JSON persistence, complete cache reload
and output. This synthetic offline workload does not measure HTTP cold starts.
Run commands are unchanged:

```sh
cargo build --release --locked
TALOON_BENCH_BINARY=target/release/taloon cargo test --release --locked --test performance -- --ignored --nocapture
```

On the final shared-payload candidate, the fixture contained 2,587,317 archive
bytes. Cold local run: 45.297119 seconds, 10,000 archive extractions, zero cache
hits. Warm samples: 0.877382, 1.068548, 1.058590, 1.170872, 1.134530 seconds.
Each warm run reported 10,000 cache hits, zero archive extractions, zero network
requests and zero downloaded bytes. All samples meet the two-second warm goal.
Cold publication performs durable writes and is outside that warm goal. Local
verification/build activity overlapped this run; these observations do not form
a controlled regression comparison against historical CLI timings.
