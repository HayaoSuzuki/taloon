# Interpretation extension reviews

Base: `dd56e43d3fb3b5d063de122f9252e60746b839f8`; local macOS arm64.
The [design and plan](plan.md) records finite scope and public released sources.
All maintained fixtures use synthetic metadata and pinned canonical texts.

## #29: five self-review passes

1. Scope/ownership: 15 version-explicit spellings in the existing fixed table;
   no public interface, dependency, configuration or acquisition change.
2. Semantics: reviewed the corresponding MPL/ZPL/BSL/EPL/CDDL identities in
   released SPDX v3.29.0. Versionless families, PSFL, GNU version-only names,
   invented versions and added conditions remain excluded.
3. Whole-input boundaries: shared unique-identity lookup remains authoritative;
   strict License-Expression, duplicate fields and precedence are unchanged.
4. Tests: 18 positive/policy/property cases and six online/offline cache cases
   failed before the aliases. Final legacy suite: 149 passed; cache suite: 32.
   Full workspace: 1,081 passed, 18 ignored, zero failed (baseline 1,051).
5. Evidence/source accuracy: literal expected IDs and generated whole-value
   properties are independent of the production table. Raw hashes/proof remain
   unchanged and cache replay requires no requests, bytes or extraction.

Scoped design: 10/10 across the eight software-design-philosophy diagnostics;
coherent ownership, deep unchanged interface, hidden naming data, documented
boundaries, replaceable implementation and explicit design/review effort all
hold. No diagnostic failed. This is not a score for the entire repository.

## #30: five self-review passes

1. Ownership: a private parser owns grammar/limits; the parent owns names.
   Existing LicenseExpression validation owns strict atom/exception semantics
   and final output. No replacement AST or policy evaluator is introduced.
2. Semantics: preserve AND precedence, explicit groups, parentheses/operator
   words inside names, and strict WITH identifiers. All input must be consumed;
   no operator, exception name or additional condition is inferred or discarded.
3. Bounds/ambiguity: 16 KiB, 64 leaves and 32 structural group levels. Longest
   complete names are checked by the existing unique-ID lookup; synthetic
   collisions and same-ID duplicates have independent cases. Raw slices use
   checked UTF-8 boundaries; structural grouping is bounded before recursion.
4. Tests: 15 initial expression/bound/property cases failed before connection.
   Generated independent Boolean formula evaluation checks allow and deny;
   literal precedence traps check unparenthesized forms. Metadata priority,
   duplicate fields and strict Expression rejection remain independently tested.
5. Evidence/source accuracy: online/offline compound cache cases retain original
   declarations, hashes and index-associated proof with zero extra requests.
   Existing malformed-expression diagnostics are retained. SPDX expression
   grammar is a source for operators, not permission to broaden Python's
   supported references or strict License-Expression field.

The expression suite passes 42 independent cases/properties, including maximum
and excess bounds. Workspace: 1,132 passed, 18 ignored, zero failed. Strict
Clippy passes. An independent read-only parser review found no actionable
issues. Existing formal permission models are unchanged; grammar correctness
is behavioral evidence rather than a new formal proof.

## #31: five self-review passes

1. Scope/sources: reviewed the BSD-2-Clause and BSD-3-Clause template fields
   in SPDX v3.29.0 and the normative matching guidelines linked in the plan.
   Adopt only listed generic role/marker substitutions and copyright symbols;
   arbitrary organization substitutions and upstream wildcards remain excluded.
2. Ownership/performance: a private BSD body normalizer hides its finite table.
   Canonical strings normalize once. A borrowed result avoids allocation when
   no substitution is present. Acquisition and public interfaces stay unchanged.
3. Whole-input/bounds: all remaining body words and clause order compare exactly.
   Shared date/holder/count checks and raw lines still constrain attribution.
   Review caught and removed a potential doubled-symbol acceptance; a separate
   regression rejects it. No missing terms, extra conditions or multiple bodies
   are discarded. Existing mandatory-term and prefix/suffix negatives remain.
4. Independent tests: 45 new positive/property cases failed before production
   changes. The final 64 matcher cases/properties cover symbol, role and marker
   combinations, negation, altered/deleted terms, unknown roles/markers, extra
   conditions, reversed clauses and two bodies. Test bodies use AAA and avoid
   scenario branches. Literal IDs and rejection expectations are independent
   of the normalizer table.
5. Evidence/policy: three representative variants exercise wheel and sdist,
   legacy sidecars, cache record roundtrips, archive-cache replay and online/
   offline raw metadata caches. Hashes/proof stay fixed, with no requests,
   bytes or extraction on replay. Complete BSD role text also follows ordinary
   allow/deny evaluation. No new dependency, runtime service or URL fetch.

Scoped design: 10/10 on the eight software-design-philosophy diagnostics; no
failed diagnostic. Fixed interpretation knowledge stays private behind existing
interfaces, with explicit boundaries and independent review. This score applies
to this change, not every module in the repository.

## Final verification

An independent read-only review of all three extensions found no actionable
issues and ran 522 focused tests successfully. The existing formal decision
models and their adapters are unchanged; these interpretation extensions are
covered by behavioral/property/fuzz checks, not a claimed new formal proof.

The final full workspace passed 1,216 tests (18 ignored), including both final
BSD policy cases.
Strict Clippy, warnings-denied Rustdoc, documentation validation and strict
MkDocs build pass on macOS arm64. A pinned nightly shuffled the relevant suites
with seed 290031: 842 passed, 11 ignored (before the two final BSD policy cases). Metadata fuzzing completed 1,000 runs
at a 32,768-byte bound against a temporary copy of the maintained corpus,
including new synthetic alias/expression/BSD seeds; no crash was observed.
