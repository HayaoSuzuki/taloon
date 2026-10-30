---
type: Decision
title: Pure core and acquisition boundary
description: Approved deep modules and request/result API for license inspection without core I/O.
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
- id: package
  resource: ../../../Cargo.toml
  revision: d5985ba2b65571147fbbeb88736e6420c852f86a
- id: uv-metadata
  resource: https://docs.astral.sh/uv/reference/internals/metadata/
---
# Decision and status

Use a pure inspection core that derives artifact/evidence requests from the
lock and policy, then evaluates acquisition results. The runtime owns files,
networking, credentials, wall-clock access, cache persistence, and output. The
owner selected this request/result approach, called A in the design interview.
The [product contract](license-inspection.md) specifies the proposed behavior.
The owner approved this design; implementation remains the Rust scaffold.[^owner][^package][^approval]

The core must determine both target coverage and evidence sufficiency. Keeping
those decisions together prevents an acquisition error or missing result from
silently narrowing a check. The runtime can change HTTP clients, scheduling,
cache layout, or parallelism without changing license rules.[^owner]

# Alternatives

| Approach | Benefit | Cost and disposition |
| --- | --- | --- |
| A: pure inspection requests and results | Small core API; independent acquisition concurrency/cache implementation; deterministic coverage and policy tests | The runtime must fulfill evidence requirements and return failures. Selected by the owner. |
| B: pure effect state machine | Pure tests cover acquisition sequencing, retry, and fallback transitions | Adds command, response, and transition contracts for workflows the acquisition module can own. Defer until real sequencing requirements justify it. |

For lock access, the owner selected direct `uv.lock` parsing without a runtime
uv dependency. uv recommends its workspace metadata command because the lock
format lacks a stability guarantee; that JSON output is also preview at the
time of inspection. A version-aware lock module and fixtures must contain the
resulting compatibility cost. Study uv's implementation without importing its
internal crates in this version.[^owner][^uv-metadata]

# Knowledge ownership

| Module | Responsibility and hidden decisions |
| --- | --- |
| `resolution` | Convert supported lock representations into package/artifact identities, workspace classification, and annotated dependency paths; hide uv fields and version quirks. |
| `policy` | Parse and validate tool.taloon, match scoped overrides, and evaluate SPDX permissions; hide TOML layout, selector normalization, and expiry rules. |
| `evidence` | Verify archive/metadata bytes and interpret declared license evidence; hide wheel/sdist/Core Metadata details and evidence-strength representation. |
| `inspection` | Own the expected target set, required evidence, aggregate outcomes, and diagnostic report; hide coverage accounting and stable result ordering. |
| `acquisition` | Satisfy evidence requests using HTTP, local files, authentication, and cache; hide index protocols, fallback, retries, storage, and concurrency. |
| CLI | Discover inputs, obtain the UTC evaluation date, invoke inspection/acquisition, and render the report; hide argument parsing and output presentation. |

These are responsibility boundaries, not a required file count or crate layout.
The initial implementer choice is to extend the existing package's library with
the pure modules and add a binary for acquisition and CLI wiring. Keep private
helpers private and add interfaces only when a consumer needs the abstraction.
Split crates only if dependency or capability isolation provides a concrete
benefit.[^owner][^package]

Do not expose uv AST nodes to acquisition or raw TOML tables to evaluation.
Use domain package/source/artifact identities. Keep source normalization in one
owner and pass the resulting identity to other modules. The acquisition module
may contain pure index-protocol parsers; a pure function need not belong to a
separate public module.[^owner]

# Core API contract

The following names describe the intended interface. Exact Rust signatures,
ownership, and serialization remain implementer choices.[^owner]

1. `Inspection::from_documents(lock_text, pyproject_text, evaluation_date)`
   validates inputs and creates an opaque immutable inspection. It owns the
   locked graph, policy, expected targets, and initial diagnostics. Global
   parse/configuration failures return a structured input error. Unsupported
   individual sources remain represented as incomplete targets.
2. `inspection.requests()` exposes immutable artifact/evidence requirements.
   Each request has an opaque stable association with a target, a source locator,
   content identity where known, and minimum evidence strength. A requirement
   can accept index-associated metadata or require archive-content verification.
   Applied clarification/custom-LicenseRef requirements come from the core;
   acquisition does not parse policy.
