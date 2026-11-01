# Policy evaluation follow-up

Base inspected: `6460fe942a5c2703238a3d3c5ee8c03382e20496`, 2026-11-01.
The owner requested another software-design-philosophy assessment and worthwhile
improvements. PR #10 remains open, so this follow-up extends that PR rather than
creating a dependent PR or presuming a merge.

The inspected boundary still satisfies the eight diagnostics (10/10 within that
scope): policy owns matching, SPDX permission and config semantics behind a small
pure API; no failed boundary diagnostic is introduced. The checklist does not
measure internal algorithm costs. Concrete remaining waste is copying every
global permission for each artifact, scanning every exception on each lookup,
and rescanning all observations for each unused-entry warning.

## Approaches and selection

- Keep the current vector and only remove allow-set copying: small change, but
  leaves repeated unrelated-package scans and warning cross products.
- Index exceptions by validated PackageId and aggregate observed usage once:
  selected within the owner's delegated internal design scope. Preserve original
  file ordinals in entries, group order within a package and final diagnostics.
- Cache decisions/persist results: requires additional keys and invalidation
  semantics; unnecessary when eliminating repeated work suffices. Defer.

Policy stores a map from PackageId to ordered scoped exceptions. Entries retain
original one-based ordinals and optional content hashes; the map owns package
identity once. A private matcher hides lookup/hash scope. Evaluation consults the
borrowed global set plus matching exceptions without cloning their union. Unused
warnings use one observation pass to collect units, package/content presence and
content hashes, then inspect configuration entries once.

Existing domain types and proof Enums remain appropriate. Option<ContentHash>
already expresses optional content scope; adding a parallel Enum would add no
invariant. Evidence rebinding still checks records and bindings; its repeated
work is deferred rather than changing trust assumptions without measurement.

## Acceptance and plan

1. Add synthetic opt-in benchmarks and independent public policy/inspection
   characterization tests; record before samples at the base implementation.
2. Preserve expired/invalid unmatched entry rejection, exact name/version/source
   and hash matching, original exception IDs/order, additive permissions, custom
   license proof requirements, unused warnings and incomplete-run suppression.
   Implement map/matcher, borrowed permission lookup and one-pass usage aggregation.
3. Re-measure identical fixtures, run fixed/generated/formal/fuzz and normal
   quality checks, perform five self-reviews per stage and independent review.
   Commit/push to PR #10 and observe CI. No merge or publication.

Keep CLI/JSON/public API and runtime dependencies unchanged. Use AAA, one
scenario/property per test and independent parametrized/generated expectations.
Performance measurements are synthetic evidence, with no CI timing threshold or
claim about network latency. Preserve the existing owning OKF concepts.
