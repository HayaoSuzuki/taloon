# Interactive acquisition progress verification

Reviewed on 2026-11-01 in the working tree based on `4ddf958`. The binary
previously joined all acquisition workers before rendering a report, leaving a
terminal silent during slow I/O. All verification inputs here are synthetic.

The blocked-transport regression first failed because the progress interface did
not exist. It now observes the initial 0/total state before transport completes,
then another notification while transport remains blocked, followed by monotone
counts through the complete request scope. Shared-content requests retain
distinct accounting. A second test covers panicked workers and an empty scope.

## Five self-review passes

1. Liveness: workers send completion events; the calling thread observes them
   with a one-second timeout before joining. A slow first request cannot hide the
   initial notification or heartbeat. Completion senders close on worker unwind.
2. Accounting: progress counts terminal outcomes, including failures; callback
   state never chooses a policy result. Panicked-worker outcomes are accounted
   for after join, and zero requests terminate without waiting for a message.
3. Output: display occurs only on interactive stderr, throttled to roughly once
   per second with initial/final updates. A real local pseudoterminal verified
   both endpoints with isolated JSON stdout for exits 0/1/2. Piped stderr remains
   quiet in the CLI regression. Display includes only counts and elapsed time.
4. Boundaries: the callback type belongs to binary acquisition. Clock, terminal
   detection and rendering belong to the CLI. No public library, permission,
   evidence, scope, timeout or dependency changes were made; the Lean policy
   model is unaffected. Existing Rust tests still execute its corpus adapter.
5. Integration/performance: all 79 normal Rust tests, strict Clippy, formatting,
   document checks and strict MkDocs passed. The opt-in release process benchmark
   retained the two-second warm target with no network or archive extraction.

## Synthetic process benchmark

Same macOS arm64 environment and fixture method as the
[initial process record](../initial-design/performance.md): 1,000 packages,
10,000 distinct local archives, 2,587,317 archive bytes. The release process uses
piped stderr, as a noninteractive caller would. Elapsed time is measured from
process spawn through report and exit, not inferred from acquisition counters.

| Run | Seconds | Cache hits | Extractions | Requests/bytes |
| --- | ---: | ---: | ---: | ---: |
| Cold local cache | 40.815975 | 0 | 10,000 | 0/0 |
| Warm 1 | 0.716488 | 10,000 | 0 | 0/0 |
| Warm 2 | 0.551745 | 10,000 | 0 | 0/0 |
| Warm 3 | 0.552945 | 10,000 | 0 | 0/0 |
| Warm 4 | 0.542070 | 10,000 | 0 | 0/0 |
| Warm 5 | 0.550694 | 10,000 | 0 | 0/0 |

These local measurements are not a remote-download guarantee. No initial online
latency target is introduced. Interactive rendering adds periodic terminal I/O;
it does not change which artifacts are inspected or cache policy decisions.
