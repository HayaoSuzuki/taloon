---
type: Contract
title: License inspection specification
description: Approved design for uv.lock coverage, Python license policy, evidence, and command behavior.
status: accepted
sources:
- id: interpretation-extensions
  resource: ../work/license-interpretation-extensions/review.md
  working_tree: untracked
  sha256: eef6f787f0ab8acfc6e2594e5ab63a06d9727f52eaed4f94fc041c11fb2cdcdb
- id: complete-text-variants
  resource: ../work/license-text-variants/review.md
  working_tree: modified
  sha256: 20089d6e72ec7bf7c15296e998444180445ef88e73b26aec1cef2f8532a838a0
- id: versioned-license-aliases
  resource: ../work/versioned-license-aliases/plan.md
  working_tree: modified
  sha256: 4374edbfc28cf8a840abd91368c34327403b24e0068d309b6f37518aa9751ee8
- id: deny-policy
  resource: ../work/deny-policy/plan.md
  working_tree: untracked
  sha256: 63afb2f7f520a264139fbb52aa03b1a3b6ec8decc1f803a4c1de804035d27193
- id: actionable-diagnostics-review
  resource: ../work/actionable-license-diagnostics/review.md
  working_tree: untracked
  sha256: f56ab86f925d5d2149ce47181fcafa15a434ecfd1fe18698fda05d0294e86473
- id: constrained-supplement-review
  resource: ../work/constrained-license-supplementation/review.md
  working_tree: untracked
  sha256: d1ff5e2be3367bada1c6a635b996e5b1ee9ac6967831400aeceebc4677d7b252
- id: legacy-license-review
  resource: ../work/legacy-license-interpretation/review.md
  working_tree: untracked
  sha256: dca68bb0997c79a7bf25ba5244afade94b4dc42a9e74b4b12e8976ba03982090
- id: bundled-license-review
  resource: ../work/bundled-license-evidence/review.md
  working_tree: untracked
  sha256: ed82b8bf06d4d89c63c8b074f3e1b400dfc3422bf7c7d6458a73ddc146c1b359
- id: declaration-reporting
  resource: ../work/evidence-and-invariants/issue15.md
  working_tree: modified
  sha256: 6460f0f6c7d69f9c97a6c594ba62c3750abf01b01d9438e03fea52ae1e230d98
- id: typed-acquisition-review
  resource: ../work/typed-acquisition/review.md
  working_tree: modified
  sha256: 00daddabad915d492d1583acaff483a254849c01dccbf3ddb26de235cdc30605
- id: describe-review
  resource: ../work/describe-licenses/implementation-review.md
  working_tree: modified
  sha256: da2011db34cf05805f633424b8c977d54a204f09fed2befa5a664eafa3d2bc34
- id: describe-design
  resource: ../work/describe-licenses/design.md
  working_tree: modified
  sha256: 739306eecb05253fec189602923098bd44db6416c2288ca57ef126b12f4b2458
- id: cache-clear
  resource: ../work/cache-clear/design.md
  working_tree: modified
  sha256: bca6e015bde6bbe3c1a2c076a0f3c6c7f666599e21d0c2b9283a69afb2fb37e4
- id: acquisition-measurements
  resource: ../work/index-acquisition/measurements.md
  working_tree: modified
  sha256: 6aaeeade6f9e0f1a495c5328501465ef1aa571660070ecd2957f3b71f4a2f67a
- id: interactive-progress
  resource: ../work/interactive-progress/review.md
  working_tree: untracked
  sha256: c205dcc8ebdabb6a0e22ef34daae56ebef6a342fa4ccdb8208fa96f068c93b8d
- id: owner
  resource: ../work/initial-design/specification-inputs.md
  working_tree: untracked
  sha256: 67d6a5e887ff58b121639ec7426339459ed84640cf7da8c3d02f5de3f13d2930
- id: approval
  resource: ../work/initial-design/implementation-approval.md
  working_tree: untracked
  sha256: 1593dc51b5663e513152059e50b39795063a696bf3b6b417c9a5c8fae959834c
