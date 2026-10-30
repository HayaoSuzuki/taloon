# Python dependency license inspection implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans for inline execution, or superpowers:subagent-driven-development if the owner selects delegation. Execute task by task. Steps use checkbox syntax for tracking.

**Goal:** Deliver `taloon check` with the approved uv.lock coverage, Python license policy, acquisition evidence, offline cache, and local pre-commit workflow.

**Architecture:** Keep resolution, policy, evidence, and inspection pure in the existing library. Put file/network/cache operations and CLI wiring in the binary. Inspection owns coverage and minimum evidence requirements; acquisition fulfills opaque requests and returns observations or failures.

**Tech stack:** Rust 1.99.1, edition 2024; serde/TOML/JSON, SPDX expressions, PEP 440 versions, URL normalization, SHA-256, ZIP/tar/gzip, a blocking HTTPS client with a bounded worker pool, and clap. Resolve compatible crate releases at their owning stage, review upstream documentation and licenses, and commit Cargo.lock. Prefer existing crates over independent implementations of these formats; do not add uv internal crates.

**Spec:** [Inspection contract](../../knowledge/license-inspection.md), [architecture and acceptance checks](../../knowledge/sans-io-design.md), and [owner approval and five-pass requirement](implementation-approval.md). Read all three before execution.

## Global constraints

- Implement in Rust; `uv.lock` is the only supported lock format.
- Check every external locked dependency, including development and transitive dependencies, across environment branches, extras, and groups.
- Check every wheel and sdist recorded for each external dependency; exclude only the project/workspace members themselves.
- Never execute inspected packages or backends, invoke uv, re-resolve, update locks, or infer licenses from text/classifiers.
- Policy lives only in the sibling pyproject.toml's tool.taloon; no cargo-deny compatibility, member-policy merge, or user-wide policy.
- Success requires complete coverage and compliance; exits are 0, 1, and 2 with incompleteness taking precedence.
- Pass one UTC evaluation date into the pure core; keep files, network, environment, processes, and clock access in the binary.
- Keep index-associated sidecars distinct from archive-content-verified evidence; clarification and hash-scoped LicenseRef permission require the latter.
- Re-evaluate policy on each run; offline mode makes zero network calls without shrinking coverage.
- Preserve Elastic-2.0, existing strict Clippy settings, SHA-pinned CI, and the repository documentation procedure. Publishing remains deferred.
- Complete at least five recorded self-review passes per stage before advancing. Passing tests do not replace self-review.
- Model permission evaluation, proof-strength requirements, completeness, and exit precedence in pinned Lean; generate a versioned oracle corpus and compare it with real Rust public behavior. A model proof does not prove Rust correspondence or I/O correctness.
- Use proptest for semantic invariants and cargo-fuzz/libFuzzer for parser/archive/inspection boundaries; retain minimized regressions and run bounded smoke checks. Runtime users require neither Lean nor fuzz tooling.
- Use validated newtypes and exhaustive enums for identities, source kinds, permissions, evidence strength, statuses, and diagnostic codes; convert strings only at serialization boundaries. Avoid public caller-set verification flags and boolean combinations representing invalid states.
- The two-second warm-cache target for 1,000 dependencies/10,000 artifacts is a measured goal, not a wall-clock CI gate.

## Review focus

1. A direct URL with credentials, signed query values, or percent-encoding must retain fetching/matching semantics while reports and cache records hide secrets; test in stages 1, 6, and 7.
2. Folded/repeated Core Metadata headers, a missing field, or a future metadata version must not turn unknown or contradictory information into permission; test in stage 3.
3. A workspace member absent from the manifest list, a same-name external package, or a stale edge must not disappear from coverage or create an invented path; test in stages 2 and 4.
4. A process interrupted while publishing cache data, concurrent writers, or a policy change requiring stronger evidence must not reuse a partial record or promote a sidecar; test in stages 4 and 5.
5. One failed artifact among many, result reordering/duplication, or redirected authentication must not cause early success or leak credentials; test in stages 4, 6, and 7.

## File ownership and shared interfaces

