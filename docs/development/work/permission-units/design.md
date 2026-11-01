# Validated permission units

Base: `1cc77aa737829e56306e57182dadcb9bdd719bea`, 2026-11-01. The owner requested
another design/performance/type review after discussing validated permission
units instead of a duplicate SPDX license Enum. PR #10 is still open; extend it.

## Assessment and scope

The inspected boundary retains 8/8 diagnostics (10/10): policy owns permissions
and validation, its interface remains deep and design intent is documented.
A worthwhile internal improvement is making atomic permission values distinct
from arbitrary names/reasons/strings in helper signatures and sets. Catalogue
management remains in the locked spdx crate; no list duplication is needed.

The installed spdx 0.13.4 LicenseItem PartialEq uses a PartialOrd comparison that
ignores or_later, while Ord includes that flag. The strict lexer rejects GNU postfix-plus spellings, while it accepts
non-GNU postfix plus; accepted deprecated identifiers remain distinct. Therefore directly deriving permission equality/order from
LicenseReq could violate the application's existing canonical-text semantics.
This is a source observation at the locked version, not a claim about all releases.
Application regression tests should require correct unit semantics, rather than
locking in that dependency implementation detail.

## Selected design and alternatives

1. Keep bare strings: existing parser protects construction, but signatures and
   collections do not express atomic permission invariants.
2. Wrap LicenseReq and use its derived equality: rejected because canonical
   supported spellings and the observed trait behavior need explicit treatment.
3. Validated PermissionUnit with canonical spelling and a private Standard/Custom
   classification: selected. Construction uses the existing SPDX expression
   parser and requires one atomic license/WITH term. Eq/Ord/Borrow agree on the
   canonical spelling, preserving GNU distinctions, whitespace normalization and lexical
   report order. Classification comes from the parsed LicenseItem enum, not a
   repeated substring test.

Expose parse/as_str/is_custom/Display and ordinary value traits. Keep fields
private and omit unchecked deserialization. Policy sets use this type; global
custom references still require a hash-scoped exception. Existing public
LicenseExpression::units and JSON remain their cached canonical string views;
no duplicate typed expression tree or second catalogue is introduced.

## Execution plan and checks

- Add fixed constructor/equality/order/scope regressions and generated canonical
  set consistency checks, following AAA and one scenario/property per function.
  Observe the missing new API, then implement it and preserve existing policy
  characterization/oracle behavior.
- Measure the same existing policy workloads before/after. This is primarily
  a type/invariant improvement; no speedup is promised. Keep per-artifact lookup
  allocation behavior and report ordering unchanged.
- Review design, implementation and final verification five times each; run
  stable/shuffled Rust, Clippy, Lean correspondence, relevant bounded fuzz and
  documentation/Python checks. Obtain a fresh read-only whole-PR review, commit,
  update PR #10 and observe final CI. Do not merge or publish.

No new runtime dependency, evidence rule, caching behavior, licence database
version or user-facing syntax is introduced. Reconsider if typed expression
consumers genuinely need a new API or measurements justify a structured lookup
key; do not copy the SPDX catalogue or change unit equality implicitly.


## Supported spellings

The first exploratory tests assumed every deprecated catalogue spelling was
accepted, including GPL/LGPL postfix-plus entries. The existing strict lexer
rejects those even with deprecated identifiers enabled; supported non-GNU plus
and deprecated GNU identifiers remain distinct cases. Tests now draw positive
catalogue inputs from the existing expression parser's supported subset and
retain GPL/LGPL-plus as fixed negative cases. Runtime parsing was not broadened.
No erroneous positive generator seed is claimed as a production regression.
