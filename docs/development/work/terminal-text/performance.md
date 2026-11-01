# Terminal text measurements

macOS arm64, Rust release profile, 2026-11-01. Baseline
`efb0cff5b47bab14d494a17dc3a87118e514d21a` plus the ignored benchmark;
candidate changes only the private terminal helper to Cow<str> and a shared
predicate. Each fixture repeats its pattern 1,024 times; each timed sample calls
the helper 100 times. Five samples per class, microseconds:

| Input | UTF-8 bytes | String/per-character-Vec baseline | Cow/direct append |
| --- | --- | --- | --- |
| ASCII locator | 34,816 | 43869, 38669, 34405, 35366, 34086 | 1516, 1507, 1503, 1484, 1482 |
| Japanese/Unicode | 41,984 | 15410, 15839, 15834, 15963, 15598 | 1107, 1106, 1118, 1139, 1154 |
| Mixed controls/bidi | 26,624 | 28325, 28018, 28183, 28195, 28278 | 2785, 2782, 2921, 2888, 2889 |

Medians: ASCII 35,366 → 1,503µs; Unicode 15,834 → 1,118µs; mixed escaped
28,195 → 2,888µs. These are synthetic helper measurements, not complete report
or CLI speedups. Fixture construction/logging are outside the timer; helper
calls and returned-value destruction are inside. A black box prevents discarding
the calls. No statistics beyond the five samples, allocator counts or peak RSS
were measured. Ordinary Cow results borrow their input; escaped results still
allocate and may grow their output String. Removing source-level Vec construction
does not establish an exact count of optimized allocator calls.

Reproduce from the repository root:

```sh
cargo test --release --locked --bin taloon terminal_text_rendering -- --ignored --nocapture --test-threads=1
```

The existing [full CLI warm-cache measurement](../evidence-ownership/performance.md)
at `56fff43` uses JSON output, so it does not exercise this helper and was not
repeated as evidence for this change. Its timings retain their original revision
and environment. Parsing/acquisition measurements remain separate.
