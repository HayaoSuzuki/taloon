# Versioned legacy license aliases

## Design

Implement [Issue #26](https://github.com/HayaoSuzuki/taloon/issues/26) by
extending the private fixed alias table in `src/evidence/legacy.rs`.
The reviewed additions cover 39 whole-value names across MIT, Apache-2.0,
BSD-2-Clause/BSD-3-Clause, MPL-1.1/MPL-2.0, CDDL-1.0/CDDL-1.1,
EPL-1.0/EPL-2.0, CC0-1.0, GPL-2.0-or-later, LGPL-2.1-or-later and
LGPL-3.0-or-later. Each added spelling is checked individually against the
[pinned CycloneDX table](https://github.com/CycloneDX/cyclonedx-core-java/blob/8e05e866d8073fc18b3b3c70a9316fd5a5999f81/src/main/resources/license-mapping.json).
This is a reviewed subset, not an import of the upstream table.
Examples include `Apache License (v2.0)`, `BSD 2 Clause`, `MPL 2.0`,
`CDDL 1.1`, `EPL 2.0` and `LGPL, v2.1 or later`. GNU aliases must explicitly
state later-version permission; a plain version number does not establish
`-only` versus `-or-later`. Short ambiguous identifiers, generic family names,
style licenses and exception abbreviations are excluded. The fixed table in
the interpreter is the owner of the exact accepted spellings.

The selected approach keeps one owner for naming knowledge and requires no new
API, configuration, dependency, transport or cache schema. Strict expressions
remain first; whole-value names trim outer whitespace and ignore ASCII case.
Identity collisions, extra conditions, repeated fields and ambiguous versions
remain unresolved. Evidence strength, hashes and original declarations stay
unchanged, and policy evaluates the resolved expression normally.

Importing the whole upstream table would guess versions for generic names.
Adding a separate resolver or online service would duplicate ownership and add
I/O without improving these exact mappings. Both alternatives are rejected.
Body recognition, version-ambiguous family names and manual clarification tooling are out of
scope. No compatibility layer is needed.

## Implementation plan

**Goal:** Recognize reviewed precise aliases without weakening evidence or policy.
**Architecture:** Existing private interpreter and raw-observation cache.
**Tech stack:** Rust, rstest, proptest; no dependency changes.
**Spec:** Issue #26 and the design above.

1. Add named metadata/evidence, field-priority, ambiguity and allow/deny
   regressions in `tests/legacy_licenses.rs`. Generalize case/outer whitespace
   and added-term protection with independent literal expected identities.
2. Extend existing HTTP fixtures in `src/acquisition/http_tests.rs` to cover
   the sidecar fast path and online/offline cache replay across alias families.
   Watch new positive cases fail before adding production mappings.
3. Add the reviewed aliases in `src/evidence/legacy.rs`, preserving unique-name
   resolution. Run targeted tests and the full workspace suite.
4. Update `docs/user/evidence.md` and the owning inspection/design contracts;
   retain reference rationale and staged self-review evidence here.
5. Run fmt, Clippy, Rust tests, documentation validation/site build and review
   the final diff before committing and opening the focused PR.

## Review focus

- Invalid present License-Expression must not use aliases from either field.
- Case and outer whitespace must not erase interior punctuation or conditions.
- MPL without a version and unknown MPL versions must stay unknown.
- GNU aliases must retain explicit later-version permission; version-only
  shorthand does not imply `-only`. Generic CDDL/EPL names stay unknown.
- Raw cache replay must preserve original hashes and index-associated proof.
- Resolved aliases must be rejected when denied, and allowed only by normal policy.

Each stage receives five self-review passes: scope/ownership, exact semantics,
evidence/I/O, independent test coverage, and documentation/source accuracy.
See the [completed review and verification record](review.md).
