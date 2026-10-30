---
type: Contract
title: License inspection specification
description: Approved design for uv.lock coverage, Python license policy, evidence, and command behavior.
status: accepted
sources:
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
License-Expression is absent, accept legacy License only if the complete value
parses as a valid SPDX expression. Otherwise report unknown license information,
which a valid hash-bound clarification may resolve.
Do not infer an expression from classifiers, prose, fuzzy name matching, or
license text.[^core-metadata][^owner]

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
lock. A missing policy is a configuration error. An explicit empty allowlist
permits nothing. Do not merge workspace-member policies or implicit user-wide
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

Global allow entries contain a single SPDX license identifier or a complete
license-with-exception unit, rather than compound AND/OR expressions. Additive
exceptions require a normalized package name, exact version, source identity,
and nonempty reason. Version ranges and source wildcards are outside this
version. A regular exception applies across that package identity's recorded
artifacts; an optional artifact hash narrows it to one content identity.
Exceptions add permission to the global allowlist.[^owner]

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

Warn about unused global permissions, exceptions, and clarifications only when
non-use can be established. Use a conservative definition: a global permission
is unused when its unit occurs in no applicable interpreted expression; an
exception is unused when its package/artifact selector matches no target; a
clarification is unused when its hash matches no target. Suppress unused
warnings for runs with incomplete evidence. Do not call a permission unused
merely because another OR branch could satisfy the expression. These warning
definitions are implementer choices under Q28.[^owner]

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

Support `--format text|json`, `--offline`, and `--deny-warnings`. Emit a structured
report for both complete and incomplete checks. In JSON mode, reserve stdout
for one versioned JSON report and send incidental diagnostics to stderr. Text
mode summarizes successful checks and groups failures by package and artifact.
Schema versioning and rendering are implementer choices under Q28.[^owner]

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