| Files | Knowledge owner |
| --- | --- |
| `src/model.rs` | Validated package/source/content identities and UTC date values. |
| `src/policy/mod.rs`, `src/policy/tests.rs` | Policy parsing, permission units, overrides, expiry, and SPDX evaluation. |
| `src/resolution/mod.rs`, `src/resolution/tests.rs` | Version-aware lock decoding, membership, targets, and annotated graph paths. |
| `src/evidence/mod.rs`, `src/evidence/tests.rs` | Metadata/archive interpretation, content verification, and evidence record validation. |
| `src/inspection/mod.rs`, `src/inspection/tests.rs`, `src/report.rs` | Expected requests, coverage accounting, policy outcomes, and stable reports. |
| `src/acquisition/mod.rs`, `cache.rs`, `http.rs`, `tests.rs` in that directory | Binary-owned file/cache/index/network operations and authentication. |
| `src/main.rs`, `src/cli.rs`, `tests/cli.rs` | Input discovery, runtime date, options, rendering, and process-level behavior. |
| `tests/fixtures/`, `tests/common/mod.rs` | Small deterministic synthetic distributions and resolution cases; no live services. |
| `tests/workflow.rs`, `tests/performance.rs` | Complete local workflow and an ignored, opt-in process-level benchmark. |
| `formal/TaloonOracle/` and `tests/fixtures/oracle/` | Pinned Lean model, kernel-checkable proofs, sensitivity witnesses, deterministic generated corpus, and Rust correspondence inputs. |
| `tests/oracle.rs`, `tests/properties.rs`, `fuzz/` | Public-API oracle adapter, generated semantic invariants, and independent cargo-fuzz targets/corpus. |
| `docs/user/`, relevant knowledge concepts, `mkdocs.yml` | Tested user instructions and current implementation guarantees. |

Keep module internals private. The signatures below are the plan's cross-stage
contract; change them together if implementation proves a simpler interface.
Use accessors rather than public invariant-bearing fields. Type aliases to
upstream value types are acceptable when they preserve these semantics.

| Owner | Types and interfaces |
| --- | --- |
| Stage 1 model | `EvaluationDate::parse(&str) -> Result<Self, InputError>`; `PackageId::parse(name: &str, version: &str, source: SourceId) -> Result<Self, InputError>`; `SourceId::parse(kind: &str, location: &str) -> Result<Self, InputError>`; `ContentHash::parse(&str) -> Result<Self, InputError>`. |
| Stage 1 policy | `Policy::from_pyproject(&str, EvaluationDate) -> Result<Self, InputError>`; `LicenseExpression::parse(&str) -> Result<Self, LicenseError>`; `Policy::minimum_evidence(&ArtifactRequest) -> EvidenceRequirement`; `Policy::evaluate(&ArtifactRequest, ContentHash, &LicenseExpression) -> PolicyDecision`; `Policy::clarification(&ArtifactRequest, ContentHash) -> Option<&Clarification>`; `PolicyDecision::is_permitted() -> bool`. Use the observed hash for hash-scoped overrides, including local files lacking a lock hash. |
| Stage 2 resolution | `Resolution::from_uv_lock(&str) -> Result<Self, InputError>`; `Resolution::targets() -> &[Target]`; `Resolution::paths(&PackageId) -> Vec<DependencyPath>`. |
| Shared model | `ArtifactRequest::new(PackageId, ArtifactLocator, ArtifactFormat, Option<ContentHash>) -> Self`; `ArtifactLocator::{registry(index: &str, url: &str), url(&str), local(&str)} -> Result<Self, InputError>`; `ArtifactFormat::{Wheel, Sdist}`; opaque RequestId; `EvidenceRequirement::{IndexAssociated, ArchiveVerified}`. Resolution creates requests with the default requirement; inspection raises it via `ArtifactRequest::requiring(EvidenceRequirement) -> Self`. `Target::requests() -> &[ArtifactRequest]` and `Target::package() -> &PackageId` expose supported requests; unsupported-source targets remain represented without fabricated requests. |
| Stage 3 evidence | `Evidence::from_archive(&ArtifactRequest, &[u8]) -> Result<Self, EvidenceError>`; `Evidence::from_sidecar(&ArtifactRequest, SidecarAssociation, &[u8]) -> Result<Self, EvidenceError>`; `Evidence::license() -> Result<LicenseExpression, LicenseError>`; `Evidence::to_record() -> EvidenceRecord`; `Evidence::from_record(&ArtifactRequest, EvidenceRecord) -> Result<Self, EvidenceError>`. |
| Stage 3 supporting types | `SidecarAssociation::new(locked_archive: ContentHash, index_archive: ContentHash, metadata: ContentHash) -> Result<Self, EvidenceError>`; opaque serde-compatible `EvidenceRecord`; `Evidence::content_hash() -> ContentHash`; `Evidence::requirement_satisfied(EvidenceRequirement) -> bool`; `Evidence::summary() -> EvidenceSummary`, containing provenance, original license fields, and metadata diagnostics for reporting. |
| Stage 4 inspection | `Inspection::from_documents(&str, &str, EvaluationDate) -> Result<Self, InputError>`; `Inspection::requests() -> &[ArtifactRequest]`; `Inspection::evaluate(&self, Vec<AcquisitionResult>) -> Report`; each `AcquisitionResult` associates `RequestId` with `Result<Evidence, AcquisitionFailure>`. |
| Stage 4 report | `Report::exit_code(deny_warnings: bool) -> u8`; `Report::with_warnings(self, Vec<Diagnostic>) -> Self`; `Report::with_acquisition_summary(self, AcquisitionSummary) -> Self`; deterministic serde-compatible semantic report schema version 1; run-level input errors use the same envelope via `Report::from_input_error(InputError) -> Self`. Diagnostic is a shared model value with severity, machine code, and safe message; AcquisitionSummary holds measured network_requests, downloaded_bytes, archive_extractions, and cache_hits counters. These counters do not decide license outcomes. |
| Stage 5 cache | Binary-private `EvidenceCache::new(PathBuf) -> Self`; `EvidenceCache::get(&ArtifactRequest) -> Result<Option<Evidence>, CacheError>`; `EvidenceCache::put(&ArtifactRequest, &Evidence) -> Result<(), CacheError>`. |
| Stage 6 acquisition | Binary-private `Acquirer::new(AcquisitionOptions) -> Result<Self, AcquisitionFailure>`; `Acquirer::acquire(&[ArtifactRequest]) -> AcquisitionBatch`; the batch contains `Vec<AcquisitionResult>`, cache-write/runtime warnings, and measured AcquisitionSummary counters. |
| Stage 7 CLI | `cli::run() -> u8`; use the stage 4 report and status semantics rather than reimplementing them. |