3. `inspection.evaluate(results)` returns a structured report and run status.
   The result collection contains either validated observations or structured
   acquisition failures for request identities. The core checks coverage itself;
   missing observations cannot produce success.

The core performs no filesystem, network, process, environment, or clock access.
It accepts owned/borrowed data, rather than file handles, HTTP clients, callbacks
that perform I/O, or paths that it opens. The same inputs, evidence, and date
must yield the same semantic report. Local source locators are data for the
runtime, not authority for the core to read files.[^owner]

Inspection objects and domain values keep their invariants private. Require
evidence construction/validation through the evidence module rather than a
public caller-set `verified: true` flag. The internal runtime remains trusted to
report the HTTP response and index association it observed. Archive evidence
requires validation against the actual supplied bytes. Streaming verified
observations or per-artifact processing may reduce memory use; they must not
expose I/O into the pure API or retain all archive bodies until final evaluation.
Choose concrete buffering/storage bounds during implementation.[^owner]

## Required invariants

- Each expected target receives a terminal semantic outcome. Missing results,
  insufficient evidence, hash/identity mismatches, and unsupported sources make
  the run incomplete.
- Reject unknown result identities and contradictory duplicates as invalid
  acquisition input. If acquisition deduplicates shared content, it must retain
  the source association for each target. Do not silently take the last result.
- Sidecar evidence cannot satisfy archive-verification requirements. Cache
  deserialization revalidates format, association, and content integrity without
  upgrading provenance.
- Clarification changes the interpreted declaration, not the observed original
  evidence or the required permission. Expiry uses the supplied run date.
- Policy changes require evaluation, even when evidence came from cache. Cache
  misses, network failures, and cache damage cannot remove expected targets.
- Concurrent acquisition order cannot affect semantic results, diagnostic
  ordering, permission matching, or exit status.

These invariants complete the API contract promised in the interview. The
implementer chose duplicate-result handling and representation details within
the delegated design scope.[^owner]

# Runtime and cache behavior

The runtime reads documents and the UTC date, creates the inspection, then
fulfills requests. Acquisition tries compatible cached evidence, permitted local
evidence, standard metadata, or verified archive acquisition as appropriate.
Offline mode forbids network operations throughout those paths. Return typed
failures to inspection when a request cannot be satisfied.[^owner]

Separate source/content identity from the source locator used for fetching.
Normalize policy identity once, retain semantically relevant locator information
internally, and redact secrets for reports/cache provenance. Authentication is
runtime state and does not grant a package a different license identity. Avoid
serializing credentials into the cache. Concrete normalization and secret-safe
display rules require fixtures, especially for direct URLs.[^owner]

The cache owns persistent representation and recovery. Implementer choices:
use versioned records, content digests, atomic record replacement, and
concurrency-safe publication; treat corrupt/incompatible records as misses.
Publish only complete evidence, not provisional success. A failed cache write
may warn while a successful in-memory inspection completes; loss of required
evidence still makes a run incomplete. Preserve separate archive-verified and
sidecar provenance. Bound memory and archive expansion during acquisition;
set concurrency and timeout defaults with measurements.[^owner]

# Acceptance checks

Add behavioral tests with implementation. Product tests must cover observed
outcomes, rather than mirroring helper calls. Keep external network services
out of deterministic tests; use local HTTP fixtures and recorded valid package
metadata/artifacts.[^owner]

