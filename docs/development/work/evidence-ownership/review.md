# Immutable payload staged review

Base `e7badc3ee22f5b5756ee0c1d0099ff44ab7461a0`, macOS arm64, 2026-11-01.
Review scope: evidence payload ownership, acquisition/cache callers and the
existing core/graph/policy changes in open PR #10.

## Design: five passes

1. Traced record construction, rebind, ledger evaluation and cache publication.
   Metadata-sized copies were representation overhead, not a caller requirement.
2. Compared consuming/borrowing APIs with private sharing. Chose the smaller
   interface; the prototype and its missing-method failures were discarded.
3. Compared Arc<str> and Arc<RecordData>. Sized payload sharing moves the String
   on deserialization and keeps caller code unchanged.
4. Checked the trust boundary: immutable shared provenance must not replace
   checksum, schema, header, hash, format, source or package revalidation.
5. Kept request identity and parsed interpretation outside shared RecordData.
   Existing proof Enum already expresses the necessary states; no new Enum,
   configuration switch or public sharing abstraction is justified.

## Implementation: five passes

1. Reviewed the entire product diff: serde rc, one Arc field, one allocation and
   an invariant comment. No caller or validation branch changes.
2. Traced hashless request checks before from_record and sidecar request/source
   checks after digest verification. Existing error precedence stays intact.
3. Checked serde's inner-object representation against existing corruption,
   expanded metadata-limit, persistence and concurrent publication tests. Cache
   Envelope and schema remain unchanged; unknown-field rejection remains enabled.
4. Reviewed named proof-binding cases and the generated cloning invariant. Fixed
   the generated future-minor example to 2.7 (2.5 is already recognized) and
   asserted warning cardinality independently. New tests follow AAA, one scenario
   or property per function; parameter cases have separate test identities.
5. Inspected complete before/after timing samples. check and describe move in
   opposite directions, so no general speed or peak-memory claim is made. Keeping
   expensive validation is intentional, not an unfinished optimization.

## Final verification: five passes

1. Full stable suite: 216 regular Rust tests passed. The generated property was
   rerun after tightening its future-version assertion (18 focused cases passed).
2. Seed-12345 and generated-seed shuffle runs each passed 216 tests. The
   generated-seed run includes the final future-version assertion.
3. Lean proof/generator build and all three corpus freshness checks passed under
   the retained 20-second deadline. Existing strict Rust adapters passed in the
   full suite. No new theorem about Arc, serde or allocation is claimed.
4. Metadata and inspection fuzz targets each completed 1,000 runs at max_len
   4096. Python 134 tests, Ruff formatting/ALL lint, strict ty, docs/OKF and strict
   user-site checks passed. Python mutation testing is inapplicable: no Python
   implementation changed.
5. Fresh whole-PR review identified one allocation-description correction:
   serde uses a transient Box before Arc on deserialization. Corrected the
   design/measurement wording; no product change was needed. Final source hashes,
   local hooks and CI are checked before handoff. No user command or output changed; no user guide
   update is required. The owning OKF design/verification concepts are updated.

The inspected boundaries satisfy eight diagnostics (10/10): coherent ownership,
deep API, replaceable representation, invariant comments, design review,
information hiding, documented boundaries and explicit design investment. No
failed row remains in this scope. This is not exhaustive project correctness.

The [design](design.md) records the API-prototype ruling and its allocation/refcount
tradeoff. The [measurement record](performance.md) separates avoided copies from
observed elapsed times. No unresolved correctness finding is being deferred.

Fresh reviewer `immutable_evidence_final_review` (separate context, 2026-11-01)
reviewed the whole PR from `9a8e2a1` through `e7badc3` plus this working tree.
It found no Critical/Important issue and one documentation allocation omission,
now corrected. It confirmed all eight scoped design diagnostics. It did not run
the suite, inspect private examples, establish unbounded parser correctness,
measure exact design-time percentage, peak RSS or optimized allocator counts,
or observe executor/CI campaigns. Those limits remain explicit; executor results
above supply separate finite execution evidence. No reviewer finding is deferred.
