# Typed acquisition staged review

Base `df637758fa6bb3349981641c68f63bf60f0950eb`, macOS arm64, 2026-11-01.
The owner's latest instruction removes taloon backward compatibility as a design
constraint. This supersedes the compatibility deferrals in earlier assessments.

## Design: five passes

1. Traced grouping and JSON parsing through acquisition to strict core rebinding.
   Key ordering/parser permissiveness are implementation choices; target coverage
   and proof sufficiency remain required semantic checks.
2. Compared formatted keys with domain values. Pinned and Unpinned variants keep
   content identity distinct from request identity without raw digest accessors.
3. Inspected current PyPA Simple API: bool/hash metadata and legacy alias are valid
   external representations, not taloon compatibility wrappers to delete.
4. Chose typed owned serde records rather than a streaming visitor or borrowed
   document framework. Unknown extension fields are ignored without Value trees.
5. Defined tighter input behavior before code: modern presence wins over legacy;
   null metadata and duplicate/malformed known fields invalidate the index and
   retain requested-archive fallback. Empty scope creates no worker partition.

## Worker implementation: five passes

1. Empty-scope test observed actual RED: one old partition versus zero expected.
   All nonempty bounds passed before implementation; all grouping cases passed
   afterward, including the existing progress/panic acquisition regressions.
2. Inspected WorkKey identity domains. Pinned hash+format and unpinned RequestId
   cannot alias; same content still shares a worker without dropping observations.
3. Checked ArtifactFormat ordering: new trait supplies deterministic keys only,
   not proof ranking. Old Debug/hex lexical order is deliberately not reproduced.
4. Traced zero-worker scope through channel shutdown, joins, summaries and the
   initial progress callback. No terminal result or counter is required/dropped.
5. Reviewed the independent integer-label property for coverage, uniqueness,
   worker bound and group affinity across 256 cases. It does not assert old
   partition order, cryptographic collision freedom or optimal load balance.

## JSON implementation: five passes

1. Eleven malformed/duplicate-shape cases failed against the old Value parser
   (accepted documents), then passed with typed decoding. This is intentional
   stricter behavior under the no-compatibility instruction.
2. Typed records retain only consumed protocol fields. Unknown extensions are
   skipped; no compatibility decoder, generic AST or custom streaming framework
   remains. The file vector/Strings still require response-proportional memory.
3. Advertisement distinguishes boolean hints from hash dictionaries. Explicit
   presence decoding prevents null from becoming absence and selecting legacy.
   Modern false/true/invalid-hash/other-algorithm cases never fall through.
4. Exact URL normalization/scope, archive digest and duplicate artifact binding
   stay in the existing shared owner. Hashless/sidecar/archive proof invariants
   remain in core validation; a typed bool never becomes verified metadata.
5. Existing 216-case index model adapter passed through JSON/HTML plus HTTP
   acquisition regressions (101 focused cases after shape corrections). Integer-code
   advertisement properties independently cross modern/legacy hints/extensions.
   This is a consumed-field decoder, not a validator for every Simple API field.

## Final verification: five passes

1. Final stable suite passed 305 regular Rust tests after the object-only fix. Added three HTTP fallback
   cases were run against the old parser (all failed the all-archive expectation)
   and then the typed parser (all passed). One malformed entry invalidates the
   snapshot, but both locked artifacts are verified and each warning retained.
2. Generated-seed and seed-12345 nightly suites each passed 305 tests after the object-only fix. Grouping
   and advertisement properties use 256 integer-oracle cases each; fixed cases
   retain format/hashless/empty boundaries and exact malformed-field failures.
3. Existing Lean build and all three corpus freshness checks passed under the
   retained 20-second deadline. Strict adapters still cover their original model
   premises. Metadata/inspection fuzz each completed 1,000 runs at max_len 4096;
   these are core regression campaigns, not direct fuzzing of binary-private
   serde/grouping. Existing malformed index properties separately exercise JSON.
4. Python 134 tests, Ruff formatting/ALL lint, strict ty, Clippy/rustdoc and docs/
   OKF/strict site checks passed. No Python implementation changed, so Python
   mutation testing is inapplicable. User evidence guidance now describes stricter
   JSON fallback and the worker bound. No new package/version was added.
5. Reconcile final independent whole-PR findings, source hashes, privacy and local
   hooks before commit; observe CI for the final SHA in the PR. The final release CLI goal
   passed all five warm samples at 0.543–0.581 seconds; complete observations
   and limits are recorded in the measurement owner. Earlier
   measurements/results retain their revision/environment rather than becoming
   guarantees for a future tree.

Scoped software-design diagnostics pass 8/8 (10/10): coherent responsibilities,
deep interfaces, replaceable internals, intent comments, design review, hidden
protocol decisions, documented boundaries and explicit design investment.
No failed row is identified; this is neither exhaustive correctness nor an exact
measurement of design-time percentage. The [design](design.md) supersedes prior
compatibility deferrals and [measurements](performance.md) record complete samples.

## Independent review finding and one fix pass

Fresh `typed_acquisition_final_review` inspected the whole branch from `9a8e2a1`
through `df63775` plus this working tree, without editing or private examples.
It identified one Important issue: serde's derived records accept positional
arrays, contradicting the selected malformed-shape guarantee. No Critical/Minor
finding was reported. Abstraction-comment diagnostic row 4 therefore failed for
that intermediate decoder (7/8, 8.75/10); correction required enforcing object
input rather than weakening the documented guarantee.

Added six correctly sized positional-array negatives at document/meta/file/
archive-hash/modern/legacy boundaries and a HTTP case carrying an actual valid
metadata digest in an array. All seven failed against the typed prototype, then
passed after a private Object<T> map-only visitor enforced the wire invariant at
all record boundaries. The complete suite passed 305 tests afterward. This is
one implementation fix pass with tests, not an unobserved second independent
approval. Final self-assessment returns to 8/8, with row 4 corrected and no known
finding deferred.

The reviewer did not certify superseded ordering/permissive behavior, full API
schema validation, private workloads, production/RSS/whole-CLI speed, pending
executor/CI results or final OKF reconciliation. Those remain separate scoped
executor evidence. The object wrapper is a local serde map adapter, not a
streaming JSON framework or a compatibility decoder.