| Area | Required behavioral evidence |
| --- | --- |
| Lock coverage | Dev/transitive/extra/group dependencies, platform/version branches, multiple sources/versions, cycles, explicit/implicit workspace roots, external local/Git failures, unsupported lock versions, additive revisions. |
| Artifact coverage | Every recorded wheel/sdist, duplicate content with distinct associations, empty artifact sets, source/metadata identity mismatches, missing and conflicting results. |
| License interpretation | AND/OR/WITH nesting, GNU identifier distinctions, valid legacy License, invalid/absent expressions, classifier/prose non-inference, original versus clarified declarations. |
| Policy | Empty/missing/invalid policy, additive exact selectors, source distinctions, global LicenseRef rejection, hash-scoped custom permission, clarification requiring archive verification, inclusive UTC expiry, unused warning suppression. |
| Acquisition | Hashed sidecars, legacy attribute, absent/unhashed sidecar fallback, index/lock disagreement, verified archives, unavailable sources, local-file content changes, netrc selection, host-changing redirects and redaction. |
| Cache/offline | Policy re-evaluation, evidence-strength requirements, record corruption/version changes, concurrent writers, unavailable cache storage, interruption before publication, zero offline network calls. |
| CLI/report | Upward/explicit discovery, JSON isolation/schema, stable ordering despite acquisition order, aggregate failures, shortest annotated cyclic graph paths, exits 0/1/2 and warning escalation. |
| Workflow | A local pre-commit hook checks the complete lock; CI uses the same policy/command; instructions match tested installation prerequisites. |

Support for a specific uv.lock version/revision must come from documented
fixtures and compatibility tests before the implementation claims it. Reject
unsupported major formats and additions whose semantics affect coverage rather
than ignoring them. Use the Core Metadata compatibility rules for future
versions; document the tested field/schema range and identifier database.
Do not infer support merely because TOML deserialization succeeds.[^owner]

Measure a fixed 1,000-external-dependency, 10,000-artifact workload with a complete
warm evidence cache. The target is two seconds from process start through report
completion. Record the build profile, revision, hardware, OS/filesystem, cache
state, fixture characteristics, and timing method. Check zero network requests,
archive downloads, and archive re-extractions. This is a performance goal, not a
wall-clock gate in normal CI; no result exists yet. Record cold-run duration and
bytes separately because artifact sizes and network conditions differ.[^owner]

Run the repository's existing [development checks](../index.md#local-checks)
and [documentation checks](../documentation.md#validate-and-preview) for the
implemented change. Update the user guide only with behavior and prerequisites
that implementation tests establish. Publishing remains deferred.[^owner]

# Design review criteria

The owner requires at least five separate self-review passes for planning and
each implementation stage. The [implementation plan](../work/initial-design/implementation-plan.md)
defines eight stages and review scopes; record actual findings and verification
before advancing. Repeated test runs alone do not count as reviews.[^approval]

The owner also requires the [formal/generated-test strategy](verification-strategy.md):
Lean model proofs with implementation correspondence, property tests, fuzzing,
and validated Rust types/enums. These extend verification; the runtime remains
independent of Lean and fuzz tooling.[^approval]

Review both correctness and complexity. For each new public interface, require
a comment defining the abstraction, inputs, guarantees, failure conditions, and
invariants. Confirm that callers do not need uv/TOML/HTTP/archive/cache details
outside their owner. A reviewer should explain each module using the ownership
table without reading its implementation. Merge helpers that only forward
arguments and do not hide useful knowledge.[^owner]

Use the eight software-design-philosophy diagnostics in implementation reviews:
one-sentence responsibilities; interfaces simpler than their implementations;
replaceable implementations; abstraction comments; design discussion during
review; meaningful information hiding; understandable boundaries; and explicit
time for design improvement. The written design addresses the two previously
open criteria through the API contract and this review procedure. Code must
earn a fresh assessment after implementation.[^owner]

# Reconsideration and delegated choices

Revisit A if real acquisition sequencing requires callers to duplicate workflow
rules, if evidence accumulation prevents bounded resource use, or if additional
consumers require a different interaction model. Consider crate separation when
capability isolation or consumer dependencies warrant it. Revisit direct lock
parsing when compatibility maintenance outweighs a supported public uv API's
runtime cost.[^owner]

The implementer selects crates, exact type/signature/file layout, network
defaults, cache format/recovery, verified lock compatibility range, configuration
keys, and report schema/ordering. Record those choices and tests. Scope
reductions, weaker success conditions, and new runtime prerequisites require an
owner decision.[^owner]

[^owner]: [Owner decisions and inspected source observations](../work/initial-design/specification-inputs.md).
[^approval]: [Design approval and stage review requirement](../work/initial-design/implementation-approval.md).
[^package]: [Current Rust package](../../../Cargo.toml).
[^uv-metadata]: [uv workspace metadata integration advice and preview schema](https://docs.astral.sh/uv/reference/internals/metadata/).
