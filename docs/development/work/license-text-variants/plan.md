# Complete license text variants

## Design

Extend the existing pure text matcher after the versioned-alias change. The
public hoimin lock has 60 unknown artifacts: precise-name additions alone do
not resolve its retained body observations. No acquisition or evidence-rule
change is needed to reinterpret those observations.

Keep whole-text recognition and existing field priority/cardinality. Select
per-license transformations rather than a fuzzy detector or general markup
normalizer; both alternatives could hide changed obligations. The current
contract's Apache canonical-only rule and five-template limit must change
together with user guidance. Multiple complete licenses and prose explaining
different licenses remain outside this change.

The pinned SPDX XML reference is
[`8791eda86b237dae9a956a55b207740de9000bd3`](https://github.com/spdx/license-list-XML/tree/8791eda86b237dae9a956a55b207740de9000bd3/src):

- [Apache-2.0](https://github.com/spdx/license-list-XML/blob/8791eda86b237dae9a956a55b207740de9000bd3/src/Apache-2.0.xml)
  marks the application appendix optional and its copyright field replaceable.
  Accept canonical sections 1–9 through END OF TERMS AND CONDITIONS, with either
  no appendix or the entire appendix. In an otherwise exact appendix, permit a
  single-line structured copyright year/holder (including paired bracket form).
  Do not accept a truncated appendix, changed terms or appended material.
- MIT retains the complete existing body. Permit the exact alternate title
  `The MIT License (MIT)`, exactly two comma-separated two-word personal names and
  a final parenthesized HTTP(S) origin as contact attribution. Origins must have
  no credentials, query, fragment or non-root path. Holder validation and raw
  line boundaries remain authoritative; no arbitrary prose is removed. The shared
  copyright validator also applies those attribution forms to existing BSD/ISC
  and Apache notices; only the title extension is MIT-specific. This is a bounded
  lexical grammar, not proof of a holder's identity or licensing completeness.
- MPL-2.0 uses the full pinned SPDX text, including both exhibits. Accept the
  single historical `http://mozilla.org/MPL/2.0/.` spelling in the same position
  as canonical HTTPS. No exhibit, secondary-license condition or other wording
  may be removed. The matcher never follows that URL.
- BSD-3-Clause permits the exact third-clause phrase `name of the copyright
  holders, nor those of its contributors` in place of the canonical singular
  holder/name wording, with existing numbered/star markers. This is an explicitly
  reviewed whole-body variant, not a general pronoun or holder-clause wildcard.
  The [SPDX BSD-3-Clause reference](https://github.com/spdx/license-list-XML/blob/8791eda86b237dae9a956a55b207740de9000bd3/src/BSD-3-Clause.xml)
  documents third-clause variation; its broad regex is not imported and does
  not itself list this exact spelling. The fixed wording is separately reviewed
  against the complete observed public text and canonical obligations.

Keep the existing result type and private interpreter boundary. No dependency,
cache schema, permission rule or metadata eligibility change. Canonical texts
remain supplied by the pinned spdx crate. Reconsider only individually reviewed
variants when the reference changes; do not import XML wildcard rules wholesale.

## Execution plan

1. Apache: write failing named canonical-body/appendix tests and mutation
   properties; implement the bounded matcher and commit this target separately.
2. MIT: write failing positive attribution cases and independent negative
   conditions; implement narrow title/name/contact support and commit separately.
3. MPL: write canonical/historical-URL tests and exhibit/altered-term negatives;
   add the exact complete template and commit separately.
4. Exercise real wheel/sdist, legacy-field, policy and persisted-cache paths.
   Run all checks and public-lock comparison without network acquisition. Update
   the owning contracts/user guide and retain verification evidence.
5. Add and verify the fixed BSD third-clause variant in a separate commit.

Use AAA, named rstest cases and proptest; each test invocation covers one
scenario or property. Keep test functions free of fixture-selection `if` branches:
split distinct contexts into separate functions or supply explicit named cases;
generated variants belong in strategies. Five self-review passes per stage cover semantics,
whole-input boundaries, proof/I/O, independent tests and source/document accuracy.
Final independent review covers the whole stacked diff. Existing formal proofs
do not prove license identities; no decision algorithm changes in this work.
