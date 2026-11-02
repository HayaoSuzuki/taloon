# Constrained legacy supplementation implementation plan

**Goal:** Complete narrowly incomplete legacy declarations using verified bundled
text while preserving explicit declarations, provenance and bounded acquisition.

**Authorization:** The owner selected recommended defaults and requested reasonable
supplementation through the existing PR. Use synthetic fixtures and public SPDX
texts for all maintained evidence. Existing branch/PR remains open.

## Design

A private typed FileConstraint (AnyRecognized/Bsd/Apache) owns whole-value eligibility
and exact family acceptance. Empty declaration sets keep existing file discovery.
Only one legacy value equal to UNKNOWN, BSD/BSD License, or Apache/Apache License/
Apache Software License/Apache 2 is eligible (ASCII case/edge whitespace ignored).
Do not add references, NONE/unlicensed markers, generic prose, fuzzy guessing or
new license templates. BSD accepts recognized BSD-2-Clause/BSD-3-Clause; Apache
accepts recognized Apache-2.0. Unsupported variants stay unknown.

Strict License-Expression always wins; invalid/repeated expressions and repeated
legacy fields cannot be supplemented. Valid legacy interpretation remains first.
Extra conditions or conflicting/multiple file expressions are not discarded.
Require the existing full-archive proof and bounded exact file reader. Inspect
fresh archive metadata before applying supplementation; cached sidecar hints do
not authorize overriding archive declarations. Existing checksummed observations
keep their schema and raw fields; completed unknown scans remain cached.

Evidence exposes one eligibility decision reused by acquisition and interpretation.
No new transport fallback layer, persistent verdict or configuration flag. Positive
and negative scans upgrade directly from existing cache only when necessary;
offline does not fetch. Add optional LicenseOrigin in the evidence summary with
four variants (LicenseExpression, LegacyDeclaration, BundledFiles,
LegacySupplement). This describes evidence before policy clarifications.
Text rendering identifies successful supplementation on both CLI commands;
policy decisions/clarifications retain their own reporting.

## Tasks

- [x] Core: public RED regressions for placeholder/family/precedence/conflicts,
  typed eligibility, origin and immutable cache replay; GREEN and five reviews.
- [x] Acquisition: independent HTTP cases for bounded upgrade, no extra work for
  declarations, stale sidecar/archive disagreement, negative warm/offline reuse.
- [x] Output: JSON origin and text supplementation labels through real commands.
- [x] Docs/OKF and exact-source review: update current owners, full/shuffled tests,
  Clippy/rustdoc/docs, bounded fuzz, one independent review and PR update.

Each stage gets five passes: required acceptance/rejection domains; production
call/provenance connection; types/storage/bounds and performance; independent
named/generated adversarial tests; and documented sources/reproduction limits.
Tests use AAA and one scenario/property per independently executed case.

Initial design assessment: 8/10. The former absent-only decision is duplicated
across interpretation and the acquisition query. A single evidence-owned typed
constraint resolves the extension without exposing header or family rules to I/O.