- id: scaffold
  resource: ../../../src/lib.rs
  revision: d5985ba2b65571147fbbeb88736e6420c852f86a
- id: core-metadata
  resource: https://packaging.python.org/en/latest/specifications/core-metadata/
- id: expressions
  resource: https://packaging.python.org/en/latest/specifications/license-expression/
- id: simple-api
  resource: https://packaging.python.org/en/latest/specifications/simple-repository-api/
- id: uv-roots
  resource: https://github.com/astral-sh/uv/blob/d041e5700a7bb39bff687ebbba4f6f6f2bda5cc8/crates/uv-lock/src/lock/mod.rs
---
# Approved design and implementation status

Implement a Rust checker that evaluates declared licenses for external Python
dependencies recorded in `uv.lock`, including development and transitive
dependencies. Developers use it before commits and in CI to detect dependencies
outside their project's explicit permission policy. The owner selected these
requirements through a design interview.[^owner]

The owner approved this written design. The
[project baseline](project-baseline.md) and
[stage reviews](../work/initial-design/implementation-reviews.md) distinguish
implemented slices from the complete contract below.
Publishing and release automation remain deferred. The
[architecture decision](sans-io-design.md) defines responsibilities, alternatives,
API contracts, review criteria, and implementation acceptance checks.
The [implementation plan](../work/initial-design/implementation-plan.md) divides
delivery into eight stages with at least five self-reviews per stage.[^scaffold][^owner][^approval]

# Dependency and artifact coverage

Use the locked resolution as the authority. Inspect every external package
record, across Python/platform branches, extras, and dependency groups. Preserve
separate identities for different versions or sources of the same name. Do not
filter by the current environment or require mutually exclusive groups to be
installable together. Traverse cycles without repeating work.[^owner]

Exclude the project itself and its workspace members, while retaining their
external dependencies. Determine membership from supported lock representations:
explicit manifest members and the implicit root identified by an editable or
virtual `.` source. Do not exclude arbitrary editable/directory dependencies.
The cited uv implementation establishes these rules for that revision; fixtures
must establish the supported historical lock range.[^uv-roots]

For each external package, inspect every wheel and sdist recorded in the lock.
Support registry artifacts, direct artifact URLs, and local archive files.
External Git, directory, and editable-directory dependencies produce explicit
incomplete-check diagnostics. A package with no supported artifact or source
evidence also makes the check incomplete. Do not discover unrecorded artifacts
or substitute another version, source, or wheel.[^owner]

Success concerns the declarations of these artifacts. It does not establish
the completeness of declarations about bundled components, the licenses of
future wheels built from sdists, or the licenses of build dependencies absent
from the lock. Do not install, build, or execute inspected packages or backends.
Do not check lock freshness against current dependency declarations, re-resolve,
modify the lock, or invoke uv.[^owner]

# License interpretation

Parse per-artifact Core Metadata. License-Expression applies to its containing
distribution, not the release as a whole. Preserve the observed raw fields and
their provenance in the evidence. Validate metadata identity against the locked
package; identity mismatches make that artifact's check incomplete.[^core-metadata]

Use a valid `License-Expression` when present. Ignore the legacy `License` when
both occur, following Core Metadata. A present invalid License-Expression makes
the check incomplete unless a valid hash-bound clarification supplies the
expression; it must not trigger fallback to another field. When
License-Expression is absent, interpret one complete legacy License value as a
strict SPDX expression first, then a unique official SPDX full name or explicit
versioned/clause-specific alias, then a supported complete license template.
Only whole-name lookup ignores ASCII case; expressions stay strict. Reject names
mapping to differing identities, unsupported family labels, arbitrary extra terms
and invented AND/OR relationships. This expands interpretation within the same
legacy field and does not permit fallback from an invalid License-Expression or
from an arbitrary present legacy value to a different file. The narrow
incomplete-value extension below supersedes the original absent-only rule.
[^legacy-license-review][^constrained-supplement-review]

