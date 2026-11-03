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
