# Policy-free description of locked package licenses

Status: owner-facing behavior confirmed in conversation; this concrete design
is proposed for approval before implementation. Current implementation inspected
at `4c481d4` on 2026-11-01. No describe command exists at that revision.

## Confirmed owner decisions

- Provide `taloon describe <package>` within the selected project's uv.lock;
  no public-index discovery or dependency re-resolution.
- Display licenses without permission evaluation or required tool.taloon policy.
- Match every locked identity with the normalized name; distinguish versions
  and sources instead of silently picking one.
- For an external package, include itself and its transitive normal and all
  recorded extra dependencies. Preserve all locked environment branches without
  marker evaluation. Exclude package development edges in this mode.
- For a project name, show the same whole external inventory as check, including
  development, transitive and otherwise unreachable locked records; do not filter.
  Exclude the project's/workspace members' own license observations.
- Keep artifact-specific license differences and unknown declarations visible.
  Never combine different artifacts' declarations into an invented AND/OR.

## Approaches

A policy-free description request/result surface over the existing resolution
and evidence modules is recommended. Graph selection belongs to resolution;
license observation/completeness belongs to the description surface; acquisition
and cache remain shared with check. This retains policy independence and avoids
unrelated artifact acquisition.

Running check and filtering its report would preserve its policy prerequisite,
permission statuses and unrelated acquisition. Adding display modes throughout
Policy/Inspection would spread conditional behavior into the permission engine.
Neither meets the intended boundary as directly. The builder may extract small
shared observation-validation helpers where both callers genuinely need them,
without creating a generic workflow framework.

## CLI and inputs (implementer proposals)

```sh
taloon describe example-client
taloon describe example-client --offline
taloon describe example-client --project /path/to/project --format json
```

Use check's upward uv.lock discovery and exact --project directory behavior.
Read only uv.lock for description: no sibling pyproject.toml, policy, expiry date
or clarification parsing is required. Credentials/cache/progress follow the
existing acquisition runtime. No --deny-warnings, version/source selectors,
extra selectors, live registry search, installation or building is introduced.
Names use the existing validated Python-name normalization. Missing/invalid names,
no matching record, unsupported lock schema and invalid/ambiguous locked edges
are input failures, before any acquisition. The whole lock is still structurally
validated even when the selected artifact scope is narrow.

## Pure selection and graph semantics

Keep graph identities/markers/source resolution private to resolution. Add a
small selection operation exposing typed roots, selected external targets and
annotated dependency relationships, without exposing raw uv structs. The exact
Rust names and ownership signatures are left to the builder.

Use the existing workspace/root classification (including manifest members),
not a second pyproject-based classifier. Implementer proposal: any matching
workspace/root identity selects AllExternal. This includes naming a member of
the same workspace; it is not a member-only dependency command. If a normalized
name matches both workspace and external identities, union semantics mean
AllExternal wins; keep matching root identities explicit in the result.

Otherwise seed every matching external identity, traverse normal edges and
all stored extra groups at every visited node, and deduplicate by full package
identity. Do not activate only incoming selected extras in this new mode:
all recorded optional branches were selected by the owner. Retain edge markers,
package resolution markers, optional-group labels and declared selected extras
as annotations. Traversal may pass through workspace nodes without requesting
or displaying their own license. Keep such intermediate nodes as graph context.
Development edges are excluded throughout this external-root traversal.

Cycles and shared descendants must terminate and appear only once in the
package inventory. Preserve relationships/back-edges as graph data; do not
expand a potentially exponential dependency tree. An external-root result
includes unsupported/missing-artifact targets within its scope as explicit
incomplete packages. Selection never acquires artifacts outside its scope.

AllExternal returns exactly Resolution's complete external target inventory,
not a reachable approximation. Its context includes known workspace/group/extra
origins; optional/dev labels remain distinct. Existing manifest origin hints
retain their best-effort ambiguity behavior, and missing origin information
cannot remove an inventory target.

## Evidence and license meaning

Request every recorded wheel/sdist for selected targets, with existing default
IndexAssociated sufficiency. Share current source/authentication, digest,
identity, cache, sidecar/archive fallback, input limits and warning rules. A
stronger cached record can satisfy a description request; never promote sidecar
proof. All acquired evidence is rebound to the exact expected artifact identity.
Missing, duplicate, foreign or insufficient observations are incomplete.

Reuse the evidence owner's license interpretation: valid License-Expression
first; otherwise a complete valid SPDX legacy License. Missing, repeated or
invalid declarations stay unknown; never guess from classifiers or license text.
Display syntactically valid LicenseRef identifiers as declared, without asserting
permission or supplying definitions. Ignore tool.taloon completely, including
clarifications, custom license definitions, exceptions and expiry. Preserve raw
observations and their proof for JSON/details; policy does not alter description.

