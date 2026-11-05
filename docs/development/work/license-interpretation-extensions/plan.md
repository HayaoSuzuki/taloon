# License interpretation extensions

## Design and scope

Implement Issues #29, #30 and #31 on one branch, with independently reviewable
commits. Existing evidence, permission policy and acquisition boundaries remain
unchanged. All fixtures are synthetic; references are public released SPDX data.
No dependency or runtime service is added.

### Precise names (#29)

Use the [released SPDX v3.29.0 identifier/name data](https://github.com/spdx/license-list-data/blob/v3.29.0/json/licenses.json)
to review this finite set of new spellings:

| Spellings | ID |
| --- | --- |
| Mozilla Public License v1.0 | MPL-1.0 |
| Mozilla Public License v1.1 | MPL-1.1 |
| Mozilla Public License v2.0; Mozilla Public License Version 2.0 | MPL-2.0 |
| ZPL 1.1; Zope Public License Version 1.1 | ZPL-1.1 |
| ZPL 2.0; Zope Public License Version 2.0 | ZPL-2.0 |
| ZPL 2.1; Zope Public License Version 2.1 | ZPL-2.1 |
| BSL 1.0; Boost Software License Version 1.0 | BSL-1.0 |
| Eclipse Public License Version 2.0 | EPL-2.0 |
| CDDL v1.0 | CDDL-1.0 |
| CDDL v1.1 | CDDL-1.1 |

These are explicit version spellings of the named licenses, not fuzzy matches.
Reject versionless families, PSFL, Dual License, unknown versions, style licenses
and GNU version-only shorthand. Existing whole-value, unique-identity matching
owns the table. Reconsider individual mappings if published identities conflict.

### Explicit legacy expressions (#30)

Strict SPDX parsing remains first. Legacy-only fallback uses the grammar:

```text
expression := conjunction (OR conjunction)*
conjunction := primary (AND primary)*
primary := '(' expression ')' | leaf [WITH strict-exception-id]
leaf := longest complete unique catalog-name/alias | strict-single-license
```

Operators are uppercase and have token boundaries. Explicit structural groups
and alias names containing parentheses are distinct: try complete names first.
Use the existing unique-identity resolver for names and the existing strict SPDX
parser for each atomic unit and final expression. Preserve explicit grouping,
AND precedence over OR and strict WITH syntax; do not infer operators or aliases
for exception names. Entire input must be consumed. Limits: 16 KiB input, 64
atomic leaves and 32 nested structural groups for this fallback. Existing valid
strict expressions retain their existing rules.

A private parser owns grammar/limits; the legacy owner owns naming data.
Building normalized syntax with bounded string appends reuses the existing typed
expression/permission model. A new public AST or global string replacement would
duplicate semantics or hide unconsumed conditions and is rejected.

### Single-text variants (#31)

Apply the [normative matching guidelines](https://spdx.github.io/spdx-spec/v3.0.1/annexes/license-matching-guidelines-and-templates/)
and released [BSD-2-Clause](https://github.com/spdx/license-list-data/blob/v3.29.0/json/details/BSD-2-Clause.json)
and [BSD-3-Clause](https://github.com/spdx/license-list-data/blob/v3.29.0/json/details/BSD-3-Clause.json)
templates to a finite first batch: equivalent leading copyright markers
`Copyright (c)`, `Copyright (C)`, `(c)`, `(C)` and `©` in short-template notices;
number/star/dash/absent markers on the recognized BSD clauses; and fixed generic
author/owner/holder role variants in BSD third clauses and disclaimers.
Do not wildcard arbitrary organization names, delete general punctuation,
ignore foreign prefix/suffix material or combine multiple bodies. Mandatory
terms stay exact apart from these explicit substitutions. Dates/holder validation,
raw line boundaries and existing size/count limits remain authoritative.

## Implementation plan

**Goal:** Extend precise legacy names, explicit expressions and bounded BSD text
recognition without weakening proof or permission semantics.
**Architecture:** Private name table/parser and existing whole-text matcher.
**Tech stack:** Rust 1.99.1, rstest, proptest; pinned SPDX data; no new dependencies.
**Spec:** The three linked GitHub Issues and design above.

1. #29: add named public evidence cases and observe failures; extend only the
   reviewed fixed alias table; test cache replay and policy; commit separately.
2. #30: add failing whole-expression, precedence, grouping, WITH and bounds
   cases; implement the bounded parser; compare generated Boolean formula
   outcomes with an independent evaluator under allow and deny; commit separately.
3. #31: add failing complete-text variants and independent altered-term cases;
   implement fixed transformations only; verify archive/sidecar/cache paths;
   commit separately.
4. Update current user and contract owners alongside each change. Run fmt,
   strict Clippy, full workspace tests, warnings-denied Rustdoc, shuffled tests,
   metadata fuzzing and documentation checks. Obtain independent final review
   and open one focused PR closing the three Issues.

## Review focus

- Names containing parentheses or operator-like words must not be split wrongly.
- Partial names, additional terms and unbalanced/empty groups must fail closed.
- Field precedence/cardinality and strict License-Expression stay unchanged.
- Reinterpreting raw observations must not change hashes, proof or acquisition.
- Text transformations must preserve conditions, paragraph order and all input.

Each stage receives at least five self-review passes: scope/ownership, semantics,
whole-input/bounds, independent tests and evidence/source accuracy. Use AAA,
named cases and meaningful properties; split contexts instead of branching test
bodies. Existing formal decision models remain unchanged; parser correctness is
tested behaviorally rather than claimed formally proved.
