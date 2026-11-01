# Policy follow-up staged review

Base: `6460fe942a5c2703238a3d3c5ee8c03382e20496`, 2026-11-01, macOS arm64.
The current owner request delegates further assessment/improvement; this extends
open PR #10. Public CLI/JSON/method signatures and proof requirements are fixed.

## Design: five passes

1. Rechecked module depth and information hiding. Policy remains the sole owner
   of config and permission matching; no new public abstraction or flag is needed.
2. Selected PackageId as the index key rather than a formatted name, retaining
   normalized name, PEP 440 version and source equality in one validated type.
3. Retained original file ordinals/order separately from package-sorted storage.
   All input entries must still validate before any query, including unmatched ones.
4. Reviewed alternatives: borrowed permission predicates and grouped lookup are
   sufficient. Decision caches and new scope Enums add unnecessary contracts.
5. Defined benchmark boundaries and independent numeric selector expectations.
   Evidence rebinding is unchanged; potential savings there do not justify
   altering trust assumptions in this policy-only stage.

## Implementation: five passes

1. Matching uses a private map lookup followed by exact optional hash scope.
   Ordinals are captured while parsing in original order; repeated normalized
   identities append without dropping duplicate/additive exception entries.
2. Permission membership is the logical union of immutable global and matching
   scoped sets. No per-artifact global set clone or union construction is needed.
   Override IDs/reasons/order and each unit decision remain observable unchanged.
3. Unused checks collect configured usage once. Global and clarification pending
   sets support early completion in common policies; no unrelated package state
   is retained. Package/hash observations cover hashless local requests too.
4. Original exception warning order is restored by numeric ordinals before final
   report sorting. All OR terms count as observed. Incomplete-run suppression
   remains owned by Inspection and is not moved into matching.
5. Public tests compare decisions, unit permissions and override ordinals against
   integer identity/hash selectors. Fixed normalized version/source cases and
   proof-strength cases complement generated warning checks. Initial nine cases
   passed on the old implementation before this behavior-preserving refactor;
   five normalization/clarification/hashless-content cases were added during review. No new
   functional defect or behavioral red/green fix is claimed.

## Assessment and evidence limits

All eight diagnostic rows remain satisfied in this inspected boundary (10/10):
clear responsibility, deep interface, replaceable internals, contract comments,
design review, hidden matching knowledge, understandable boundary and deliberate
improvement. No failed row remains. This checklist score does not measure
algorithm costs or imply whole-project perfection. The current work improves
internal costs without manufacturing an architectural defect.

Existing Lean permission/completeness proofs and strict adapters remain relevant;
no new theorem about the exception index is claimed. Integer-selector proptests
provide separate public implementation evidence for scoped permissions and
warnings. Timings apply only to the two synthetic workloads in the measurement
record; remaining common-case/setup/end-to-end costs are not quantified there.

## Final verification: five passes

1. Compared exact package identity, hashless observed hashes, optional scopes,
   custom proof requirements and original exception ordinals against old source
   and fixed/generated public tests. No evidence-validation rule changed.
2. Checked usage edge cases: no observations, no configured entries, all-used
   globals, observed/unobserved clarifications, unrelated package records and
   additive/duplicate exceptions. Early exits depend only on fulfilled usage.
3. Checked cost bounds and reported scope: matching touches one package group;
   warnings avoid repeated full observation scans and preserve output order.
   Timings exclude parsing/setup and do not measure warning costs independently.
4. Ran full stable/shuffled suites, focused property tests, Clippy/rustdoc,
   existing Lean build/corpus freshness/strict adapter, and bounded policy plus
   inspection fuzz (1,000 runs, max_len 4096). No theorem, heartbeat bound,
   dependency graph or runtime requirement changed.
5. Obtained a fresh independent whole-PR review: 73 targeted tests passed for
   the reviewer; no actionable findings. Confirmed all eight diagnostics pass
   within scope. Rechecked staged content, source hashes and documentation links
   before committing and updating the existing PR.

Lean build completed in 0.600s with the retained 20s deadline. The unchanged
model establishes its original permission/completeness theorems; its finite
strict adapter still matches. It does not prove the new private index correct.
The new numeric-selector properties are independent implementation tests.

No maintained user workflow changes, so no user-guide update is needed. The
owning sans-io/verification concepts describe the new private representation and
its evidence. Unchanged Python tooling requires no mutation campaign for this
Rust-only stage. GitHub evidence is recorded for the actual final PR SHA.

Final local evidence: 171 regular Rust tests passed in stable, generated-shuffle
and seed-12345 runs; Python 134 passed. Clippy, rustdoc and all seven prek hooks
passed. The final 10,000-artifact process goal check passed all five warm runs
(0.541981–0.555705s), with zero network and re-extractions.
