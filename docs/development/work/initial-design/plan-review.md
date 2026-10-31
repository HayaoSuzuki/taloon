# Initial implementation plan self reviews

The planning stage received five separate self-review passes on 2026-10-30,
following the owner's [minimum review requirement](implementation-approval.md).
The reviewer was the active Codex agent. These reviews assess the
[implementation plan](implementation-plan.md), not product code or future task
completion.

The reviewed plan snapshot before the owner's verification extensions was
uncommitted on base revision
`d5985ba2b65571147fbbeb88736e6420c852f86a`, SHA-256
`30ccf77b6376eebdddbb6d89fd01deabcd6f5b19474ef5708c0734b2fc324e68`.

## Pass 1 specification coverage

Compared both approved knowledge documents with the eight tasks and their
acceptance checks. Found that the no-uv/no-build/no-mutation requirement had
constraints but lacked a process regression test. Added
`cli_never_resolves_builds_or_mutates_inputs` in task 7, including unchanged
input bytes and sentinel executables/build code.

Coverage map after correction:

| Contract area | Tasks |
| --- | --- |
| Explicit SPDX permissions, scope, expiry, custom licenses, clarification | 1 and 4 |
| Universal dependency/artifact coverage, membership and paths, format limits | 2 and 4 |
| Per-artifact identity, metadata precedence, proof strength and original evidence | 3 and 4 |
| Local archives, persistent evidence, corruption/concurrent publication | 5 |
| Sidecars, archive fallback, HTTPS/authentication, offline transport and aggregation | 6 |
| Discovery, outputs/exits, warning escalation and user instructions | 7 |
| Hook/CI agreement, measured performance and full verification | 8 |

No unassigned contract area remained after this pass. Future implementation must
still demonstrate the behavior with tests.

## Pass 2 step scan

Checked that each stage has files, cross-stage interfaces, named failing tests,
expected assertions, targeted red/green commands, implementation work, a
five-review gate, and a commit boundary. Added concrete assertion snippets to
the eight stages so expected statuses, target counts, and evidence requirements
cannot depend on the implementer's interpretation.

Removed the need for process tests and the benchmark to fabricate private cache
bytes. They now warm an isolated cache through the real CLI and select that
cache using TALOON_CACHE_DIR. This keeps the tests on observable behavior.
The plan has eight stages and 50 checkable steps; an automated structural scan
confirmed files/interfaces/assertions and five-review gates for each stage.

## Pass 3 type consistency

Traced definitions and consumers through policy, resolution, evidence,
inspection, acquisition, and CLI. Found that hash-bound policy operations only
accepted an ArtifactRequest. A local archive without a lock digest needs its
observed content hash for clarification and scoped permission. Added that hash
to Policy::evaluate and Policy::clarification.

Specified ArtifactRequest construction/requirement changes, evidence summary
accessors, PolicyDecision::is_permitted, and Report::with_warnings. Added a
measured AcquisitionSummary and Report::with_acquisition_summary so the CLI and
process benchmark do not depend on absent APIs or guessed activity counters.
The shared interface table names their owners and consumers.

## Pass 4 review focus and failure interactions

Traced the five Review Focus classes into tests in their owning tasks. Checked
partial results, shared content across sources, weak cache evidence after a
policy change, redirects, malformed metadata, and interrupted writes.

Corrected cache expectations: archive-verified content may serve another request
after digest/package validation, while sidecar source associations remain
separate. Added an inspection assertion that both request associations must
still appear in coverage accounting. This permits content deduplication without
allowing a single result to erase another target's policy decision.

The benchmark now asserts measured network/download/extraction counters in
addition to timing and target counts. Its fixture uses local locked archives;
the report must record that scope and avoid claiming a measured remote cold run.

## Pass 5 proportion and completion evidence

Compared plan size with the two design documents: 4,063 words against 3,965
combined design words. The plan has 230 lines and no implementation function
bodies. It fixes interfaces, test outcomes, sequencing, and review gates while
leaving delegated implementation bodies to execution.

Checked that all five passes above contain actual findings/corrections or
specific inspection results. Future implementation reviews remain pending and
have no pre-filled passing entries. The per-stage five-pass rule and final
independent review supplement testing; repeated test runs do not count as
additional reviews.

The plan-level design assessment is 8/8 diagnostics, or 10/10: coherent
responsibilities, small interfaces, replaceable acquisition, interface contracts,
review criteria, hidden format knowledge, documented boundaries, and explicit
design-improvement passes. No row remains open at the planning level. Reassess
the actual code at each stage and record any failed row and required fix.

## Limits

No Rust product code, new dependency lock resolution, product test, hook run, or
performance measurement has occurred. Exact third-party versions and the tested
uv.lock compatibility range remain implementation tasks, as the owner delegated.
The owner subsequently selected inline execution and added the verification
requirements reviewed below.

## Verification extension review on 2026-10-31

The active Codex agent performed five further self-review passes after adding
the owner's formal, property-test, fuzzing and typed-representation requirements:

1. **Coverage:** mapped policy proofs/correspondence to task 1, evidence/coverage/
   status proofs to task 4, parser fuzzing to tasks 1–4, properties to tasks 1–7,
   and proof/corpus/fuzz CI to task 8. No new runtime prerequisite or narrower
   artifact scope was introduced.
2. **Ordering:** found generated-test steps appended after some stage commit
   gates. Moved them before those gates so a stage cannot finish without its
   required proof/property/fuzz checks. The task-brief extractor accepts the
   plan's heading format.
3. **Correspondence:** separated inductive model proofs, finite generated cases,
   public Rust correspondence, and I/O tests. Initial worksheet cases remain
   model-only until real API observations exist; runtime/infrastructure failures
   cannot count as semantic matches. No expected corpus is hand-maintained.
4. **Representation and failure modes:** required newtypes for validated values
   and exhaustive enums for source/evidence/outcome/code domains. Kept remote
   hash requirements and unsupported-source outcomes explicit, and retained
   sidecar-promotion/missing-result/precedence broken witnesses.
5. **Resource and execution checks:** bounded Lean commands and fuzz campaigns,
   kept heavy enumeration out of imported proof modules, preserved the five
   self-reviews per task, and selected inline execution without more permission
   prompts. Checked the plan's types against its original interfaces; the
   verification extension does not rename cross-stage production APIs.

The post-extension plan snapshot has SHA-256
`db7531cebb6535bd8da6ecb671d87662b5995ae3995a9a6e50d62b1e6a73ab6c`.
This review records planning work only. Actual proofs, property tests, fuzz
campaigns, and implementation-stage self-reviews remain execution obligations.
