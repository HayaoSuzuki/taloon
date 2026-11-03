# Complete text variant review

Base: `172ab2ea52b4eb29620aba5c3e0b75b8ca4d83bd`; local checks on macOS arm64.
The [design](plan.md) records pinned sources, alternatives and scope.

## Apache stage: five self-review passes

1. Semantics: checked SPDX XML's optional appendix and replaceable copyright
   field; sections 1–9 and END remain exact and mandatory in this subset.
2. Whole-input boundary: no prefix/substring success; partial appendices,
   appended conditions, edited terms and conditions in the holder field fail.
3. Evidence/I/O: only pure matching changes. One raw copyright line must match
   the complete normalized field; no metadata, proof or cache format changes.
4. Tests: five positive/property cases failed first; all 16 new cases passed
   after connection. Full workspace: 898 passed, 18 ignored, zero failed.
5. Design/source accuracy: optional content is a license-specific rule behind
   the existing result interface. No XML interpreter or fuzzy wildcard engine
   is introduced. Tests and design use pinned SPDX facts and synthetic holders.

## Attribution stage: five self-review passes

1. Semantics: fixed MIT title only; shared copyright grammar permits exactly two
   two-word person names and an optional final parenthesized contact origin.
   Name-shaped attributions are a lexical scope rule, not semantic authorship proof.
2. Whole-input boundary: full existing body remains exact; added sentences,
   extra lines, comma conditions and non-contact URL forms are independently tested.
3. Evidence/I/O: URL parsing is local validation, never acquisition. A discovered
   normalization hazard erased dot paths/backslashes; three new cases failed
   before a literal-origin roundtrip check fixed it. No paths/parameters remain.
4. Tests: six new positive/property cases failed before implementation; final
   text tests: 113 passed. Workspace: 921 passed, 18 ignored, zero failed.
   Generated names/years and hidden-path cases generalize the named regressions.
5. Design/source accuracy: one shared attribution owner avoids different holder
   rules in each license branch. Existing corporate/role cases still pass; fixed
   reserved terms prevent common condition phrases from becoming personal names.