`InputError`, `LicenseError`, `EvidenceError`, `AcquisitionFailure`, and
`CacheError` are structured diagnostic types defined by their owners. Store
stable enum-backed machine codes and secret-safe messages. Model source kinds,
artifact locators, evidence requirements, and result statuses with exhaustive
enums. Validated package names, versions, dates and hashes are private newtypes.
Shared request constructors live
with the model/resolution owner; tests can use module-private fixture builders.
Cache envelopes belong to acquisition; EvidenceRecord validates proof semantics
and raw metadata. A local cache is trusted persisted output, not a cryptographic
proof that an arbitrary third party observed the archive.

## Formal and generated-test workflow

Follow hoimin's separation of Model, Proofs, Cases, and an executable generator.
Use `formal/TaloonOracle/lean-toolchain` pinned to the inspected
`leanprover/lean4:v4.32.2`, a dependency-free lake project, and a deterministic
schema-version-1 JSONL corpus. Keep proofs cheap to import; keep enumeration and
serialization in the executable. Add correct examples and deliberately broken
witnesses before implementing the corresponding rule. Do not hand-edit generated
expectations or derive them with the Rust code under test.

The first model owns atomic permission units (WITH stays one atom), AND/OR
evaluation, evidence strength, expiry boundaries, complete expected-target
accounting, and exit precedence. Prove permission monotonicity, insufficient
evidence rejection, missing-target rejection, and incomplete-over-violation
precedence under explicit premises. Prove properties over arbitrary expressions
or target lists with induction where appropriate; finite case enumeration alone
is not an unbounded theorem. Add fixed broken witnesses for swapped AND/OR,
sidecar promotion, missing-result acceptance, and reversed exit precedence.

Before the model, record a correspondence worksheet with premise, Lean value,
Rust public entry point, observation, evidence, and mode. Use exactly strict,
model-only, internal-fixture, or infrastructure-error. Cases eligible for the
public policy/inspection APIs run in isolation through `tests/oracle.rs`; report
match/mismatch separately from setup/parse/panic failures. Do not weaken the
model to fit production. Record reproductions and unresolved counterexamples.

Use an external 20-second deadline for initial individual Lean commands, and
one Lean process at a time. Record finite case counts and resource measurements.
Do not use unlimited heartbeats or increase exploration bounds after a sharp
cost jump; prefer inductive lemmas. CI checks corpus freshness after building
proofs, then runs the strict Rust adapter.

Use proptest at each owning stage: explicit-unit permission monotonicity,
name normalization, target coverage under lock/result reordering, identity/source
separation, hash mutation rejection, unchanged outcomes after evidence cache
roundtrip, and exit precedence. Begin with 256 cases and persisted minimized
regressions; compare with independent predicates, not implementation helpers.
Use depth-bounded expression/graph generators and make resource bounds explicit.

Create independent cargo-fuzz targets for policy TOML, uv.lock, metadata bytes,
wheel/sdist bytes, and structured inspection results. Seed with valid and
malformed fixtures; run both valid structured inputs and arbitrary bytes so
fuzzing reaches semantic code as well as rejection paths. Initial smoke runs use
1,000 executions and a 4,096-byte input limit per parser target; archive targets
also receive valid compressed seeds. Run on installed pinned nightly
`nightly-2026-10-08` and keep discovered crash inputs as regressions. The separate
fuzz workspace must not add libFuzzer to production dependencies.

