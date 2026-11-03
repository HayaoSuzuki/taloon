# Versioned legacy license aliases

## Design

Implement [Issue #26](https://github.com/HayaoSuzuki/taloon/issues/26) by
extending the private fixed alias table in `src/evidence/legacy.rs`.
The reviewed candidates are `MPL 1.1`, `Mozilla Public License Version 1.1`,
`MPL 2.0` and `Mozilla Public License, Version 2.0`. Their identities are
MPL-1.1 and MPL-2.0 respectively, as recorded in the
[pinned CycloneDX table](https://github.com/CycloneDX/cyclonedx-core-java/blob/8e05e866d8073fc18b3b3c70a9316fd5a5999f81/src/main/resources/license-mapping.json).
This is a reviewed subset, not an import of the upstream table.

The selected approach keeps one owner for naming knowledge and requires no new
API, configuration, dependency, transport or cache schema. Strict expressions
remain first; whole-value names trim outer whitespace and ignore ASCII case.
Identity collisions, extra conditions, repeated fields and ambiguous versions
remain unresolved. Evidence strength, hashes and original declarations stay
unchanged, and policy evaluates the resolved expression normally.

Importing the whole upstream table would guess versions for generic names.
Adding a separate resolver or online service would duplicate ownership and add
I/O without improving these exact mappings. Both alternatives are rejected.
Body recognition, versionless names and manual clarification tooling are out of
scope. No compatibility layer is needed.

## Implementation plan

**Goal:** Recognize reviewed MPL aliases without weakening evidence or policy.
**Architecture:** Existing private interpreter and raw-observation cache.
**Tech stack:** Rust, rstest, proptest; no dependency changes.
**Spec:** Issue #26 and the design above.

1. Add named metadata/evidence, field-priority, ambiguity and allow/deny
   regressions in `tests/legacy_licenses.rs`. Generalize case/outer whitespace
   and added-term protection with independent literal expected identities.
2. Extend existing HTTP fixtures in `src/acquisition/http_tests.rs` to cover
   the sidecar fast path and online/offline cache replay for MPL aliases.
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
- Raw cache replay must preserve original hashes and index-associated proof.
- Resolved MPL must be rejected when denied, and allowed only by normal policy.

Each stage receives five self-review passes: scope/ownership, exact semantics,
evidence/I/O, independent test coverage, and documentation/source accuracy.
See the [completed review and verification record](review.md).