Reviewed fixed MPL aliases include `MPL 1.1` and
`Mozilla Public License Version 1.1` for MPL-1.1, and `MPL 2.0` and
`Mozilla Public License, Version 2.0` for MPL-2.0. They apply only to the
complete legacy value, with outer whitespace and ASCII case ignored. Unknown
versions, additional terms and versionless `MPL` are not aliases. Normal
allow/deny evaluation follows recognition; this does not grant permission.
The subset is checked against a pinned public mapping rather than importing
ambiguous upstream entries. Reconsider individual aliases if their identity
becomes disputed or conflicts with the pinned SPDX catalog.
[^versioned-license-aliases]

The finite follow-up alias set includes additional explicit MPL, ZPL, BSL, EPL
and CDDL version spellings reviewed against released SPDX v3.29.0 identifiers.
The table, accepted list and rejected ambiguous families are recorded together;
no runtime version inference or source lookup is added.[^interpretation-extensions]

The same fixed table includes individually reviewed variants for MIT,
Apache-2.0, clause-specific BSD, versioned CDDL/EPL and CC0-1.0.
GNU additions resolve only spellings that explicitly grant later-version
permission (GPL-2.0-or-later, LGPL-2.1-or-later and LGPL-3.0-or-later).
Do not map version-only GNU shorthand to `-only`, guess a generic family's
version, interpret an exception abbreviation or turn a style license into its
reference license. The aliases are a curated subset of the pinned reference;
they do not enable generic word removal or punctuation normalization.
[^versioned-license-aliases]

The legacy text matcher reads the original field span in retained metadata;
unfolded values remain the source for expression/name interpretation and original
field reporting. LF/CRLF and Unicode byte offsets must preserve independent
copyright/condition lines. Raw cache bytes and provenance do not change; the
current interpreter re-evaluates existing observations without acquisition.
Structured substitutions allow at most 32 copyright notices and 32 year/range
entries per notice, with narrowly listed corporate/role forms. They do not prove
semantic authorship or licensing completeness.[^legacy-license-review]

Otherwise report unknown license information, which a valid hash-bound
clarification may resolve. Do not infer an expression from classifiers,
unrecognized metadata prose or fuzzy name/text matching. The original declaration-only boundary is extended by the
owner's bundled-file request: when both fields are absent, complete local
license-template recognition may supply the expression. Additionally, one legacy
value equal to UNKNOWN, BSD/BSD License, or Apache/Apache License/Apache Software
License/Apache 2 (whole value, outer whitespace/ASCII case ignored) may be completed
from verified bundled text. BSD requires BSD-2-Clause or BSD-3-Clause; Apache
requires Apache-2.0. UNKNOWN accepts an existing recognized template. Valid
declarations remain first; present expressions, repeated fields, family mismatches
and added conditions cannot be overridden. Fresh archive metadata, not a sidecar
hint, decides eligibility. Completed negative observations remain reusable.
JSON license_origin records the evidence interpretation source before policy
clarifications; text identifies successful legacy supplementation.
[^constrained-supplement-review]

The supported set is MIT, BSD-2-Clause,
BSD-3-Clause, ISC, Apache-2.0 and MPL-2.0; formatting/holder substitutions are narrow,
and mismatched/additional/ambiguous terms remain unknown. AUTHORS basenames are
retained attribution observations, not license candidates; this is a scope rule,
not proof about their legal contents. Other selected files must agree on one
recognized expression. No OR/AND relationship is invented between differing
files.[^core-metadata][^owner][^bundled-license-review]

The complete-text variant extension supersedes Apache's canonical-only rule:
all canonical terms through END remain mandatory, with either no application
appendix or the complete appendix. Only one structured raw copyright line can
vary inside an otherwise exact appendix. MPL-2.0 requires its full text and both
exhibits; only the exhibit locator's HTTP/HTTPS spelling varies. BSD's fixed
plural-holder wording retains complete-body comparison. Shared attribution
grammar accepts exactly two two-word names and a final literal web contact
origin, excluding credentials, paths and parameters. Raw line boundaries and
whole-input comparison must prevent extra material from becoming holder text.
These rules are individually reviewed subsets, not a general SPDX XML engine;
reconsider them only with full-text evidence and independent negative tests.
[^complete-text-variants]