Add the formal workflow in stages 1 and 4, properties in stages 1–7, fuzz targets
as their parser APIs become available in stages 1–4, and CI/reproduction evidence
in stage 8. Record sources copied from hoimin with revision/license attribution;
prefer a fresh taloon model over importing unrelated hoimin semantics.

## Stage review and completion gate

For each task below, perform five distinct self-reviews after the first passing
implementation. Record them in
`docs/development/work/initial-design/implementation-reviews.md` under that
stage, with the reviewed tree/diff or commit, scope, concrete findings, changes,
and evidence. Write “no finding” only after examining the stated scope.

1. **Specification:** trace that stage's requirements to outcomes, including accepted exclusions.
2. **Adversarial inputs:** trace boundaries, malformed/absent data, ambiguity, and identities against its fixtures.
3. **Design:** check ownership, interface contracts, information leakage, and the eight design diagnostics; record score, failed rows, and fixes.
4. **Failure/recovery:** trace partial work, offline behavior, concurrent/order-dependent operations, and escalation as applicable to that stage.
5. **Verification:** inspect red/green evidence, meaningful assertions, remaining limits, and the final diff/documentation.

Fix findings and rerun the checks affected by each fix. These are five actual
review passes, not five assertions of completion or five repeated test runs.
Stage-specific emphasis appears below. Record further passes if corrections
uncover new issues. Commit only after the gate passes; keep unrelated changes
out of task commits. Use an isolated execution checkout according to the
worktree skill, preserving the approved design and this plan.

---

## Task 1: Validated identities and policy evaluation

**Files:** Modify `Cargo.toml`, `Cargo.lock`, `src/lib.rs`, and `deny.toml` if justified by actual dependencies. Create `src/model.rs`, `src/policy/mod.rs`, `src/policy/tests.rs`.

**Interfaces:** Produce the stage 1 model/policy/shared-request interfaces above. Use an SPDX crate for syntax and explicit permission evaluation; use parsed PEP 440 version equality, not a range matcher that permits local versions implicitly. Keep source identity distinct from fetch/display locators.

- [x] Write `nested_expression_respects_explicit_units` and `policy_requires_exact_scoped_overrides` with these assertions: allowing only MIT accepts `MIT OR Apache-2.0`, rejects `MIT AND Apache-2.0`, and rejects `MIT WITH LLVM-exception`; accepting a valid WITH unit requires that exact unit; separate GPL-only/or-later cases stay separate. Scope exceptions by name/version/source and optional hash. For named PolicyDecision values assert `assert!(mit_or.is_permitted()); assert!(!mit_and.is_permitted()); assert!(!mit_with.is_permitted());`; define `PolicyDecision::is_permitted() -> bool` with the decision type.
- [x] Write policy tests requiring schema-version 1 and an explicit allow array; reject missing/unknown keys, compound global entries, global LicenseRef, broad versions, blank reasons, conflicting clarifications, and expired unmatched entries. On evaluation date `2026-10-30`, an expiry of `2026-10-30` is valid and `2026-10-29` is invalid.
- [x] Run `cargo test --locked --lib policy` and `cargo test --locked --lib model`; confirm the new tests fail on the scaffold before implementing behavior. Add fixture declarations/imports as needed without implementing the assertions' behavior.
- [x] Implement the stated constructors and policy methods. Centralize name/source/version/hash normalization and redaction. Require custom permission and clarification archive verification. Record the SPDX list version from the chosen crate.
- [x] Test encoded direct URLs, userinfo and signed-query redaction, different sources sharing a name/version, local version distinctions, and no cwd/environment dependence. Then rerun the two targeted commands; expect all policy/model tests to pass.
- [x] Before this gate, build the permission/expiry Lean slice, its negative/broken witnesses, and generated corpus; add strict policy correspondence and proptest monotonicity/normalization. Add the policy fuzz target and run its bounded smoke campaign. Record actual results, not merely target existence.
- [x] Complete the five-pass gate, emphasizing selector identity, AND/OR/WITH semantics, and expiry. Run formatting/Clippy for touched targets and the license/source check after dependency additions. Commit as `feat: add license policy and validated package identities`.

## Task 2: Complete locked resolution and dependency paths

**Files:** Create `src/resolution/mod.rs`, `src/resolution/tests.rs`, `tests/fixtures/locks/`. Modify `src/lib.rs` and the inspection contract's tested compatibility range when evidence exists.

