# Core accounting and graph design assessment

Inspected base: `9a8e2a1f26176ad663fa2fed74f83e88260d04b2` on 2026-11-01.
The owner requested assessment and implementation of worthwhile performance,
type/Enum and structural improvements using software-design-philosophy. Existing
CLI, JSON, public Rust entry points and evidence guarantees are constraints.
Internal layout and algorithms are delegated implementation choices.

## Assessment

The eight diagnostics give the inspected Rust core 6/8, or 7.5/10. This is a
qualitative assessment of the inspected boundaries, not a correctness metric.

| Diagnostic | Base | Evidence / change needed |
| --- | --- | --- |
| One-sentence responsibilities | Pass | Resolution, policy, evidence, inspection, description and binary-owned acquisition have distinct responsibilities. |
| Interfaces simpler than implementations | Pass | Documents become opaque inspections/descriptions and structured reports. |
| Replaceable implementations without caller changes | Fail | Both inspection and description know how result multiplicity, missing IDs and rebinding work. Give shared observation accounting one private owner. |
| Abstraction comments explain guarantees | Fail | Many legacy model accessors/constructors use generic validation comments. Document identity, normalization, proof and ordering contracts beside the APIs, including new internal boundaries. |
| Design discussion in reviews | Pass | Current OKF and staged reviews discuss ownership and invariants. |
| Each module hides meaningful knowledge | Pass | Lock, policy, archive, index and cache representations have concrete owners. |
| Newcomer can understand boundaries | Pass | Module docs and the sans-io ownership table identify responsibilities. |
| Explicit investment in design | Pass | Owner-requested design audit with before/after checks. |

Performance risks identified in source, requiring measurement: dependency edge
matching scans every locked package and reparses version constraints per
candidate; shortest-path BFS clones entire annotated path prefixes into queued
branches. Inspection also constructs the same evidence summary twice. The first
two are algorithmic waste, independent of HTTP/cache latency. Shared result
accounting currently allocates a Vec per ID and retains arbitrary duplicate
payloads although only cardinality matters.

Other reviewed areas are sound enough to retain: content/source identity types,
private proof enums, per-run OnceLock index sharing, bounded HTTP/archive reads,
panic-safe metrics, and separate display/policy semantics. Do not replace these
with generic effect frameworks, extra crates or flags shared across commands.

## Alternatives and selected implementation

1. Cosmetic helper extraction only: low risk, but preserves duplicated protocol
   ownership and the measured algorithmic costs.
2. Focused private redesign (selected by implementer within delegated scope):
   one observation collection with Single/Duplicate states, name-indexed lock
   matching, and predecessor-based BFS. Preserve request identities, exact
   diagnostics and output ordering. Add no runtime dependency or persistent cache.
3. Broad graph/interner/public API rewrite: may reduce more copying but changes
   too many boundaries before measurements establish the need. Defer.

The observation owner stores at most one payload per distinct request ID; any
second result makes that slot permanently duplicate. Taking an expected request
returns missing/duplicate errors or asks Evidence to rebind proof to that request.
The command-specific evaluation still owns policy/evidence requirements and
foreign-result wording. Keep `inspection::AcquisitionResult` available by re-export.

Resolution builds an ephemeral normalized-name index once per document. Source,
PEP 440 version equality and specifier checks still select exactly one identity;
ambiguity and invalid input remain errors. Keep exception scopes and manifest
origin-hint fallback unchanged.

BFS retains its entry/edge order and visited state `(package, activated extras)`.
Queue parent indices instead of complete path prefixes. Reconstruct annotated
steps only after selecting the first shortest path; preserve markers, extras,
cycles and equal-length tie behavior. No global memoization or mutable cache is
needed. A single-parent arena remains private to resolution.

## Verification and limits

Use synthetic fixtures only. Measure the same opt-in release benchmarks before
and after, recording toolchain, fixture size and five samples; assert semantic
results, not timing thresholds. Separate lock construction and path lookup.
Network performance is not inferred from these measurements.

Add independent parametrized and generated coverage through public APIs for
result cardinality/order and graph ambiguity/shortest paths. Use AAA and one
scenario/property per function. Preserve existing Lean adapters and fuzz targets;
run the relevant generated corpus and bounded smoke checks. Five self-review
passes apply to design, each implementation stage and final verification.

Conditions for reconsideration: public entry points must change, evidence checks
weaken, observed semantics differ, or added indexing/arena complexity outweighs
the measured gain. Update the existing sans-io decision with current ownership;
user guidance does not change when CLI behavior remains identical.


## Assessment refinement during implementation

A concrete Debug disclosure was found in SourceId/ArtifactLocator and their
nested PackageId/ArtifactRequest representations. A small regression fix applies
the existing display redactor inside those type owners; public identity/key
algorithms, URL fetching and JSON are unchanged. Fixed/generated tests failed
before and passed after; the minimized synthetic witness is retained.

An independent review confirmed the private-ledger and graph boundaries. Its
minor source-kind comment issue was corrected: Registry/Url locations normalize
HTTP(S) URLs, while Git/Path/other kinds retain opaque nonempty data. Runtime
normalization was deliberately unchanged. No new per-fork parse cache was added;
validation remains lazy and candidate-ordered.
