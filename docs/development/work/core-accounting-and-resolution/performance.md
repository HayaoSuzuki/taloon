# Core graph performance evidence

Local measurement on 2026-11-01: macOS 15.7.7 arm64, Apple M4 Pro, 24GiB RAM, Rust 1.99.1, release profile (thin LTO). The same host and synthetic benchmark inputs were used before and after; background machine activity was not isolated. No network work is measured here.

Before: production sources at `9a8e2a1f26176ad663fa2fed74f83e88260d04b2` plus the new measurement test. After: this task's working tree. Fixture source SHA-256: `6c0e061d00487255ea7ce3ecca96d62d6e4d80e28d2ed2485640a4ab9a3ca050`.

Chain: 1,500 external packages and one outgoing edge per package. Fanout: 3,000 packages with up to four forward edges. A virtual root selects the first package. All external targets are retained even when they lack artifacts. Each sample parses the lock, then queries the last package's shortest path and asserts path depth. Construction and path timings are separate.

| Shape / phase | Before median (µs) | Final after median (µs) | Ratio before / after |
| --- | ---: | ---: | ---: |
| Chain / lock construction | 7986 | 3318 | 2.41× |
| Chain / shortest path | 28610 | 983 | 29.10× |
| Fanout / lock construction | 76116 | 14238 | 5.35× |
| Fanout / shortest path | 109218 | 2725 | 40.08× |

## Complete samples

| Stage / shape | Construction µs, samples 1–5 | Path µs, samples 1–5 |
| --- | --- | --- |
| before / Chain | 10571, 8026, 7986, 7475, 6870 | 34077, 27492, 28610, 26160, 29123 |
| before / Fanout | 75598, 75444, 76444, 78771, 76116 | 104589, 105949, 109218, 110682, 115320 |
| intermediate after / Chain | 7161, 5648, 3974, 3678, 3970 | 1034, 1015, 997, 1065, 982 |
| intermediate after / Fanout | 19073, 47856, 32450, 18754, 47202 | 9371, 6000, 2996, 3730, 2955 |
| final after / Chain | 3606, 3243, 3129, 3318, 4124 | 983, 960, 942, 1015, 1346 |
| final after / Fanout | 14441, 14238, 13271, 12985, 14711 | 2673, 2674, 2752, 2818, 2725 |

The intermediate series ran earlier in verification and was noisier. The final series ran after other task test/build processes had exited; both series show improvement. These small synthetic timings are not general speed guarantees and do not isolate each optimization's contribution.

## Full CLI goal check

The existing 1,000-package / 10,000-distinct-artifact local-cache process fixture also passed using the new release executable. Cold local acquisition: 49.862552s. Warm process durations: 0.880296, 1.060040, 1.060125, 1.199119, 1.373035s. Every warm report retained 1,000 packages and 10,000 artifacts, with 10,000 cache hits, zero network requests/downloaded bytes and zero archive re-extractions. All five met the existing two-second goal. Other verification ran during this campaign; these figures establish the goal in this environment, not a comparison to earlier campaign timings.

## Reproduction

```sh
cargo test --release --locked --test design_performance -- --ignored --nocapture --test-threads=1
cargo build --release --locked
TALOON_BENCH_BINARY=target/release/taloon cargo test --release --locked --test performance -- --ignored --nocapture
```

Normal CI does not enforce wall-clock thresholds. Index/network and filesystem conditions require separate measurements. Candidate follow-ups are policy-override scans, repeated version/specifier parsing within same-name forks, repeated record validation when rebinding immutable evidence, and output allocation costs; no measured claim about their payoff is made here.