**Interfaces:** Consume stage 1 identities; produce Resolution, Target, annotated DependencyPath, and artifact requests. The core never reads a path from disk.

- [x] Write `all_locked_branches_and_artifacts_are_targets`: a synthetic lock has an implicit root, dev group, extra, transitive cycle, and eight external targets: dep, leaf, two branch versions, two dup sources, an external directory, and Git. Give the six archive-backed targets nine recorded artifacts in total; assert `assert_eq!(resolution.targets().len(), 8);` and `assert_eq!(resolution.targets().iter().map(|t| t.requests().len()).sum::<usize>(), 9);`, plus the exact identity set. Assert that the root's absence from manifest.members does not make it external and unsupported directory/Git targets remain incomplete rather than excluded.
- [x] Write `paths_preserve_entrypoint_and_conditions`: two entries reach one dependency; assert one shortest path per entry with marker/extra/group labels, stable ties, and cycle termination. Assert unresolved/ambiguous edge references cannot invent a source/version association.
- [x] Run `cargo test --locked --lib resolution`; confirm the named tests fail before implementation.
- [x] Implement `Resolution::from_uv_lock`, `targets`, and `paths`. Start with real fixture-backed `version = 1, revision = 5` from this repository and derive any older supported revisions from inspected uv fixtures. Add provenance for copied upstream fixtures. Reject unsupported semantic changes and unknown source forms; preserve per-target unsupported-source diagnostics.
- [x] Add fixtures for explicit members, virtual root, same-name external identity, no artifacts, URL/local archives, unknown version/revision, and benign additive fields; run `cargo test --locked --lib resolution` and expect all assertions to pass. Record the actual supported range rather than assuming all version-1 locks work.
- [x] Include proptest lock reorder/identity/coverage invariants and the uv.lock fuzz target before this stage's review/commit gate.
- [x] Complete five reviews, emphasizing coverage and graph identity. Update contract evidence with tested scope, run touched-code checks, and commit as `feat: inspect complete uv locked dependency graphs`.

## Task 3: Verified distribution and metadata evidence

**Files:** Create `src/evidence/mod.rs`, `src/evidence/tests.rs`, `tests/fixtures/distributions/`. Modify `Cargo.toml`, `Cargo.lock`, `src/lib.rs` and dependency policy only as required.

**Interfaces:** Consume ArtifactRequest and ContentHash; produce Evidence, EvidenceRecord, SidecarAssociation, and EvidenceError. Constructors inspect bytes only and keep proof state private.

- [x] Write `archive_and_sidecar_proofs_remain_distinct`: matching SHA-256 wheel bytes yield ArchiveVerified evidence; changed bytes fail; a hashed sidecar with matching archive association yields only IndexAssociated evidence; mismatching archive or metadata hashes fail. Assert `assert!(archive.requirement_satisfied(EvidenceRequirement::ArchiveVerified));` and `assert!(!sidecar.requirement_satisfied(EvidenceRequirement::ArchiveVerified));`.
- [x] Write `metadata_license_precedence_and_identity`: valid License-Expression wins over License; invalid License-Expression does not fall back; absent expression allows a whole valid legacy SPDX value; classifier/prose alone stays unknown. Assert name/version mismatch fails independently of any license clarification.
- [x] Run `cargo test --locked --lib evidence`; confirm failing behavior before implementing the constructors and parsing.
- [x] Implement the stated evidence API using wheel ZIP and sdist tar.gz readers over bytes, SHA-256, and a Core Metadata parser. Preserve raw headers and original declarations. Validate supported Metadata-Version rules, repeated singleton fields, continuation lines, UTF-8, missing identity, and duplicate/ambiguous archive metadata members. Do not extract archive paths onto the filesystem.
- [x] Add folded headers, contradictory duplicates, metadata 2.4 and 2.6, a future compatible minor warning, an unsupported major error, sdists with dynamic license information, corrupt archives, and evidence-record checksum/schema/identity failures; rerun `cargo test --locked --lib evidence` and require pass. Record archive and metadata compatibility and the runtime's size/buffering limits.
- [x] Include proptest byte-mutation/proof-strength invariants and metadata/wheel/sdist fuzz smoke runs before this stage's review/commit gate.
- [x] Complete five reviews, emphasizing proof strength, metadata ambiguity, and resource use. Check added dependency licenses and commit as `feat: verify artifact license metadata evidence`.

## Task 4: Coverage-owning inspection and stable reports

**Files:** Create `src/inspection/mod.rs`, `src/inspection/tests.rs`, `src/report.rs`. Modify `src/lib.rs`.

**Interfaces:** Consume stages 1–3; produce Inspection, AcquisitionResult, AcquisitionFailure, Report, and the report exit-code contract. `Inspection::evaluate` owns complete expected-result accounting.

