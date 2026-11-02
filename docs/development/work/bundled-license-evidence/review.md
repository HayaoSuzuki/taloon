# Bundled license evidence: design and verification

The owner requested automatic handling of absent declaration fields, minimizing
HTTP requests/bytes, and selected recommended defaults through PR creation. This
extends the earlier declaration-only contract, which explicitly excluded license
text inference. The [plan](plan.md) records the implemented replacement boundary
and alternatives. Invalid/repeated explicit fields still do not fall back.

## Design and ownership

The existing spdx dependency supplies canonical texts; no runtime license database
request or new scanner dependency is added. A private matcher supports complete
MIT, BSD-2-Clause, BSD-3-Clause, ISC and Apache-2.0 templates. Formatting and
attribution substitutions are deliberately limited. It does not use similarity
scores, invent multi-file OR/AND relationships, or prove legal equivalence.
AUTHORS basenames are an explicit attribution classification, not proof that
those files contain no additional conditions. Other unknown selected files block
inference; hash-bound clarifications remain available.

The archive reader consumes already verified bytes and the validated metadata
root. It checks exact declared paths (or narrow conventional names), uniqueness,
regular-file status, UTF-8 and limits before retaining text. A source archive
needs its existing metadata pass and one bounded selected-file pass; no duplicate
root-discovery pass or arbitrary-member collection is retained. Matching data
initializes once per process with LazyLock.

Evidence stores optional raw file observations under the existing checksum and
reinterprets them with the current matcher. None serializes as before, preserving
unrelated existing cache hits. A completed unknown/unavailable scan remains
cached; a missing observation is a distinct state. Sidecar records cannot claim
archive-file observations. Acquisition asks the evidence owner whether files are
still needed, then upgrades old/weak observations directly to an archive. Offline
never upgrades over HTTP. Full archive SHA-256 remains required, so range requests
cannot reduce received bytes while preserving that guarantee and would add trips.

## Red/green and review evidence

- Matcher: named canonical/variant cases and independent generated formatting
  and changed-term properties; initial missing implementation cases failed.
- Reader: wheel/sdist exact paths, legacy/modern ambiguity, metadata namespace,
  links/duplicates/missing files, UTF-8, generated roundtrips and all bounds.
  Initial positive cases failed; root-validation regressions failed before the
  single-scan refinement, then passed.
- Public evidence: four absent-field/record cases failed before integration and
  pass afterward. Named cases retain invalid/repeated/legacy precedence, file
  hashes/original-field provenance, attribution roles and multi-file ambiguity.
- HTTP: absent sidecar and warm recognized cases failed before archive escalation;
  cached online sidecar upgrade also failed before the fix. Named request/byte
  checks retain the valid-metadata fast path, direct upgrade, completed unknown
  cache reuse and network-free offline behavior.
- Cache: old omitted fields/checksums, forbidden sidecar observations and restored
  size/count bounds have regressions. A maximum-size escaped metadata/file/path
  observation roundtrips through the actual disk cache; it passed without a new
  bound adjustment, confirming the proposed capacity concern did not reproduce.

Five passes were applied at each stage: requirements/precedence; actual production
connection; state/storage/bounds and performance; independent fixed/generated
adversarial observations; and sources/reproduction/documentation ownership.

A fresh independent reviewer found two Important issues: a title-case restrictive
sentence could be consumed as a copyright holder after line folding, and accepted
paths over 4,096 bytes could later fail cache restoration. Three attribution
regressions failed before their fix; original line structure and sentence
boundaries are now retained for substitutions. Path overflow failed before the
shared extraction/restoration bound; public at/over-bound rebinding cases pass.
No blacklist expansion or second independent approval is claimed. The reviewer
found no further required precedence, proof-strength or acquisition correction;
it did not certify legal equivalence or rerun the final full suite.

Final design assessment: 10/10 against the eight software-design diagnostics for
this scoped change. The earlier missing evidence-owned escalation query is
resolved. Format/path/template rules remain private, I/O stays acquisition-owned,
and no generic scanner/transport abstraction or parallel status store was added.
This score evaluates structural complexity, not semantic/legal proof completeness.

## Public reproduction and measurement

An isolated temporary project locked the exact public sqlparse 0.6.0 sdist and
wheel hashes reported in the example. Neither artifact is committed. Both are
recognized as BSD-3-Clause; JSON retains the absent original fields and records
AUTHORS/ LICENSE paths, roles and observed hashes. The same LICENSE bytes occur
in both, but each artifact is independently verified.

Observed macOS arm64 cold CLI result: compliant, exit 0; 4 HTTP requests and
264,336 response-body bytes: one index (29,741), one sidecar (6,048), two archives
(228,547). A subsequent check and offline describe both exit 0 with two cache hits,
zero requests and zero received bytes. These are one public release/environment's
measurements, not a general latency or bandwidth bound. Retries, corruption,
cache clearing/eviction and distinct artifact hashes can require further requests.
No external repository license endpoint is consulted at runtime.

Existing Lean/Creusot models and claims remain unchanged; matching/ZIP/tar parsing,
legal completeness and clock/network correctness are not proved by those models.
Synthetic bundled-license wheel/sdist fuzz seeds use canonical MIT text from the
existing spdx dataset, with fixture package identity only. The wheel and sdist
fuzz targets each completed 1,000 bounded executions without a crash. Commands
and ordinary verification gates remain in the developer entry point.

Final local integrated checks: 529 Rust tests and 529 randomized-order tests pass,
with existing ignored fixtures/benchmarks retained. Strict Clippy, formatting,
rustdoc, build, documentation/OKF links, strict user-site build, Lean build and
three corpus freshness checks pass. The final executable reuses the public
sqlparse cache for check and offline describe with zero requests/bytes. Python
source and workflow definitions are unchanged. cargo-deny is not installed in the
local environment, so root/fuzz dependency-policy enforcement is left to the
existing required CI job; no local policy-pass claim is made. Hosted results are
recorded separately on the PR.
