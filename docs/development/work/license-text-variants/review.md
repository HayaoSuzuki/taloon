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

## MPL stage: five self-review passes

1. Semantics: complete pinned MPL-2.0 text, including both exhibits; only the
   historical HTTP locator in its canonical position differs from HTTPS.
2. Whole-input boundary: removed exhibits, changed grant/secondary-license
   wording, different locators and generated appended conditions remain unknown.
3. Evidence/I/O: pure string comparison, no URL resolution, new observations or
   permission inference. Existing metadata spans and proof remain unchanged.
4. Tests: both complete-text cases failed first; all eight new cases passed
   after connection. Text tests: 121 passed. Workspace: 929 passed, 18 ignored.
5. Design/source accuracy: array length derives from the supported template set;
   unknown future templates no longer implicitly use ISC's body marker. No full
   SPDX enum or generic template language is introduced for six supported texts.

## BSD stage: five self-review passes

1. Semantics: compare the complete observed third clause with canonical BSD-3;
   only the fixed plural/pronoun phrase changes. All obligations remain present.
2. Whole-input boundary: numbered/star cases fail first, then pass; negated use,
   changed permission requirements and generated inserted conditions are rejected.
3. Evidence/I/O: reuse the existing whole-body BSD variant path. No prefix-only
   success, similarity score, extra retrieval or raw-evidence mutation.
4. Tests: 127 text tests pass, including six new BSD cases. Public archive,
   metadata and online/offline cache adapters also cover the BSD variant.
5. Design/source accuracy: the source distinguishes explicit SPDX alternatives
   from this separately reviewed exact wording. No generalized regex is copied.
   All newly added unit test bodies avoid fixture-selection `if` branches, as
   requested; separate Apache/MPL contexts and generator strategies preserve
   one test case/property per function invocation.

## Final integration: five self-review passes

1. Public boundary: synthetic typed fixtures exercise both wheel and sdist raw
   record serialization/replay, legacy sidecars and normal allow/deny evaluation.
2. Evidence: archive proof stays verified; sidecar proof stays index-associated.
   Metadata/content hashes and original-field handling are checked independently.
3. Acquisition: real fixture transports and disk caches verify no requests,
   received bytes or extraction on replay, including online and offline legacy
   paths. The implementation changes no eligibility or transport decisions.
4. Verification: 974 workspace tests pass, 18 remain intentionally ignored.
   Shuffle seed 2604 passes 602 selected library/binary/legacy/bundled tests,
   with 11 binary fixtures intentionally ignored.
   Strict Clippy, fmt and warnings-denied Rustdoc pass. Metadata fuzzing completes
   1,000 runs with a 32,768-byte bound so complete folded Apache/MPL seeds fit.
   CI uses that same metadata bound; the other five targets retain 4,096 bytes.
   Workflow lint/security checks, documentation checker and strict site build pass.
5. Documentation/design: update existing user, procedure and contract owners.
   A separate read-only reviewer found no actionable Critical/Important/Minor
   issues. No verdict cache, new runtime I/O, dependency or public API is added.

The scoped design scores 10/10 on software-design-philosophy's eight diagnostics:
coherent module purpose, deep unchanged interface, replaceable implementation,
abstraction comments, explicit design review, hidden matching rules, documented
boundaries and deliberate design work. No diagnostic failed for this change.
This is not an assessment of the entire repository or a proof of license identity.
Existing formal-correspondence tests pass; the decision crate is unchanged.

## Public-lock validation

The public hoimin checkout at `a11ce718ed4f79c677373faed2fae6c2bdaf6b89` supplies
84 packages and 921 locked artifacts. The lock is unmodified relative to that
revision, SHA-256 `7ad78d6e68c655115654c170d6b4c317d038520d5844abb2ca2100fec7171de8`.
Its local pyproject differs from upstream, SHA-256
`d40795f84252aa6d74b30a78ece535f6489550c4e05b91ccae08ecb5bec47d33`;
both executions use those same bytes and deny policy. Do not attribute these
local policy results to an unmodified upstream configuration.

Compare the alias branch at `172ab2ea52b4eb29620aba5c3e0b75b8ca4d83bd` with
this branch's complete matcher using `check --offline --format json`.
Both reports contain identical request IDs and all 921 cached observations.
Neither run performs requests, downloads or archive extractions. This validates
retained evidence re-interpretation, not a live refresh of package indexes.

| Outcome | Before body variants | After body variants |
| --- | ---: | ---: |
| Compliant artifacts | 861 | 888 |
| Incomplete artifacts | 60 | 33 |
| Packages with incomplete artifacts | 11 | 3 |

Resolved artifacts: colorama 2 (BSD-3-Clause), ghp-import 2 (Apache-2.0),
markdown-it-py 2 (MIT), mergedeep 2 (MIT), pathspec 2 (MPL-2.0),
rfc3161-client 13 (Apache-2.0), rfc8785 2 (Apache-2.0) and
sigstore-rekor-types 2 (Apache-2.0). The 27-artifact reduction is 45% of the
previous incomplete set. No formerly recognized artifact regressed.

Remaining: libcst 29 (multiple-license prose), mdurl 2 (multiple complete bodies
with contextual text) and python-dateutil 2 (`Dual License` without a recognized
expression). No combined expression is inferred from those inputs. Those require
separate reviewed evidence/design rather than a more permissive substring rule.

A single final debug CLI run took about 0.80 seconds, including report rendering
but excluding uv startup. This is a warm-cache observation, not a controlled
performance comparison or an initial-download measurement.

## MIT preamble extension: five self-review passes

This addendum is based on `4932769c69ca18f3e4cdb72b667d6e40f7c79219`.
Fixtures and rationale below use released SPDX data and invented attributions.

1. Semantics: compared the pinned mandatory MIT body with released v3.29.0;
   verified the normative distinction between replaceable notices/titles and
   additional substantive text. Do not implement the copyright wildcard.
2. Whole-input boundaries: mandatory body equality precedes the MIT fallback.
   Every raw prefix line must be a known title, structured notice or bounded
   final contact; changed bodies and unknown/additional material stay unknown.
3. Types and limits: an explicit Notices/Contacts phase rejects notices after
   contacts and contacts without a notice. Bounds are 5,000 prefix bytes, 32
   notices, 32 dated groups and two domains. Other license branches are unchanged.
4. Tests: initial ten new positive/property cases failed before connection;
   two further named range/role cases failed before their narrow extensions.
   All 45 synthetic preamble cases/properties now pass, including negative and
   maximum/excess boundaries. New test bodies have no fixture-selection branches.
5. Design and proof: private MIT ownership avoids expanding shared heuristics.
   Original metadata/file bytes, hashes, priority, evidence strength and policy
   evaluation remain unchanged. The independent reviewer found no actionable
   issues; attribution grammar remains lexical rather than authorship proof.

The scoped design retains 10/10 across the eight design diagnostics: meaningful
private ownership and state, unchanged deep interface, hidden license-specific
rules, documented boundaries, implementation replaceability and explicit review
and design work. No diagnostic failed for this extension.
