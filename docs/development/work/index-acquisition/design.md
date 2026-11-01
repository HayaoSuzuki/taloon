# Measured per-run index reuse

Status: approach approved; detailed design proposed and builder-reviewed.
Source baseline: `4242995`.
The owner requested request/byte/wait measurements and reduced duplicate index
acquisition, selected per-run sharing before persistent index caching, and
requested software-design-philosophy. Existing inline execution and five review
passes per stage remain the development method.

## Scope and approach

Keep all locked artifact variants and current archive/sidecar proof requirements.
Do not change policy, offline scope, credentials, redirects, retry limits or
timeouts. Add measurements first, then remove repeated index fetch/parse work.
Use synthetic, repeatable protocol fixtures for publishable comparisons. Account
credentials, source URLs and dependency names do not appear in aggregate metrics.

The owner selected an in-memory index snapshot shared by acquisition workers for
one check. A persistent HTTP/index cache would additionally require freshness,
credential and invalidation contracts; it is deferred. Increasing worker count
would retain duplicate traffic and is not the first optimization.

## Measurement contract

Keep existing `network_requests`, `downloaded_bytes`, `archive_extractions` and
`cache_hits`. Add fields to the schema-1 acquisition object; additions do not
change the meaning of existing fields or verdicts.

- `elapsed_us`: wall elapsed acquisition time across all workers, including cache
  work and waiting. It excludes input discovery and final policy/report rendering.
- `http.index`, `http.metadata`, `http.archive`: each has `requests`,
  `downloaded_bytes`, `request_time_us`, and `retry_delay_us`.
- `index_cache_hits`: lookups reusing another lookup's index snapshot, including
  lookups that wait for its in-flight initialization.
- `index_wait_us`: accumulated time waiting for an in-flight index snapshot.
- `index_parse_count` and `index_parse_us`: actual index parse attempts and time.

Count every actual HTTP attempt, including retries and redirects, in its purpose.
Preserve actual received-byte accounting on partial/failed responses. Measure
transport duration around `Transport::send` through body receipt; measure retry
sleep separately. Monotonic integer microseconds avoid rounding away short
parses; saturate conversions and sums rather than wrap. Aggregate durations may
overlap across workers and must not be described as wall time or added to it.
Text reports present a compact breakdown; JSON retains named counters/units.
No per-URL tracing, new flag or runtime dependency is required.

Keep worker measurements independently of a successful worker return. A worker
panic must not discard already observed requests/bytes; elapsed attempt timing
must also finish during unwind. Test a nonzero attempt followed by initializer
panic with another worker waiting. This does not imply measurement can survive
process abort or count bytes a transport never reported.

## Module ownership

`http` owns attempt/retry/redirect accounting and accepts a typed request purpose.
The report layer owns pure metric values and one aggregation operation; it reads
no clock. The binary measures acquisition wall time once, rather than summing
worker wall times. The CLI renders metrics; they never determine compliance.

The index module becomes a deep lookup module: it owns package endpoint
construction, accepted representations, bounded parsing, per-run synchronization,
and exact sidecar associations. Acquisition asks for a sidecar association and
does not manage index HTTP or cache internals. Comments document lookup scope,
failure reuse, memory bounds and proof guarantees before implementation.

## Snapshot and concurrency rules

Create a fresh index lookup scope inside each acquisition call, even when an
Acquirer instance is reused. Key by the constructed request endpoint before
redirects, retaining
origin, path and query, with ignored userinfo/fragments removed consistently with
HTTP handling. The HTTP authentication context is immutable within this scope.
Different sources/endpoints do not share snapshots merely by package name.

Each key has a shared once-initialized result: one worker fetches/parses it;
others wait and reuse that result. Retain terminal failure after bounded retries
for this run; each artifact independently falls back to full archive verification.
An initializer panic must release waiters or permit another initialization, not
leave a permanently pending entry. Never hold the global entry-map mutex during
network I/O. A new acquisition call observes the index afresh when needed.
Final redirect URLs still determine resolution of relative artifact links.

Collect the complete normalized locked-URL set for each endpoint before any
initializer starts; the first artifact must not define the snapshot's filtering
scope. Parse each fetched document once. Retain associations only for URLs
requested by the locked scope, not every historical release on the index. Bound input at the
existing 16 MiB limit. Collapse repeated matching links into an explicit ambiguous
state rather than retaining an unbounded duplicate list. Preserve existing JSON/
HTML version, base-URL, duplicate-link, URL/hash and metadata-hash checks. Ignore
invalid unrelated links as before. A bad hash or ambiguity remains specific to
its exact URL, rather than invalidating unrelated valid associations. A shared snapshot is not artifact proof:
every lookup still verifies its own exact locked URL and archive hash; metadata
bytes and package/version identity are still checked after download.

No metadata-body or archive-cache redesign is included. Their existing proof and
persistence rules continue to apply. Warm sufficient artifact evidence and
offline execution must cause zero index/network access.

## Implementation and acceptance

1. Add typed accounting, pure aggregation, and an opt-in controlled cold-acquisition
   benchmark. Measure the unchanged fetch behavior with a fixed synthetic index,
   distinct locked artifacts, response sizes and transport latency. Test retries,
   redirects, partial failures and metrics before committing the stage.
2. Add per-run shared parsed index lookup, concurrency/failure/source isolation
   tests, and compare the same benchmark. All artifact results and decisions stay
   equivalent. Assert deterministic request/byte/parse reductions; record timings
   from repeated runs rather than asserting an internet latency threshold.

Use actual HTTP-path fixtures with controlled transport, generated URL/source/
permutation cases, and a small dependency-free Lean model for association/reuse
invariants where useful. Do not claim a model proves Rust synchronization, HTTP,
or legal completeness. Run relevant strict checks, warm-cache regression,
five self-review passes per stage and one fresh independent final review. Update
the owning contracts and user guidance, then commit/push/create a PR through gh.
Do not merge or publish prematurely; the normal main workflow owns publication.

Current design assessment: 6/8 diagnostics, approximately 7.5/10. Implementation
changes currently cross the index/acquisition boundary, and lookup cost/guarantees
are not documented at the interface. Encapsulation and lookup contracts above
address these rows. Reassess all eight diagnostics after implementation; do not
claim a measured design-effort fraction without recording it.