Each package summary groups exact validated expression text from evidence.license()
with artifact IDs and counts; no algebraic equivalence or simplification is inferred. Different expressions remain separate values, not equivalent alternatives
for one distribution. Account for all expected artifacts, including unknown or
failed ones. Unsupported packages with no usable artifact still have a terminal
outcome. A valid license difference alone is not incompleteness or a violation.
Acquisition/cache warnings remain warnings if usable declarations are complete.

## Output and exits (implementer proposals)

Default text is a deterministic package list, each package once, with normalized
name, version, redacted source, observed license groups and unknown counts.
Group dependency relationships under the owning package and annotate normal,
extra and marker conditions; project-wide mode also has workspace/group origin
context. This is a superset of locked alternatives, not an installed environment
or a claim that optional edges are simultaneously active. Do not use a recursive
rendering tree. Escape terminal controls and redact sources as in check.

Synthetic illustration (names/licenses below are fixture data):

```text
example-client 1.0  [registry: https://index.example.test/simple]
  MIT: 2/3 artifacts
  Unknown: 1/3 artifacts (missing declaration)
  dependencies:
    example-core 2.0 [normal]
    example-addon 3.0 [extra: feature; marker: python_version >= '3.14']

example-core 2.0  [registry: https://index.example.test/simple]
  BSD-3-Clause: 1/1 artifacts

example-addon 3.0  [registry: https://index.example.test/simple]
  MIT: 1/1 artifacts
```

JSON has its own schema-version-1 describe envelope, not a check compliance
report. Include command, normalized query, matched roots, selection mode,
complete/incomplete status, package/artifact observations, annotated graph/origin
context, diagnostics, acquisition metrics and exit_code. Stable keys/order and
exact Rust type names are builder choices. No allow/deny decision is fabricated.
JSON stdout contains one object, including startup/input errors for describe;
progress stays on interactive stderr. Original check/cache command output stays
unchanged.

Exit 0 means all selected external targets have complete valid declarations;
exit 2 means input error, incomplete acquisition/interpretation or output failure.
There is no policy-violation exit 1. Print available observations even when some
are unknown, without silently presenting partial coverage as complete. A project
with no external targets can complete with an empty inventory; a missing package
name is an input error rather than an empty success.

## Verification and acceptance

Keep the pure core sans I/O. Add typed selection modes and description outcomes;
source, package and request identities retain existing validated types. No new
runtime dependency or persistent-cache format is planned.

Test exact subset requests before acquisition, normalized/all-identity matching,
root/member AllExternal equivalence to check's inventory, collisions, multiple
versions/sources, normal/extra/marker traversal, cycles/shared descendants and
unsupported intermediates. Test all artifact associations, different/unknown
licenses, raw LicenseRef, ignored policy/clarification, missing/duplicate/foreign
results and acquisition-order invariance. Process tests must prove no pyproject
policy requirement, no unrelated downloads, shared cache, offline behavior,
redaction, text/JSON/startup errors and continued check/cache behavior.

Use bounded generated graph permutations and an independent reachability oracle.
A small Lean model should establish selection coverage/reachability invariants
and generate strict Rust correspondence cases if its premises map to the actual
selection API. Do not claim it proves lock parsing, HTTP or metadata truth.
Extend an existing fuzz target or add a focused one only for the newly exposed
input/graph boundary, with bounded deterministic smoke campaigns. Preserve
minimized regressions. Follow existing strict checks and source-attributed OKF
maintenance. At least five self-review passes per stage and one fresh final
whole-branch review precede commit/push/PR handoff; hosted CI must be observed.
Use only public/synthetic fixtures; do not use or mention private examples.

## Design assessment and reconsideration

Proposed design: 7/8 software-design-philosophy diagnostics supported, about
8.75/10. Scope knowledge is in resolution, observed declarations in description/
evidence, and I/O in acquisition; callers need only requests and typed results.
The alternative of pushing permission/display flags throughout Inspection is
avoided. The design-effort time fraction is not measured; record time in future
work to substantiate the final diagnostic. Reassess ownership/comments after
implementation; this is not a score for unwritten code.

Revisit if environment/extras selection, per-version/source disambiguation,
workspace license acquisition, policy-aware describe, package-index discovery,
or additional lock formats are requested. These alter the product contract,
not merely rendering.

See the [design review record](design-review.md) for five self-review passes and
the independent builder check.
