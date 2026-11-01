# HTML index parser measurements

macOS arm64, Rust release profile, 2026-11-01. Baseline:
`56fff43cdec130da66510a4f1bb5f135efbf9544` plus the new ignored benchmark.
Candidate: private streaming HTML sink with borrowed attributes. No dependency
or input fixture change between samples. Five samples per case, microseconds:

| Historical links | Body bytes | Retained-tag baseline | Streaming + borrowed attributes |
| --- | --- | --- | --- |
| 1,000 | 244,086 | 2047, 1840, 1828, 1843, 1863 | 2057, 1723, 1712, 1717, 1766 |
| 10,000 | 2,449,086 | 19936, 20066, 19738, 19296, 19574 | 17231, 16985, 17293, 17906, 17457 |

Medians: 1,843 → 1,723µs and 19,738 → 17,293µs, respectively about 6.5% and
12.4% lower on these synthetic inputs. Five local samples without statistical
controls are limited observations, not a general speed guarantee. The benchmark
constructs bodies outside the timer; parsing/filtering is inside, and the parsed
output is destroyed after the timer. Each parse must retain exactly one locked
entry. This does not measure HTTP, evidence parsing, full CLI speed or peak RSS.

An intermediate streaming sink still copied per-tag attribute maps. Its samples
were 2197, 1908, 1900, 1939, 1936µs (1,000) and 20632, 19187, 19427, 19389,
19308µs (10,000): broadly neutral against the baseline. This supported removing
the redundant attribute copies too. No larger speedup is inferred from removing
the retained tag vector alone.

The structural result is independent: no second vector of all historical tags,
tag-name Strings or copied attribute maps remains. Output bindings are bounded
by locked URLs, with one selected base URL and absorbing parse outcome. The
response body, tokenizer state and current tag still occupy memory; this is not
constant total memory or an allocator-count/RSS claim. Existing HTTP bounds stay
unchanged. JSON parsing still constructs its existing Value representation.

Reproduce from the repository root:

```sh
cargo test --release --locked --bin taloon historical_html_index_parse -- --ignored --nocapture --test-threads=1
```

The existing full-CLI warm goal was measured at `56fff43`, before this follow-up;
see [its measurements](../evidence-ownership/performance.md). It uses offline
local artifacts and does not exercise HTML index parsing, so repeating it would
not establish an HTML benefit. Those timings retain that revision/environment.
