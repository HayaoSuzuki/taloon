# Process performance evidence

Measured on 2026-10-31 with the release executable, from process spawn until
stdout/report completion and child exit. This is a local synthetic benchmark,
not a remote-download or general hardware guarantee. The two-second target is a
measured goal; normal CI does not assert elapsed time.

## Environment and fixture

- macOS 15.7.7, Darwin arm64, Apple M4 Pro, 24 GiB RAM, APFS.
- Rust 1.99.1; release profile with thin LTO and stripped symbols.
- 1,000 synthetic external packages, 10 recorded wheel artifacts each, no
  dependency/environment filtering. All 10,000 SHA-256 values are distinct and
  present in the lock. Archives total 2,587,317 bytes.
- Each archive contains matching package/version metadata, an MIT declaration,
  and a distinct variant header. Files are local archives in a temporary project;
  local content is read and rehashed on every run, including warm runs.
- The first real CLI invocation populates an isolated cache. Subsequent invocations use that cache.
- The fixture generator and assertions are in [the process test](../../../../tests/performance.rs),
  SHA-256 `381f764e916c60f7eebe4741525b3e70b5b5d9e6096f83ea655892e7fd6af57c`.
- Measured executable SHA-256:
  `fcabdf0f4ceaa16ddb31815a2108d9630b1467a80cb00c706480938ffdaae82c`.
  It was built from the working tree based on `a4914e4` with the stage-8 runtime
  corrections. The machine was not isolated from other activity.

## Observations

| Run | Seconds | HTTP requests | Downloaded bytes | Archive extractions | Cache hits |
| --- | ---: | ---: | ---: | ---: | ---: |
| Cold local evidence cache | 41.447161 | 0 | 0 | 10,000 | 0 |
| Warm 1 | 0.608586 | 0 | 0 | 0 | 10,000 |
| Warm 2 | 0.553469 | 0 | 0 | 0 | 10,000 |
| Warm 3 | 0.562964 | 0 | 0 | 0 | 10,000 |
| Warm 4 | 0.575524 | 0 | 0 | 0 | 10,000 |
| Warm 5 | 0.565200 | 0 | 0 | 0 | 10,000 |

Every report was complete/compliant with exactly 1,000 packages and 10,000
artifact results. These are measured acquisition counters, not values inferred
from --offline. All five warm runs met the two-second goal in this environment.
The cold run includes archive interpretation and synced atomic cache publication;
it is not a measurement of network acquisition. Warm local reads/rehashing remain
part of the measured work.

An earlier run before the final runtime corrections measured a 46.574217-second
cold local run and warm durations 0.667953, 0.556634, 0.570084, 0.566928, and
0.560280 seconds. Tool installation/compilation ran concurrently during that
campaign. Retain those observations as earlier evidence, not an independent
performance guarantee. An intermediate run before the cache serialization-bound
and buffered-publication correction measured 45.452586 seconds cold and warm
durations 0.678761, 0.548830, 0.552170, 0.561576, and 0.546471 seconds.

An additional pre-identifier checkpoint measured 40.818807 seconds cold and warm
durations 0.607028, 0.626674, 0.644444, 0.619909, and 0.676481 seconds.

## Reproduction

From the repository root:

```sh
cargo build --release --locked
TALOON_BENCH_BINARY=target/release/taloon cargo test --release --locked --test performance -- --ignored --nocapture
```

Re-measure after changes to acquisition/cache lookup, completeness aggregation,
metadata retention, or report serialization. Larger archives, different storage,
cold OS caches, and remote cold acquisition require separate measurements.
