# Policy performance evidence

Base production: `6460fe942a5c2703238a3d3c5ee8c03382e20496`. Follow-up working tree on the same host: macOS 15.7.7 arm64, Apple M4 Pro, 24GiB RAM, Rust 1.99.1 release profile with thin LTO. Synthetic public Policy calls only, with fixture/config parsing outside the timer. Each case runs minimum_evidence and evaluate for 2,000 requests, for five samples. Host activity was not isolated. No network or full CLI timing is inferred.

GlobalPermissions configures 256 compiled SPDX list entries plus MIT, with no exceptions. ScopedExceptions configures one MIT exception per distinct package identity (2,000 entries) and no global permissions. All decisions must remain permitted.

Benchmark source SHA-256: `54f4fc47ff4f1ef131ad5df6b66a78bb2076d1ea273231e05d398601e0092740`.

| Workload | Before samples µs | After samples µs | Before median µs | After median µs | Ratio |
| --- | --- | --- | ---: | ---: | ---: |
| GlobalPermissions | 10214, 10177, 10114, 10235, 9981 | 258, 268, 269, 266, 263 | 10177 | 266 | 38.26× |
| ScopedExceptions | 23975, 23881, 24014, 24160, 23652 | 491, 463, 481, 460, 461 | 23975 | 463 | 51.78× |

Measurements isolate immutable-set copying and exception lookup work. They do not quantify the unused-warning pass independently, policy parsing, common tiny configurations or end-to-end speed. Index construction trades some setup work for reusable lookup; no blanket speedup claim follows.

```sh
cargo test --release --locked --test policy_performance -- --ignored --nocapture --test-threads=1
```

Normal CI ignores these timing cases and has no elapsed-time assertion.


## Final CLI goal check

The final follow-up release binary also passed the existing 1,000-package /
10,000-distinct-artifact local process fixture. Cold local acquisition: 42.944229s.
Warm durations: 0.541981, 0.551220, 0.553177, 0.555705, 0.542498s. All five met the existing two-second goal,
retaining 10,000 cache hits with zero network requests/downloaded bytes and zero
archive re-extractions. These goal checks do not attribute whole-CLI gains to
the policy changes; earlier campaigns had different background machine load.

Reproduce with the existing process benchmark:

```sh
cargo build --release --locked
TALOON_BENCH_BINARY=target/release/taloon cargo test --release --locked --test performance -- --ignored --nocapture
```