MIT-specific preamble validation extends this subset using released SPDX
v3.29.0 data and normative matching rules. The complete mandatory MIT body must
match first. Only known titles (optionally one Markdown heading marker), absent
preambles, structured yearless/dated notices, fixed role/alias forms and at most
two final bare contact domains are accepted. The new parser preserves raw lines,
enforces notice/contact ordering and bounds its prefix to 5,000 bytes, 32 notices
and 32 dated groups per notice. Unknown prefix material and changed/appended
terms are rejected; the general SPDX copyright wildcard is not implemented.
Other licenses retain their existing notice rules. Reconsider only documented
forms supported by complete-text evidence and independent negative cases.
[^complete-text-variants]

Use SPDX expression syntax and the identifiers allowed by Python's License
Expression specification. Pin the identifier database with the implementation
and document its version. Treat unknown identifiers as invalid unless they are
valid custom LicenseRef identifiers.[^expressions]

Interpret expression terms as explicit permission units:

| Expression | Requirement |
| --- | --- |
| `MIT OR Apache-2.0` | At least one alternative is permitted. |
| `MIT AND Apache-2.0` | Both terms are permitted. |
| `Apache-2.0 WITH LLVM-exception` | That complete combination is permitted. |
| `GPL-2.0-only`, `GPL-2.0-or-later` | Separate units; permission for one does not imply permission for the other. |

Do not infer compatibility, legal obligations, or permission from a license's
family, OSI/FSF classification, or an organization's usual choices. A valid
expression that no permission combination satisfies is a policy violation.
Missing or invalid information is an incomplete check.[^owner]

# Policy and scoped overrides

Load one `[tool.taloon]` policy from the `pyproject.toml` beside the selected
lock. Schema 1 requires exactly one explicit `allow` or `deny` array; both
(including empty arrays) and neither are configuration errors. An empty allowlist
permits nothing. An empty denylist permits recognized standard SPDX units only.
Deny rejects listed exact atomic/WITH units, while OR still requires any permitted
term and AND all terms. Unknown, invalid and missing declarations remain
incomplete; custom references never gain implicit permission. Deny wins over
matching exceptions, while hash-scoped exceptions remain usable for reviewed
custom references. This deliberately extends the original allow-only contract.
[^deny-policy]

Do not merge workspace-member policies or implicit user-wide
policy. The following schema is an implementer choice for review; the owner
delegated key names and serialization details.[^owner]

```toml
[tool.taloon]
schema-version = 1
allow = ["MIT", "Apache-2.0", "Apache-2.0 WITH LLVM-exception"]

[[tool.taloon.exceptions]]
name = "example"
version = "1.2.3"
source = { kind = "registry", location = "https://pypi.org/simple" }
allow = ["BSD-3-Clause"]
reason = "Reviewed for this dependency and release."
expires = "2027-01-31"

# A LicenseRef exception also requires an artifact hash.
# [[tool.taloon.exceptions]]
# name = "custom-example"
# version = "2.0.0"
# source = { kind = "registry", location = "https://pypi.org/simple" }
# artifact-hash = "sha256:<64 hexadecimal digits>"
# allow = ["LicenseRef-Vendor"]
# reason = "Reviewed this artifact's custom terms."

# [[tool.taloon.clarifications]]
# artifact-hash = "sha256:<64 hexadecimal digits>"
# expression = "MIT AND BSD-3-Clause"
# reason = "Reviewed the declared files in this artifact."
# expires = "2027-01-31"
```

Global allow and deny entries contain a single SPDX license identifier or a complete
license-with-exception unit, rather than compound AND/OR expressions. Additive
exceptions require a normalized package name, exact version, source identity,
and nonempty reason. Version ranges and source wildcards are outside this
version. A regular exception applies across that package identity's recorded
artifacts; an optional artifact hash narrows it to one content identity.
Exceptions add permission to the global allowlist.[^owner] In deny mode they cannot
override a listed denial; custom entries may be listed to reject them, never to
grant unreviewed custom permission.[^deny-policy]

