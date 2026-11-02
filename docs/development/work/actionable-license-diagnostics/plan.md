# Actionable license diagnostics implementation plan

> Execute inline with superpowers:executing-plans. The owner delegated selection
> and implementation; earlier recommended-default authorization remains in force.

**Goal:** Resolve Issue #13 by explaining unknown licenses and reducing repeated
artifact output without changing inspection, acquisition or permission semantics.

**Architecture:** Evidence assigns a typed LicenseProblem at the failure branch.
A shared binary-owned presentation groups identical displayed results plus exact
original declarations within one package. Verbose and JSON retain artifact detail.

**Tech stack:** Existing Rust/serde/clap/rstest/proptest; no new dependencies.

**Spec:** The design and scope assessment below.

## Issue assessment and alternatives

Read open Issues #13, #14, #16, #17, #18 and #19 on 2026-11-02 using gh.
The source is the owner's issue text at https://github.com/HayaoSuzuki/taloon/issues.
#13 is needed now: the implementation conflates missing/repeated declarations and
repeats each wheel in text. #14's manual-draft support and #18's review diffs are
useful subsequent work, after causes and review units are clear. #16 purpose-based
policy guidance, #17 use-specific policy and #19 fulfillment records each need
separate use/distribution assumptions and are deferred, not rejected as useless.
No compatibility presets or automatic permissions are introduced here.

Alternatives: documentation alone cannot distinguish machine causes; message
parsing duplicates interpretation knowledge in rendering. Adding new top-level
codes would unnecessarily disturb the existing UnknownLicense category. Select a
typed optional license_problem on Error, assigned by evidence, plus shared CLI
presentation. JSON schema 1 receives this additive optional field; wording is not
a machine contract, and future incompatible layouts require a schema revision.

## Design / global constraints

- UnknownLicense remains the broad code. LicenseProblem distinguishes missing
  declarations, repeated declarations, invalid expressions, unrecognized legacy,
  unavailable files, unrecognized text, conflicting files and family mismatch.
  The enum is serialized snake_case and attached only to license interpretation
  errors; transport/identity/integrity errors retain their own codes.
- Evidence owns field precedence and raw-file interpretation. Never classify by
  matching diagnostic prose. Keep observation/cache serialization unchanged.
- Default check/describe text groups equal displayed outcomes and exact full
  redacted original expression/legacy arrays within a single package identity/source.
  Status, interpretation, proof strength, origin, diagnostics/severity and policy
  units/overrides participate through the shared result body. File hashes and
  archive digests identify individual artifacts and remain in verbose/JSON.
- Show artifact count, a representative safe locator, original declaration
  preview, typed cause and actionable next step. Preview at most 200 Unicode
  scalars, before terminal escaping; full values participate in grouping.
- --verbose restores per-artifact locators, observed hashes, full declarations,
  file evidence and diagnostics. JSON always retains every artifact regardless
  of --verbose. Neither mode changes scope, verdict, exit code, proof or I/O.
- Existing supplementation labels and policy clarification provenance survive.
  Existing JSON fields remain; missing problem fields deserialize as None.
- Use synthetic artifacts/public license text only. Tests follow AAA, independent
  named rstest cases and generated properties with independent expectations.

## Review focus

1. Multiple wheel declarations differ after a long shared preview: never merge.
2. Acquired evidence has invalid expression plus valid legacy: expression wins.
3. Equal declarations with differing policy overrides/proof/status: never merge.
4. Missing network evidence is not described as a malformed declaration.
5. Terminal controls/credentials in original prose or locators stay escaped/redacted.

## Tasks

### 1. Typed license causes

Files: src/model.rs, src/evidence/mod.rs, src/evidence/file_observation.rs;
tests/license_diagnostics.rs. Error gains optional LicenseProblem; raw persisted
observations do not change. Tests assert serialized causes from real archive or
sidecar interpretation, successful supplementation stays successful, raw replay
retains cause, and generated repeated fields never become invalid-expression.

- [x] Write named public tests and observe absent structured causes (RED).
- [x] Assign causes at existing interpretation branches; preserve broad codes.
- [x] Run targeted/full Rust suite; review five times and commit.

### 2. Shared grouped/verbose presentation

Files: src/cli.rs, src/cli/describe.rs, new src/cli/artifacts.rs;
tests/cli_diagnostics.rs. Borrowed artifact views share rendering/grouping rules;
BTreeMap indexes result body plus full redacted declarations, avoiding pairwise scans.

- [x] Real binary RED cases: same/different declarations and causes, check/describe,
  verbose/JSON detail, mixed acquisition failure, policy decisions and safe text.
- [x] Add --verbose to both commands, route shared presentation; preserve exits.
- [x] Named and generated grouping cases, full/shuffled Rust suite, Clippy/fmt,
  rustdoc, docs build; five review passes and commit.

### 3. Decision/usage and completion

Update docs/user/{usage,evidence}.md and relevant existing OKF owners. Record
assessment and five-pass reviews here, actual source hashes, fresh independent
whole-branch review and its limits. Do not close deferred issues or imply all
six proposals have been delivered.

- [x] Update owners, validate source attribution, OKF and strict user-site build.
- [x] Run bounded metadata/wheel/sdist fuzz for error/replay changes.
- [x] Independent review; resolve important issues with RED/GREEN regressions.
- [x] Push feature branch, create PR closing #13 only and explaining deferred scope.

Plan five-pass review: requirements mapped to tasks; exact branch-owned reason
assignment avoids wording dependency; borrowed view unifies two renderers; full
redacted declarations prevent preview collisions; explicit verification and privacy
limits recorded. Initial design score 8/10: current renderer duplication and
opaque missing/repeated diagnostic fail information hiding and interface clarity.
The typed cause and shared presenter address those two failures.
