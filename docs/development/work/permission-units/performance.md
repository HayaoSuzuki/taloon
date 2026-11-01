# Permission unit performance guard

Base production: `1cc77aa737829e56306e57182dadcb9bdd719bea`; after: typed permission-unit working tree. macOS 15.7.7 arm64, Apple M4 Pro, 24GiB RAM, Rust 1.99.1 release/thin LTO. Same existing 2,000-request policy benchmarks, five samples each, with setup/parsing excluded. Background activity was not isolated.

| Workload | Before µs | After µs | Before median µs | After median µs |
| --- | --- | --- | ---: | ---: |
| GlobalPermissions | 282, 262, 250, 258, 280 | 239, 227, 228, 232, 228 | 262 | 228 |
| ScopedExceptions | 512, 498, 497, 511, 517 | 475, 522, 479, 493, 502 | 511 | 493 |

The results remain in the same sub-millisecond range. No speedup is attributed to this type change: the main gain is preventing unvalidated/non-atomic permission values and keeping equality/ordering contracts explicit. Storage size, parser setup, warning-pass costs and whole-CLI performance are not measured here. There is no claim of zero overhead or a universal speedup.

```sh
cargo test --release --locked --test policy_performance -- --ignored --nocapture --test-threads=1
```

No wall-clock threshold is enforced by ordinary CI. The earlier full CLI goal check remains evidence about its recorded revision/environment, not a new measurement of this follow-up.
