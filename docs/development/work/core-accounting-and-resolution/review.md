# Staged design and implementation review

Base `9a8e2a1f26176ad663fa2fed74f83e88260d04b2`; local macOS arm64, 2026-11-01.
Evidence concerns the named working-tree stage, not a future commit or CI run.

## Design: five passes

1. Reviewed pure-core and acquisition contracts against current modules: preserve
   the sans-io boundary and private proof validation; do not add I/O or flags to
   core consumers. Existing source/hash/format enums are appropriate.
2. Traced missing, duplicate and foreign results through check and describe.
   Multiplicity and rebinding are shared protocol knowledge; command-specific
   permission/strength and foreign-result wording remain outside the ledger.
3. Traced normalized names, forks, source equality and invalid selector errors.
   Index candidates by name only, retaining existing per-candidate validation
   order so absent matches and malformed selectors do not change diagnostics.
4. Traced path queue order, cycles, activated extras and marker provenance.
   Parent links must retain the original edge and predecessor; visiting by
   package alone would incorrectly collapse extra activation states.
5. Reviewed alternatives, test independence and measurement limits. Keep the
   public API and no new dependencies. Synthetic release timings precede changes;
   use no wall-clock assertion and do not infer network or private-project gains.

Ruling: use characterization tests for this behavior-preserving refactor. They
pass on the old implementation before changing ownership/algorithms. No new
semantic bug or behavioral red/green fix is claimed. Compilation mistakes in test
construction are diagnosed and are not regression evidence.

## Observation accounting: five passes

1. Centralized multiplicity and request rebinding; the original public
   `inspection::AcquisitionResult` path remains a re-export. No extra public
   module or command mode was introduced.
2. Made Duplicate an absorbing state for any second observation. A failed result
   cannot overwrite a success or mask a duplicate; unclaimed IDs remain ordered.
3. Dropped duplicate payloads rather than retaining their entire vectors.
   Clippy identified a 352-byte variant; boxing Single keeps tree slots small
   and reserves one exact payload rather than a growing Vec. The caller's input
   Vec still exists; this is not a claim of bounded total caller memory.
4. Kept Evidence as the binding owner and left policy/strength checks intact.
   Rebinding remains as strict as before. Construct evidence summaries once
   instead of copying declarations twice in inspection.
5. Shared public behavior tests cover both consumers, duplicate-after-failure,
   arbitrary counts and shuffled receipt order. Existing identity/weak-evidence,
   description, inspection and Lean correspondence tests remain applicable.

## Resolution: five passes

1. Name indexing retains canonical package order within each name; source and
   PEP 440 checks keep their prior order and errors. All external targets remain,
   including unreachable records and unsupported artifact sources.
2. Borrowed candidate IDs remain owned by immutable records throughout parsing.
   No interner, stable-address assumption or self-referential owner is introduced.
3. Parent indices always refer to an earlier arena slot. Edges borrow the
   immutable graph/entry, and final path reconstruction is iterative. Work no
   longer copies every prefix into every queued branch.
4. Retained visited state includes activated extras, and seen is marked on pop,
   preserving queue ties. Fixed cases cover cyclic extras/marker provenance;
   exhaustive simple-path DFS is an independent generated shortest-path oracle.
5. Re-ran matching/path properties and existing resolution/description tests.
   Five-sample release timings show the expected improvement; report complete
   samples and an intermediate noisy series rather than only the best run.

## Model debug and contract documentation: five passes

1. Confirmed concrete debug disclosure with four fixed cases and a generated
   test, all failing before the fix. Retained the minimized synthetic regression.
2. Source and remote-locator Debug use the existing redaction owner. Nested
   PackageId/ArtifactRequest derive their safe representation from those types;
   no caller has to remember a new redaction step.
3. Source fingerprints and request keys still use their original canonical data,
   not the changed Debug implementations. CLI/JSON schema and fetch identity do
   not change. Local paths retain local-path semantics.
4. Checked generated URL secrets and diagnostic context: credentials/query values
   disappear while the host remains useful. This is a domain-rendering contract,
   not a claim to sanitize arbitrary strings in user-owned data structures.
5. Replaced generic model-constructor comments with actual normalization,
   validation, identity and proof contracts. Requirements do not change request
   identity. The proof/evaluation boundary remains explicit.

Ruling: the Debug defect discovered during the requested type/encapsulation
assessment is included as a small regression fix. It changes diagnostic Debug
rendering, not the CLI or JSON. Unlike the pure refactors, it has observed
behavioral red/green evidence.

## Formal correspondence and limits