Forbid any LicenseRef permission in the global allowlist. An exception allowing
a LicenseRef unit requires an artifact hash in addition to its package selector.
A custom identifier's spelling does not justify permission for another
artifact.[^owner]

A clarification replaces the interpreted license expression for one artifact
content hash. Require a nonempty reason and a valid SPDX expression, preserve
the original metadata, and evaluate the clarified expression under the same
global and scoped permissions. Clarification may remedy absent, invalid, or
inaccurate license declarations; it cannot repair acquisition, archive integrity,
package-identity, or unsupported-source failures. Verify archive content against
the clarification hash before applying it. An unmatched clarification must not
apply to another artifact. Reject conflicting clarifications for the same hash
as invalid configuration.[^owner]

Exceptions and clarifications may have an expiry date. Use one UTC evaluation
date supplied by the runtime for the run. The entry remains valid through its
expiry date; any expired entry is a configuration error, even when unmatched.
Retain reasons and applied override identities in reports. Reject unknown
policy keys and malformed selectors rather than silently ignoring them; these
validation details are implementer choices under Q28.[^owner]

Warn about unused global allow entries, deny entries, exceptions, and clarifications only when
non-use can be established. Use a conservative definition: a global permission
is unused when its unit occurs in no applicable interpreted expression; an
exception is unused when its package/artifact selector matches no target; a
clarification is unused when its hash matches no target. Suppress unused
warnings for runs with incomplete evidence. Do not call a permission unused
merely because another OR branch could satisfy the expression. These warning
definitions are implementer choices under Q28. Deny warnings use distinct
denial wording and observe the same complete-run/all-OR rule.[^owner][^deny-policy]

# Acquisition evidence and cache

Prefer standard per-artifact index metadata. Confirm that the index entry
matches the locked artifact and advertises its locked archive digest; verify
the metadata's advertised digest before using it. Support the current
core-metadata attribute and its legacy dist-info-metadata spelling. An absent
metadata digest, unavailable sidecar, or inability to establish the association
requires archive fallback. A metadata digest verifies the sidecar bytes; the
index supplies the association to the archive. Do not label this evidence as
archive-hash verified.[^simple-api][^owner]

For archive acquisition, verify any lock-recorded archive digest before using
extracted metadata. For a remote artifact whose identity/integrity cannot be
established from the lock and acquisition, report an incomplete check rather
than substituting another artifact. Preserve integrity failures in diagnostics.
Use standard wheel ZIP and sdist tar.gz fixtures as required archive coverage;
record tested additional encodings during implementation.[^owner]

Implementer choice for local archives: read the selected local file and compute
its content digest, verifying the lock digest if present. If the lock provides
no digest, report the observed content identity and its local-file provenance;
this establishes what was inspected, not lock-pinned content. Recompute the
digest on subsequent runs before reusing content-keyed metadata. A missing
local file makes the check incomplete. A hash-scoped permission or clarification
matches the observed, verified content bytes.[^owner]

Evidence distinguishes index-associated, metadata-hash-verified sidecars from
archive-content-verified observations. Require archive-content verification for
manual clarification. Also require it for hash-scoped LicenseRef permission as
an implementer choice consistent with the custom-license content boundary.
Cache provenance at its actual strength; do not promote a sidecar result to
archive-verified evidence because it shares the same lock hash.[^owner]

Cache acquired metadata and provenance by artifact content identity. Validate
record identity, integrity, and format before reuse; schema/parser changes
invalidate incompatible entries. Re-evaluate SPDX interpretation and policy
on every run, including policy changes and expiry dates. Cached verdicts must
not determine acceptance. Keep cache format independent of uv internals.
Corrupt or incompatible records are cache misses. Online execution can fetch
replacement evidence; offline execution reports incompleteness if it cannot
obtain sufficient cached or local evidence.[^owner]