- [ ] Write `missing_or_weak_results_never_succeed`: build an inspection containing two artifacts, return one observation, and assert `assert_eq!(missing.exit_code(false), 2);`; return two permitted observations and assert `assert_eq!(complete.exit_code(false), 0);`; a complete valid unpermitted expression yields `assert_eq!(violation.exit_code(false), 1);`. A mixed violation/incomplete result yields 2. Empty validated external scope yields 0.
- [ ] Write `clarification_is_not_permission`: malformed/absent expression plus a matching verified archive clarification still violates policy when the clarified unit is unpermitted. Assert wrong hash, expired config, wrong metadata identity, unsupported source, or weak sidecar cannot pass via clarification/custom permission.
- [ ] Run `cargo test --locked --lib inspection` and `cargo test --locked --lib report`; confirm the new assertions fail before implementation.
- [ ] Implement the stated Inspection API, report schema version 1, run-level error envelope, scope matching, conservative unused warnings, and ordered diagnostics/paths. Include applied reasons and proof strength. Use expected request identities to detect missing, unknown, and contradictory duplicate results.
- [ ] Add permutation tests yielding identical semantic JSON, shared-content/different-source cases, original-versus-clarified evidence, UTC boundary expiry, warnings suppressed on incompleteness, unused OR-term handling, --deny-warnings exit 1, and incomplete-with-warning exit 2. Include two distinct requests sharing content and assert one verified observation can be rebound to both with separate policy/source decisions, while omission of either result association remains incomplete. Rerun targeted inspection/report tests and require pass.
- [ ] Complete the Lean coverage/evidence/exit slice and prove its stated invariants; regenerate/check the corpus and run the strict public Inspection adapter. Add proptest result permutations/precedence and structured-inspection fuzzing before the review/commit gate.
- [ ] Complete five reviews, emphasizing completeness, deterministic aggregation, and stable machine diagnostics. Commit as `feat: evaluate complete inspections and explain outcomes`.

## Task 5: Local acquisition and recoverable evidence cache

**Files:** Create `src/acquisition/mod.rs`, `src/acquisition/cache.rs`, `src/acquisition/tests.rs`. Create the minimal `src/main.rs` module declaration so binary-private cache tests compile. Modify Cargo manifests for cache/test prerequisites.

**Interfaces:** Consume requests, EvidenceRecord, Evidence, and structured failures; implement EvidenceCache and a binary-private `acquire_local(&ArtifactRequest, &EvidenceCache) -> Result<Evidence, AcquisitionFailure>` helper. No public library file access.

- [ ] Write `cache_requires_matching_identity_strength_and_format`: write/read verified evidence, reject mismatched content/package identity, reject a sidecar hit for a full-archive requirement, and treat corrupted/incompatible entries as misses. Assert `assert!(cache.get(&archive_required).unwrap().is_none());` for a sidecar-only record. Archive-verified content may satisfy another source association only after validating that request's package identity and digest; index-associated sidecars retain source-specific association. Assert policy/date never lives in the acceptance cache key or cached verdict.
- [ ] Write `local_archive_without_lock_digest_is_rehashed`: inspect local bytes, change that same path's contents, and assert the new digest/evaluation is used; assert a recorded lock hash mismatch fails and a missing file remains incomplete. A restored archive-verified record preserves its original strength, not a newly asserted proof.
- [ ] Run `cargo test --locked --bin taloon acquisition`; confirm failures before cache/local acquisition implementation.
- [ ] Implement the cache API, versioned/checksummed envelope, content/association-aware lookup, atomic publication, and local acquisition. Store evidence only; validate on read and keep credentials out of cache data. Use platform cache location with an injected path in tests; create no persistent user cache in unit tests.
- [ ] Test two writers publishing the same evidence, interrupted temp publication, truncated committed entry, read-only/unavailable cache, and in-memory success with cache-write warning. Run `cargo test --locked --bin taloon acquisition` and require pass.
- [ ] Include proptest cache roundtrips and mutation/corruption rejection before this stage's gate; keep disk atomicity/concurrency tests separate from model proof claims.
- [ ] Complete five reviews, emphasizing crash/concurrency behavior, content re-hashing, and evidence upgrades. Keep main's temporary entry-point behavior explicit until task 7; commit as `feat: cache validated evidence and inspect local archives`.

## Task 6: HTTPS index/sidecar/archive acquisition

**Files:** Create `src/acquisition/http.rs`; extend `src/acquisition/mod.rs` and its tests; create binary-private local HTTP/TLS fixture helpers. Modify Cargo manifests and deny.toml only with reviewed dependency evidence.

