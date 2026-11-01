# Describe implementation reviews

## Stage 1: pure selection and observations

Based on `67ddd08`, local macOS arm64 verification on 2026-11-01 passed
109 normal Rust tests, formatting, strict Clippy/rustdoc and documentation/OKF
validation. The public Description tests first failed on the missing module;
all nine new behavioral tests now pass, including a 256-case independent
Floyd-Warshall/permutation property and 50 strict Lean-generated selections.
The model/generator and all three corpus freshness checks passed within the
existing 20-second command deadline. No peak-memory measurement was taken.

Five self-review passes:

1. Scope: all normalized-name identities seed the closure, workspace matches
   select the exact full external inventory, and workspace/external collisions
   retain roots. All extra/marker branches are included; external traversal
   excludes development edges and terminates through cycles/workspace routing.
2. Evidence: every selected artifact has a terminal result; duplicate, missing,
   foreign and substituted evidence is incomplete. Rebinding/interpretation stay
   with Evidence, with default IndexAssociated strength and original metadata.
3. Reporting: exact expression strings are grouped per artifact, not joined as
   SPDX alternatives. Unknown/invalid declarations and unsupported targets stay
   visible. Valid LicenseRef is descriptive, not a permission grant. Ordering
   does not depend on package/edge/result input permutations.
4. Ownership/bounds: resolution hides graph details; Description owns scope and
   completeness. No Policy object, pyproject/date/I/O or new dependency is used.
   The graph inventory is flat; relationships retain cycles without tree growth.
   Existing check result structure/evaluation and cache representation are intact.
5. Correspondence: canonical graph premises, finite four-node expansion and
   the public request-inventory mapping are explicit. Model proofs establish
   roots/filter/all-mode properties, not general Rust queue or HTTP correctness.
   Sensitivity covers extra omission, dev inclusion, narrowed all and lost roots.

The AcquisitionResult value adds a crate-private consuming accessor; external
callers still construct it through existing constructors. Safe graph display
identities carry source fingerprints/redacted locations rather than raw uv data.
CLI behavior and whole-branch review remain stage 2 work.

## Stage 2: CLI and shared acquisition

Based on `ac88a8f`, local macOS arm64 verification passed 115 normal Rust tests,
95 Python tooling tests, formatting, strict Clippy/rustdoc, docs/OKF checks,
strict user-site build, 45 workflow ShellCheck blocks and pinned zizmor.
Initial process tests reproduced the missing command and wrong startup envelope;
valid-name normalization in input errors also had a failing regression before
its fix. All describe process tests now pass.

The changed inspection and uv_lock fuzz targets each completed 1,000 executions
with a 4,096-byte configured maximum, with no crash. Inspection reported 435 MiB
RSS and local symbolizer warnings; uv_lock reported 86 MiB. These bounded runs
are not exhaustive parser/graph proofs. Generated non-crash corpus growth was
kept in the temporary task workspace, not added as unexplained regressions.

Five self-review passes:

1. Scope/I/O: process and controlled-transport tests prove selection precedes
   acquisition. An unrelated unavailable artifact does not enter selected
   requests; project/member mode still selects the full external inventory.
   Default/explicit discovery, empty projects and all matching versions/sources
   are pinned by process tests.
2. Output/security: describe has its own complete/incomplete JSON envelope,
   including startup errors; no permission status/exit is fabricated. Source
   credentials/query values are redacted, terminal markers are escaped, and
   original differing/unknown observations remain visible. Dependency edges are
   indexed once for flat text rendering instead of rescanned per package.
3. Sharing: check and describe use one runtime acquisition/progress helper,
   authentication, persistent evidence schema and proof checks. Both commands
   reuse observations; offline performs no HTTP. Description reads no pyproject,
   including malformed policy, and never applies clarifications or permission.
4. Errors/invariants: no matching package fails before acquisition; normalized
   valid query names are retained in input errors. Missing, duplicate, foreign,
   unknown and unsupported outcomes cannot become complete. Check/cache
   behavior and their existing process tests remain intact.
