# Obsolete compatibility review

Base: `1a4b23a` (PR #33 merged); local macOS arm64.
The [plan](plan.md) records scope, state ownership and error-model rationale.

## Error aliases: five self-review passes

1. Usage: source, test, fuzz and tool searches found the four aliases only at
   their declarations. Current fallible domain APIs already return Error.
2. Semantics: Code expresses actionable categories; LicenseProblem retains
   license-specific distinctions. Module ownership alone does not require a new
   recovery or construction boundary.
3. Complexity: remove unused names rather than add forwarding error wrappers
   and conversions. Distinct error enums remain an option for future semantic
   return-type constraints, as documented in the design owner.
4. Public surface: deleting aliases deliberately breaks callers naming them.
   No deprecated re-export or compatibility alias is retained.
5. Behavior: no executable decision, diagnostic payload, code category or CLI
   output changes. Whole-workspace checks cover current consumers.

## Evidence format: five self-review passes

1. Representation: FileObservation owns NotRead, Complete and Unavailable.
   RecordData requires that enum, removing Option/default/omitted serialization.
   Both a missing field and explicit null now fail deserialization.
2. Versions: payload schema 2 is the only restored schema. A schema-1 record
   with a correctly recomputed checksum is rejected. Generated other schema
   values are also rejected. There is no old-format migration or reader.
3. Current states: current sidecars and sufficiently declared archives may be
   NotRead. Completed positive and negative scans retain their outcome; NotRead
   cannot become a successful file interpretation. Existing sidecar proof
   restrictions, file bounds, hashes and checksums remain validated.
4. Runtime: disk-cache miss and online/offline synthetic transport cases prove
   old payloads are not reused. Online reacquires index/metadata; offline sends
   zero requests and reports insufficient cached evidence. The disk envelope
   and filename layout stay unchanged, permitting safe clear/overwrite without
   a compatibility decoder. No filesystem or network access enters the core.
5. Tests/documentation: four new cases failed before the representation change.
   Named independent cases and a generated schema property use AAA with no
   scenario branch in test bodies. Current user/design owners replace previous
   old-record upgrade/checksum-preservation claims; historical work snapshots
   remain evidence of their original revisions.

Scoped software-design-philosophy score: 10/10, all eight diagnostics satisfied.
The enum removes representation assumptions, the error interface loses unused
names, and each current format has one owner. No new generic compatibility
framework, configuration, dependency or I/O path is added. This is not a score
for the entire repository. Formal proof/decision models are unchanged; format
acceptance is tested behaviorally, not claimed formally proved.

## Verification

Independent production-code review found no actionable findings and ran 305
evidence-focused tests. It identified stale current documentation claims, which
were corrected before submission. Final checks are recorded below.

Final workspace: 1,228 passed, 18 ignored, zero failed. Strict all-target,
all-feature Clippy, formatting and warnings-denied Rustdoc passed. Focused
nightly shuffle seed 350002 passed 680 tests (11 ignored). Metadata fuzz smoke
completed 1,000 runs at a 32,768-byte bound on a temporary corpus copy without
a crash. Documentation/OKF checks and strict MkDocs passed.

The final independent review additionally passed the three obsolete-cache
disk/online/offline tests. It identified one remaining current paragraph whose
unchanged-schema wording belonged to an earlier ownership optimization; that
paragraph was qualified before submission. No production-code findings remained.