**Interfaces:** Implement Acquirer, AcquisitionOptions, AcquisitionBatch. Options include project base directory, cache path, offline flag, and tested internal concurrency/timeouts/retries. Batch warnings join the stage 4 report without changing coverage accounting.

- [ ] Write `sidecar_fallback_and_offline_keep_scope`: a fixture index with current/legacy hashed metadata serves a sidecar; absent/unhashed sidecar falls back to a lock-hash-verified archive. Offline missing evidence yields a failure for each unresolved request and `assert_eq!(transport.request_count(), 0);` through a private fixture counter. A cached sidecar plus a new clarification triggers archive acquisition online and incomplete offline.
- [ ] Write `redirects_and_errors_do_not_leak_or_skip`: assert NETRC precedence, missing credentials, same-host versus host-changing redirects, TLS failure, timeout/retry exhaustion, a bad digest, and one failed request among successes retain safe diagnostics and full result identity coverage.
- [ ] Run `cargo test --locked --bin taloon acquisition`; confirm the new HTTP behavior fails before implementation.
- [ ] Implement HTTPS acquisition with one reusable blocking client and a bounded worker pool; keep fallback/retry/index parsing here. Prefer standard Simple API JSON with HTML compatibility. Match the exact locked artifact and archive digest; never follow a package's latest release or substitute another URL/version. Disable unchecked auth forwarding and redact sensitive URL values in errors.
- [ ] Add fixtures for percent-encoding, URL fragments, signed queries, conflicting index hashes, 404 sidecars, HTML base/relative URLs, oversized/malformed responses, TLS redirects, content dedup with separate associations, and permuted completion order. Local HTTP is test-only through an injected transport; production remains HTTPS. Rerun targeted binary acquisition tests; require pass and recorded request counts.
- [ ] Include generated URL/association and result-order properties before this stage's gate, with bounded graphs and explicit secret-redaction assertions.
- [ ] Complete five reviews, emphasizing trust boundary, offline transport reachability, credential isolation, and worker completion. Record selected defaults, tested authentication and supported protocol variants; commit as `feat: acquire all locked artifact evidence over HTTPS`.

## Task 7: Usable CLI, diagnostics, and local hooks

**Files:** Implement `src/main.rs`, create `src/cli.rs`, `tests/cli.rs`, `tests/common/mod.rs`, and `tests/fixtures/projects/`. Update `docs/user/index.md`, add focused usage/configuration/troubleshooting pages, and update `mkdocs.yml`. Update implementation status in knowledge owners when behavior becomes available.

**Interfaces:** Consume Inspection, Acquirer, Report, and their errors; implement `cli::run() -> u8`. Obtain the UTC date and runtime environment once. No policy interpretation or archive logic in argument handlers.

- [ ] Write process tests `cli_finds_nearest_lock_and_requires_sibling_policy` and `cli_json_reports_complete_and_incomplete_runs`: assert nearest-lock discovery through a member directory, explicit project without fallback, missing/invalid config, summary for zero targets, and exit 0/1/2. Assert JSON stdout contains exactly one schema-version-1 object and stderr holds incidental errors.
- [ ] Write `cli_offline_policy_change_rechecks_cached_evidence`: populate a temporary fixture cache by invoking the real CLI on a local archive project, change allow permissions, and assert changed verdict without network/archive extraction; test --deny-warnings and report/path redaction. For the two process outputs assert `assert_eq!(allowed.status.code(), Some(0)); assert_eq!(denied.status.code(), Some(1));`. Isolate cache through documented `TALOON_CACHE_DIR`; otherwise use the platform cache location. Do not duplicate the private cache encoding in tests.
- [ ] Run `cargo test --locked --test cli`; confirm failures before implementing parsing, discovery, rendering, and orchestration.
- [ ] Implement `taloon check`, --project, --format text/json, --offline, and --deny-warnings. Convert startup errors into the report envelope even in JSON mode. Attach measured AcquisitionSummary counters and runtime warnings through the report interfaces; keep warning escalation precedence consistent. Preserve deterministic semantic report ordering.
- [ ] Add `cli_never_resolves_builds_or_mutates_inputs`: capture lock/pyproject bytes before and after a check, place sentinel uv/Python executables on the child's PATH, and include executable build code in an sdist fixture. Assert both documents remain byte-identical, no sentinel executes, and the declaration alone determines the result.
- [ ] Add user instructions using tested local `cargo install --locked --path .` prerequisites, TALOON_CACHE_DIR, policy/clarification/custom-license examples, supported lock/metadata/artifact/auth ranges, offline behavior, trust limits, and a local hook with language system, pass_filenames false, always_run true. Avoid claiming published binaries. Run CLI tests and strict user-site build; require pass.
- [ ] Include generated verdict/exit/report consistency cases and document formal/proptest/fuzz reproduction commands before this stage's gate.
- [ ] Complete five reviews, emphasizing process behavior, discovery, diagnostics, user instructions, and scope claims. Commit as `feat: expose license checks through the CLI and local hooks`.