5. Ownership/docs: selection remains private to resolution; Description exposes
   requests and typed observations. Public graph output types contain safe
   identities rather than raw uv records. README/user guidance and owning
   contracts describe implemented behavior; source snapshots distinguish the
   approved design, implemented tests and model limits.

Stage rulings: the existing uv_lock fuzz target also exercises Description
because structured inspection fuzz uses a fixed graph; this requires both
changed targets' smoke checks. Inline feature checkout remains the existing
session choice, with no separate filesystem isolation.

Hosted CI results are tracked in the PR after observation.

## Final independent review and association fix

On 2026-11-01, fresh reviewer gpt-6-astra (`describe_final_review`) inspected
`4c481d4..fd133be` read-only and independently ran the description/CLI test
binaries. It reported one Important finding, no Critical or Minor findings,
and no behavior declined as out of scope. Live network/hosted checks were not
independently executed. The initial verdict was with fixes.

The reviewer reproduced archive evidence acquired for one hashless local wheel
being accepted for a different wheel of the same package through arbitrary
AcquisitionResult IDs. The author pinned that reproduction: expected exit 2,
observed exit 0. Evidence now retains its originating request ID in memory and
owns a rebind operation. Description, Inspection and cache publication use it;
hashless observations cannot transfer to another request. Digest-pinned archive
content reuse remains valid, and sidecars retain their existing exact bindings.
A positive content-reuse regression and the existing cache/process tests pass.

The source fix is `12f7b7b`. The full suite passed 117/117 normal Rust tests on
macOS after correction, with strict formatting/Clippy/rustdoc. The author fixed
the one Important finding in one RED→GREEN pass; no second independent review
was requested. There are no deferred Minor findings.

Implementation rulings, in order:

1. Continue inline on the session's dedicated feature checkout: branch isolation
   without separate filesystem isolation from other local changes.
2. Extend uv_lock fuzz as well as structured inspection fuzz: fixed-graph result
   fuzzing alone would not exercise selection from mutated graph inputs. Both
   changed targets therefore need bounded smoke verification.
3. Keep request provenance transient, restoring trusted persisted observations
   only after the runtime freshly hashes local content. This preserves existing
   content-keyed v1 cache records without migration; that trusted fresh-hash
   boundary remains a premise for hashless cache restoration.

## Synthetic warm process regression

The existing release-profile check benchmark ran against source `12f7b7b` on
macOS arm64: 1,000 synthetic packages, 10,000 distinct local archives and a fresh
evidence cache. Its first local population took 46.905769 s;
five warm processes took 0.700427, 0.543940, 0.558798, 0.566864, 0.634141 s,
all below the existing two-second criterion. Every warm process reported 10,000
cache hits, zero HTTP/bytes, zero index parses and zero archive extraction.
This is a local cached-check regression, not a cold-network or describe latency
guarantee. Measured executable SHA-256:
`90a2fe7a73784d504342992dd03ce4c6f4e2a8608aaf31d3cbe1f66601f12c88`.

The first harness invocation lacked TALOON_BENCH_BINARY and stopped before any
product execution. With that prerequisite supplied, an earlier `fd133be` run
also passed. The final run above was repeated because the subsequent shared
association fix changed evidence validation; it is the final source evidence.
After the fix, the structured inspection/description fuzz campaign was repeated
for 1,000 executions with max_len 4,096 and passed (reported RSS 434Mb), as did
the actual loopback TLS rejection/timeout regression. No second code review or
extra unrelated fuzz campaigns were run.

## Final design assessment

Seven of eight software-design-philosophy diagnostics are supported, about
8.75/10: named boundaries, deep request/result interfaces, hidden graph/protocol
choices, interface intent comments, design review, decision ownership and readable
module boundaries. The design-effort time fraction was not measured; record it
in future work to substantiate the remaining diagnostic. No speculative cache
framework or policy mode was introduced. Evidence rebinding owns the association
rule for all consumers, with the persisted-record trust premise stated explicitly.