Default execution fetches missing evidence. `--offline` performs no network
requests and keeps the same target scope. A sidecar cache record cannot satisfy
an archive-verification requirement. Hot runs with remote evidence fully cached
perform no network requests, archive downloads, or archive re-extraction.
Local files without lock digests still require content re-hashing.[^owner]

Initially support HTTPS acquisition and HTTP authentication from `.netrc`, using
`NETRC` when specified. Credential-free HTTP transport, uv's credential store,
keyring, and cloud authentication integrations remain outside this version.
Do not print credentials or forward them to a different host during redirects.
Runtime acquisition owns credential discovery, retries, concurrency, and
timeouts. Diagnostics use redacted source locators.[^owner]

# Command and report behavior

`taloon check` searches upward for the nearest directory containing `uv.lock`,
then requires its sibling `pyproject.toml` and policy. A nearer pyproject without
a lock does not stop discovery. `--project <directory>` selects that directory
without upward fallback. Missing/ambiguous inputs are errors. These exact
discovery rules are implementer choices under Q28.[^owner]

`taloon cache clear` uses the same configured/default cache location without
project, policy, credential or HTTP initialization. The cache module alone owns
recognition of completed canonical v1 archive/index cache names. Only immediate
regular files with those names are removed, even when contents are corrupt;
unknown names, temporary files, directories and links remain. Root links/files
are rejected; a missing cache succeeds. Empty directory overrides resolve to
current directory consistently with existing cache writes. Success prints the
removed count and exits 0; failures use safe stderr diagnostics and exit 2.
Clearing is per cache location across projects, not atomic with active checks,
and can make offline evidence insufficient. Revisit recognition whenever cache
naming/schema changes; retain migration/old-record handling deliberately rather
than widening deletion to filename prefixes.[^cache-clear]

Support `--format text|json`, `--offline`, and `--deny-warnings`. Emit a structured
report for both complete and incomplete checks. In JSON mode, reserve stdout
for one versioned JSON report and send incidental diagnostics to stderr. Text
mode summarizes successful checks and groups failures by package and artifact.
Check JSON includes `policy_mode` after valid policy loading, even for empty or
incomplete inspection; startup errors before loading omit it. Artifact permission
decisions include `mode`, and text identifies the selected mode without exposing
configured arrays. Describe retains its separate policy-free schema.[^deny-policy]
Schema versioning and rendering are implementer choices under Q28.[^owner]

Interactive stderr displays acquisition progress as processed/total artifact
counts and elapsed time, including during network waits. Counts include terminal
failures and do not decide compliance. The progress stream contains no package
or source details. Redirected stderr suppresses progress; JSON stdout remains
one final report. This presentation is owned by the binary, with no I/O added
to the pure inspection library.[^interactive-progress]

The acquisition summary retains its original counters and adds HTTP-purpose
request/received-byte/transport-duration/retry-sleep measurements, index parse
and reuse/wait counters, and acquisition wall elapsed time. Durations use integer
microseconds; overlapping worker durations are accumulated separately from wall
time. Aggregation saturates sums and never adds worker wall times. Already
observed traffic is retained after worker unwind. These observations do not
choose policy results or disclose per-source identities.[^acquisition-measurements]

Within each acquisition call, one immutable HTTP/authentication context owns a
sealed catalog keyed by the constructed package endpoint before redirects
(origin/path/query retained). All locked URLs are collected before workers
initialize snapshots. One fetch/parse, or bounded terminal failure, is shared
per endpoint; initialization panic permits retry. Only locked associations are
retained, and exact URL/hash validation remains per request. Each failed lookup
independently falls back to archive verification. Snapshots are never persisted
and do not change evidence strength or target scope.[^acquisition-measurements]

| Exit | Meaning |
| --- | --- |
| `0` | Complete check, all targets comply, no escalated warnings. |
| `1` | Complete check with a policy violation, or warnings escalated by `--deny-warnings`. |
| `2` | Invalid configuration/input or an incomplete check, including mixed violations and incompleteness. |