Claim: complete checks require one sufficient observation for each expected ID
and no foreign IDs. The existing Lean model already owns this claim; a new
cardinality theorem or rewritten corpus would add no new implementation premise.
The refactor is checked against the unchanged model and generated expectations.

| Premise / observation | Lean representation | Production / observation | Mode |
| --- | --- | --- | --- |
| Missing/single/duplicate expected ID | Observation list and per-ID filter length in complete | AcquisitionResult vectors passed through Inspection::evaluate; report exit/status | strict |
| Sidecar/archive and required strength | Observation.archive and required | Public evidence construction with scoped policy; public report | strict |
| Foreign ID | Observation.request outside expected list | Foreign public request result; public diagnostic/exit | strict |
| Enum layout and dropped duplicate buffers | Not represented in the existing model | Private ResultLedger layout / ownership | model-only |
| BFS allocation costs, Debug secret rendering | Outside completeness model | Independent public API tests and release benchmarks | model-only |

`lake build TaloonOracle generate_corpus generate_index generate_selection`
completed in 0.441s with a retained 20s external deadline. Corpus freshness and
existing sensitivity witnesses passed: 48 permission, 3 expiry and 1,296
inspection cases. Existing Rust strict adapters passed. No bound or heartbeat
limit was raised, no corpus was hand-edited, and no new theorem about Rust
correctness, memory or network behavior is claimed. New independent proptests
exercise higher multiplicities and both check/describe, separately from Lean.

Uniqueness/coverage and proof/exit precedence remain exercised by the existing
corpus's duplicate, foreign, weak and missing cases. Persisted transactionality
is outside this in-memory refactor; no new durable state machine is introduced.
No specification ambiguity or unresolved semantic mismatch was found.

## Documentation disagreement recorded before correction

The existing sans-io decision retained initial-design statements that no process
performance result existed and publishing was deferred, although the baseline
and distribution decision already recorded implemented behavior. These historical
sentences conflicted with their own current linked evidence. Replace them with
links to the current owners; no product behavior or publishing configuration is
changed by this documentation correction.

## Final verification: five passes

1. Compared final scope with the owner request and original public paths. The
   pure core remains free of filesystem/network/clock access; CLI, JSON and
   dependency requirements remain unchanged. Debug redaction is the one observed
   diagnostic rendering fix, not an identity normalization change.
2. Rechecked safety cases: missing/duplicate/foreign/weak/misbound results remain
   incomplete, and failed duplicates cannot mask multiplicity. Public behavioral
   tests exercise both consumers; fixed/generated paths preserve cycles, extras,
   annotation provenance and deterministic ties.
3. Rechecked performance evidence: identical synthetic fixtures, all five samples
   and intermediate series retained; no wall-clock gate or inferred network gain.
   The full 10,000-artifact CLI workload met the existing goal in all five warm
   runs. Remaining unmeasured costs are follow-ups, not implicit speed promises.
4. Ran stable and shuffled workspace checks, Clippy/rustdoc, Lean freshness and
   adapters, bounded uv_lock/inspection fuzz smoke (1,000 runs, max_len 4096),
   and documentation/Python quality hooks. Keep ignored opt-in benchmarks and
   TLS fixture checks explicitly separate from ordinary suite counts.
5. Obtained a fresh read-only whole-change review. The reviewer independently
   passed 61 targeted Rust tests and found no Critical/Important issue. Its sole
   minor contract-comment issue was corrected and re-reviewed. Final source
   scope, attribution, source hashes and staged diff were inspected before commit.

## Outcome and remaining opportunities

Within the inspected module/type boundaries, all eight diagnostics now pass:
responsibilities, interface depth, replaceability, accurate abstraction comments,
design review, information hiding, understandable boundaries and deliberate
improvement. The rubric score moved from 6/8 (7.5/10) to 8/8 (10/10); the
independent reviewer confirmed the latter after the comment correction. This
checklist score does not assert whole-project perfection or maximal performance.

The interface leak is resolved by one private owner; comment failures are
resolved with kind-specific identity and proof contracts. No diagnostic row
remains failed in the inspected scope. Policy scans, same-name-fork selector
parsing, immutable-evidence revalidation and report allocation remain measurement
candidates. Additional parse caches must preserve lazy validation/error order;
removing record/binding checks requires separate evidence-owner analysis.

Local final regular suite: 157 Rust tests on macOS, including the new fixed and
128-case generated checks; Python baseline: 134 cases. New opt-in release cases
are excluded from ordinary CI execution. Existing Lean proofs are unchanged;
model proofs, finite correspondence and property/fuzz evidence remain distinct.
GitHub CI is observed for the eventual PR SHA and recorded in its description.
