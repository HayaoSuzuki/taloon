# Legacy license interpretation implementation plan

**Goal:** Interpret precise names and complete known texts in the legacy License
field without additional acquisition or relaxed SPDX-expression validation.

**Authorization:** The owner requested the recommended available improvements and
previously selected implementation through PR delivery. Continue the open bundled
license PR. Use only synthetic fixtures and public SPDX texts in maintained work.

**Architecture:** A private legacy declaration owner keeps strict expression
parsing first, then exact official full-name/curated alias lookup, then existing
complete-text recognition. License-Expression remains strict and authoritative.
Repeated values remain unknown. No archive fallback is added for present fields;
existing raw metadata caches re-evaluate under the updated interpreter.

## Scope and review risks

Official names come from the pinned spdx list, with ambiguous duplicate names
rejected. Explicit aliases identify a version/clause count; unqualified BSD,
Apache Software License, free prose and invented AND/OR combinations stay unknown.
Case-insensitive name lookup is confined to whole legacy names, never expressions
or arbitrary substrings. Preserve custom expression behavior and policy checks.

Extend attribution syntax only for bounded, structured standard notices (multiple
copyright lines, year lists, corporate punctuation and explicitly known author
role/connector words). Keep original line/sentence boundaries and complete body
comparison. Arbitrary extra conditions and unrecognized preambles stay unknown;
no fuzzy score or legal completeness guarantee is added.

## Tasks

- [x] Names: named public regressions RED, private precise-name owner, GREEN;
  generated whole-value/case properties and five review passes; commit.
- [x] Text: public complete-text/precedence regressions RED, connect existing
  matcher only for legacy fields, GREEN; cache/sidecar/HTTP tests; commit.
- [x] Attribution: synthetic structured variants RED, bounded grammar extension,
  GREEN; adversarial punctuation/added-term regressions; commit.
- [x] Update existing user/OKF owners and staged evidence; strict full/shuffled
  tests, Clippy/rustdoc/docs, bounded fuzz and one final independent review.

Delivery: update the open PR after local verification; hosted results remain on it.

Five self-review passes per stage: requirements and field priority; real production
and cache connection; bounded types/storage/performance; independent fixed and
generated adversarial cases; and documentation/reproduction/trust limits.

Initial design assessment: 8/10. The field-specific interpretation decision is
currently mixed with generic SPDX parsing in Evidence. A private legacy owner
hides the expanded formats behind one existing-result interface and keeps I/O
unchanged. Final assessment follows integration and review.