## Task 8: Complete workflow, performance evidence, and final review

**Files:** Create `tests/workflow.rs`, `tests/performance.rs`; update `.github/workflows/ci.yml` only for deterministic workflow coverage. Maintain knowledge owners, the development guide, `docs/development/work/initial-design/implementation-reviews.md`, and a performance evidence record. Preserve existing repository hooks unless a tested product hook is justified.

**Interfaces:** Consume the final executable and fixture generators from prior stages; introduce no new policy semantics.

- [ ] Write `local_hook_checks_the_full_lock`: invoke an installed binary through the documented local hook with a fixture project; assert `assert_eq!(hook.status.code(), Some(1));` for a violation in an unchanged/transitive artifact and assert filenames cannot narrow coverage. Run the equivalent CI command against the same project and compare semantic results. Use available prek/pre-commit as an explicit integration prerequisite; no Python interpreter or hook runner becomes a requirement for taloon check itself.
- [ ] Run `cargo test --locked --test workflow`; confirm the integration test fails if any required workflow wiring is absent. Finish that wiring, then require pass. Keep the test offline and independent of release infrastructure.
- [ ] Add an ignored `warm_cache_process_target` benchmark that creates 1,000 external packages and 10,000 distinct local archive artifacts with lock digests, warms an isolated cache through the real CLI, invokes a release binary as a child process, and reports process-start-to-report-completion duration. Assert complete target counts and zero network_requests, downloaded_bytes, and archive_extractions from measured report counters; record local-artifact provenance and any re-hashing cost. Do not seed private cache bytes or describe this as a measured remote cold run. Record the two-second goal without an elapsed-time assertion in normal CI.
- [ ] Run `cargo build --release --locked` and then `TALOON_BENCH_BINARY=target/release/taloon cargo test --release --locked --test performance -- --ignored --nocapture`. Record environment, fixture definition, multiple raw durations, cold/warm distinctions, transferred bytes, and results. If the goal is missed, investigate before claiming performance success; retain observed evidence.
- [ ] Complete five final self-reviews across specification traceability, adversarial interactions, all eight design diagnostics, crash/offline/credential boundaries, and actual verification output. Confirm each prior stage has at least five populated review records and resolved findings. Obtain a fresh independent whole-change review under the selected execution workflow; this supplements self-review.
- [ ] Run the full Rust checks from the development guide, cargo audit, cargo deny licenses/sources, documentation-tooling tests, OKF/link validation, strict MkDocs build, and workflow linters if workflow files changed. Inspect complete outputs. Resolve findings and rerun affected checks; do not repeat unaffected suites merely to count reviews.
- [ ] Run pinned Lean proof/corpus freshness checks, strict Rust correspondence, proptest, and every fuzz smoke target; wire reproducible verification into CI with pinned tooling. Record scope, finite bounds, sensitivity results, and infrastructure limits without claiming model proofs establish the complete implementation.
- [ ] Update baseline/user guidance to describe only verified behavior and supported platforms/formats. Record limitations and review evidence. Commit the finished implementation after checks and reviews; leave merging, deployment, and publishing outside this plan.

## Planning review and selected execution

The planning stage itself receives five self-review passes, recorded in
[plan review](plan-review.md). Future task reviews remain pending until actual
execution; do not pre-fill passing records.

The owner selected inline execution in this session. All eight stages build on
the same identity/evidence/inspection interfaces; inline execution keeps changes
to those interfaces in one context. Complete five self-review passes per stage
and one independent whole-change review at the end. First commit the design and
plan documentation on the current branch, as the owner requested; then execute
without further stage-by-stage permission prompts.

## Dependency documentation consulted during planning

These are planning references, not locked dependencies or verified license
approvals. Re-check exact selected versions and transitive licenses during
implementation:

- [spdx](https://docs.rs/spdx/0.13.6/spdx/): expression parsing/evaluation and the bundled identifier-list version.
- [pep440_rs](https://docs.rs/pep440_rs/0.7.3/pep440_rs/): parsed version values; distinguish equality from range-matching semantics.
- [reqwest blocking](https://docs.rs/reqwest/0.13.5/reqwest/blocking/): a synchronous client suitable for a bounded worker pool.
- [toml](https://docs.rs/toml/1.1.8+spec-1.1.0/toml/): serde-compatible TOML parsing.