Continue after per-artifact failures when other targets remain checkable.
Invalid policy or an unreadable/unparseable lock can prevent target discovery
and produce a run-level error. An empty external target set can succeed after
input/policy validation. Deduplicate acquisition for shared content identities
without merging distinct package/source evidence associations.[^owner]

For failures include the package identity, redacted source, artifact identity,
observed or clarified expression when available, evidence provenance, applied
override, and permission/incompleteness reason. Preserve all relevant AND/OR
evaluation reasons. Provide a shortest locked dependency path from each
relevant project/group/extra entry point, retaining marker and extra/group
annotations. Paths describe locked inclusion, not a claim that incompatible
branches coexist in one environment. Use stable tie-breaking, cycle detection,
and a fallback diagnostic for a target lacking a reconstructable path.[^owner]

Provide a local pre-commit example using an installed `taloon check`,
`language: system`, `pass_filenames: false`, and `always_run: true`. A commit
checks the complete lock. CI uses the same command and policy. Publish user
instructions with implementation and verified prerequisites; this draft does
not advertise an available executable.[^owner]

# Acceptance and reconsideration

The [architecture acceptance checks](sans-io-design.md#acceptance-checks) cover
behavior and the two-second warm-cache performance target. Stage reviews record
product checks as implementation progresses; no performance-target measurement
has run yet. Keep user guidance and current implementation
status distinct from this proposed contract.[^scaffold][^owner]

Revisit this contract when adding another lock format, Git/directory sources,
build execution, license-text inference, target selection, policy composition,
new authentication, or a stronger metadata-to-archive trust guarantee. Consult
the owner before shrinking coverage, weakening success, or adding runtime
prerequisites.[^owner]

[^owner]: [Owner decisions and inspected source observations](../work/initial-design/specification-inputs.md).
[^approval]: [Design approval and stage review requirement](../work/initial-design/implementation-approval.md).
[^scaffold]: [Current library scaffold](../../../src/lib.rs).
[^core-metadata]: [Python Core Metadata](https://packaging.python.org/en/latest/specifications/core-metadata/).
[^expressions]: [Python License Expression specification](https://packaging.python.org/en/latest/specifications/license-expression/).
[^simple-api]: [Simple Repository API](https://packaging.python.org/en/latest/specifications/simple-repository-api/).
[^uv-roots]: [uv lock implementation at the inspected revision](https://github.com/astral-sh/uv/blob/d041e5700a7bb39bff687ebbba4f6f6f2bda5cc8/crates/uv-lock/src/lock/mod.rs).

[^interactive-progress]: [Interactive progress tests and review](../work/interactive-progress/review.md).

[^acquisition-measurements]: [Measured acquisition work and review](../work/index-acquisition/measurements.md).

[^cache-clear]: [Confirmed selective cache-clearing design](../work/cache-clear/design.md).

# License-description command

`taloon describe <package>` is implemented as a policy-free request/result
operation over the selected uv.lock. An external name selects itself plus all
normal/extra dependencies across locked environment branches, without development
edges. A workspace/project name selects check's complete external inventory;
own/workspace license observations remain excluded. All matching versions/sources
are included, with AllExternal winning workspace/external collisions. Workspace
routing and cycles remain graph context without recursive output expansion.
Selection occurs before acquisition, which shares check's evidence cache,
authentication, fallback, proof and progress behavior.[^describe-design]

The command reads no pyproject policy or date and applies no clarifications,
exceptions or permission decisions. It uses original declarations or the same absent-declaration bundled-file
recognition as check, exact-expression artifact grouping and explicit unknown/unsupported outcomes; differing artifact
licenses are not joined into a synthetic expression. Complete description exits
0, input/incomplete outcomes exit 2, and no permission exit 1 is produced. Text
lists unique packages and annotated relationships; the separate describe JSON
schema includes roots/scope, graph, observations, diagnostics and acquisition
metrics, including startup errors. Current operation is documented in the
[user command guide](../../user/usage.md#describe-package-licenses). Check's
whole-inventory and permission contract is unchanged.[^describe-design]

[^describe-design]: [Proposed describe design and confirmed owner decisions](../work/describe-licenses/design.md).

Hashless in-memory archive observations retain their acquiring request ID;
Evidence.rebind rejects cross-request substitution in describe, check and cache
publication. Digest-pinned archive reuse remains content-based. Persisted v1
records are unchanged: local cache restoration still requires a fresh observed
content hash from the trusted runtime. This boundary was pinned by a reproduced
substitution regression, independently of the graph model.[^describe-review]

Implementation evidence and staged review limits are recorded separately from
the approved design.[^describe-review]

[^describe-review]: [Staged implementation and validation](../work/describe-licenses/implementation-review.md).


# Consumed JSON index validation

JSON acquisition rejects malformed consumed field shapes, duplicate consumed
keys and positional arrays where object records are required. Explicit null
metadata advertisements invalidate the index rather than selecting the legacy
field. Unknown extensions remain ignored; this decoder does not validate every
Simple API schema field. Index failure causes each locked artifact to fall back
to archive verification with its diagnostic retained. Typed availability hints
supply no metadata digest and cannot upgrade evidence. These stricter rules and
changed scheduling order are intentional under the owner's removal of taloon
backward compatibility, while valid external metadata spellings remain supported.
[^typed-acquisition-review]

[^typed-acquisition-review]: [Typed acquisition staged review](../work/typed-acquisition/review.md).


# Original declaration rendering

Check and describe JSON preserve ordinary declaration prose/email in their
original_expression/original_license fields. Declaration rendering redacts only
embedded URL tokens using the existing credential/query/fragment policy; invalid
URL-like tokens fail closed individually. It does not change permission parsing,
header precedence or cache records. Shared CLI JSON formatting escapes disruptive
control/bidi scalars while preserving decoded values, and text retains its
existing terminal escaping. The reproduction and generated regressions address
Issue #15; parser and cryptographic correctness are separate boundaries.
[^declaration-reporting]

[^declaration-reporting]: [Declaration reporting correction and five-pass review](../work/evidence-and-invariants/issue15.md).

[^bundled-license-review]: [Bundled-file design, network evidence and reviews](../work/bundled-license-evidence/review.md).

[^legacy-license-review]: [Legacy-field interpretation and structured-notice reviews](../work/legacy-license-interpretation/review.md).
[^versioned-license-aliases]: [Versioned alias design and reference](../work/versioned-license-aliases/plan.md).
[^interpretation-extensions]: [Additional interpretation design and staged reviews](../work/license-interpretation-extensions/review.md).
[^complete-text-variants]: [Complete-text variant scope, staged review and verification](../work/license-text-variants/review.md).

[^constrained-supplement-review]: [Constrained supplementation staged review](../work/constrained-license-supplementation/review.md).

# Actionable diagnostic presentation

Unknown-license errors retain their broad code and additionally expose a typed
license_problem, assigned by the interpretation branch. Causes distinguish
missing/repeated declarations, invalid expressions, unrecognized legacy text,
unavailable/unrecognized/conflicting files and family mismatches. Transport and
identity/integrity failures retain separate codes. Causes do not change declared
license meaning, evidence requirements, permissions or exit precedence.

Default check/describe text groups equivalent displayed outcomes within a single
package/source and retains exact full redacted declarations in its key. Counts,
a representative locator, a bounded declaration preview and actionable guidance
replace repeated artifact blocks. Proof strength, origin, diagnostics/status and
policy units/overrides remain discriminants. Applied overrides and supplementation
remain visible even for compliant packages. Verbose shows each artifact and full
report evidence; JSON keeps all artifacts and gains the optional snake_case
license_problem field under schema 1. Diagnostic wording is not a machine
contract; incompatible future layouts require a new schema version.
[^actionable-diagnostics-review]

[^actionable-diagnostics-review]: [Actionable diagnostic assessment and staged review](../work/actionable-license-diagnostics/review.md).

[^deny-policy]: [Deny policy design, implementation and staged reviews](../work/deny-policy/plan.md).
