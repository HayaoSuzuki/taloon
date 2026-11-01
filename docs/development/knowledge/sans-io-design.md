---
type: Decision
title: Pure core and acquisition boundary
description: Approved deep modules and request/result API for license inspection without core I/O.
status: accepted
sources:
- id: terminal-text-review
  resource: ../work/terminal-text/review.md
  working_tree: untracked
  sha256: cb3282d33446737e80e3432c30dd0710a7624ef4f306b61aa64d91127ada745e
- id: html-index-filtering-review
  resource: ../work/html-index-filtering/review.md
  working_tree: untracked
  sha256: f040c5b2244aa4d8e5dcaa5c6762316c0a59a1d85e193b32269d1983154b3f55
- id: evidence-ownership-review
  resource: ../work/evidence-ownership/review.md
  working_tree: untracked
  sha256: 9da1686b4f85fe2e04ea4f700aea467fd9ca15b8678e4459b53699adb5a33d78
- id: permission-unit-review
  resource: ../work/permission-units/review.md
  working_tree: untracked
  sha256: 38137e298604b7ba9efc0d82deda8f5ca649d9106e7005866847968929f08eca
- id: policy-evaluation-review
  resource: ../work/policy-evaluation/review.md
  working_tree: untracked
  sha256: fb1ab90f04a90894c3876baf58456f31b3e08be6661077e217065f1d91037ef7
- id: core-design-review
  resource: ../work/core-accounting-and-resolution/review.md
  working_tree: untracked
  sha256: 9dbaff1fe9d2990398de44ec0ee376a531b48e8ae67bf9826a424d62cb870434
- id: describe-review
  resource: ../work/describe-licenses/implementation-review.md
  working_tree: modified
  sha256: da2011db34cf05805f633424b8c977d54a204f09fed2befa5a664eafa3d2bc34
- id: describe-design
  resource: ../work/describe-licenses/design.md
  working_tree: modified
  sha256: 739306eecb05253fec189602923098bd44db6416c2288ca57ef126b12f4b2458
- id: acquisition-measurements
  resource: ../work/index-acquisition/measurements.md
  working_tree: modified
  sha256: 6aaeeade6f9e0f1a495c5328501465ef1aa571660070ecd2957f3b71f4a2f67a
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
The owner approved this design. The [project baseline](project-baseline.md)
records the current implemented slice.[^owner][^package][^approval]

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
wall-clock gate in normal CI; the [baseline](project-baseline.md) links measured results. Record cold-run duration and
bytes separately because artifact sizes and network conditions differ.[^owner]

Run the repository's existing [development checks](../index.md#local-checks)
and [documentation checks](../documentation.md#validate-and-preview) for the
implemented change. Update the user guide only with behavior and prerequisites
that implementation tests establish. Distribution follows the [current distribution decision](python-distribution.md).

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

# Per-run index ownership

The runtime's private IndexCatalog owns endpoint construction, complete locked
URL scope, bounded Simple JSON/HTML parsing and once initialization. Construction
binds one immutable Http/authentication context; lookup returns an exact sidecar
association or failure. The caller continues to own metadata download, artifact
fallback and evidence-cache publication. An immutable precollected endpoint map
avoids a global lock during transport. This hides index protocol/synchronization
from acquisition orchestration without adding a public core API or runtime
dependency. Reconsider if endpoints must be added dynamically or authentication
may vary during a single call; either would invalidate the sealed-scope
premise.[^acquisition-measurements]

[^acquisition-measurements]: [Measured acquisition and stage reviews](../work/index-acquisition/measurements.md).

# Description boundary

The implemented describe boundary keeps selection/conditional-edge knowledge in resolution,
original declarations and completeness in a policy-free description surface,
and request fulfillment/cache in the existing acquisition runtime. It does not
run a full check and filter after acquisition, or add display flags throughout
permission evaluation. The public Description surface accepts lock text/name, returns selected artifact
requests and evaluates observations into a typed description report. Runtime
acquisition/progress is shared with check; resolution retains private graph
selection knowledge. No policy/date/I/O enters Description.[^describe-design]

[^describe-design]: [Proposed description ownership and alternatives](../work/describe-licenses/design.md).

Implementation evidence and staged review limits are recorded separately from
the approved design.[^describe-review]

[^describe-review]: [Staged implementation and validation](../work/describe-licenses/implementation-review.md).


# Shared observations and graph internals

Inspection and description share a private ResultLedger. Its Single/Duplicate
Enum owns cardinality, missing observations, request rebinding and remaining
foreign IDs; Evidence still validates proof and each command still owns its
semantics and required strength. A duplicate slot retains no observation payload,
and cannot later become Single. The public inspection::AcquisitionResult path is
preserved through a re-export. This removes shared protocol knowledge from the
two evaluators without combining permission and display semantics.[^core-design-review]

Resolution owns a borrowed per-document name index and a predecessor arena for
shortest paths. Candidate ordering/validation and `(package, activated extras)`
visited states remain unchanged. Parent links borrow immutable edges and point
only backward in one search arena; annotations are materialized for the chosen
path. The public interface and deterministic tie order stay unchanged.
Performance observations belong in the [measurement record](../work/core-accounting-and-resolution/performance.md).[^core-design-review]

Domain Debug rendering uses the same remote-location redaction owner as display.
Registry/Url identity normalization and opaque locations for other source kinds
are unchanged. In particular, Debug formatting does not construct source or
request identities. Reconsider the private layout only if new consumers need a
different observation protocol or measured workloads justify more indexing.
Do not remove binding validation merely to save copying.[^core-design-review]

[^core-design-review]: [Core assessment and staged reviews](../work/core-accounting-and-resolution/review.md).


# Indexed policy evaluation

Policy owns a PackageId-keyed map of ordered exception entries. The validated
key retains normalized name/version/source semantics, while each entry retains
its original file ordinal and optional content hash. One private matcher serves
proof-requirement and permission queries; unrelated package entries are not
scanned. Evaluation borrows global/scoped permission sets rather than cloning
their union. Usage accounting scans observations once, retaining configured
package/content usage and pending global/clarification entries; diagnostics
retain original ordinals and final report ordering. This implements the earlier
policy-scan follow-up without adding a cache or public API.[^policy-evaluation-review]

Construction still validates every input entry in file order. Hashless local
requests match overrides against observed content, and all OR units count for
unused-permission reporting. Reconsider the layout if measured same-package
fork/hash workloads justify a further index; current measurements isolate lookup
and evaluation, not setup or whole-CLI costs.[^policy-evaluation-review]

[^policy-evaluation-review]: [Policy evaluation assessment and staged review](../work/policy-evaluation/review.md).


# Atomic permission values

Policy stores validated PermissionUnit values, with private canonical text and
Standard/CustomReference classification. Construction reuses the expression
owner and requires one atomic license/WITH term. Scope remains a policy rule:
custom references require hash-bound exceptions. Equality, ordering and borrowed
lookup use canonical spelling; native requirement comparison does not implicitly
change the application's GNU/plus distinctions. No duplicate SPDX catalogue or
AST is introduced. The existing expression units() API is a cached canonical
reporting view, independent of typed permission storage.[^permission-unit-review]

[^permission-unit-review]: [Validated permission unit assessment and reviews](../work/permission-units/review.md).


# Immutable record payloads

EvidenceRecord shares its private immutable payload across clones. Acquisition
and evaluation keep their existing APIs and revalidate each requested binding;
sharing does not confer additional provenance or bypass checks. Parsed metadata
and transient request identity remain in Evidence. The persisted JSON/schema
and checksum inputs stay unchanged. This hides ownership optimization inside
the evidence module instead of introducing consuming/borrowing protocols into
callers. serde's rc support adds no new package, but adds reference-count costs
and a retained allocation (plus a transient Box on deserialization). Reconsider
for measured short-record regressions or new mutation requirements. The staged
record and its linked measurements distinguish avoided payload copies from
unproven overall speed/RSS improvements.[^evidence-ownership-review]

[^evidence-ownership-review]: [Immutable payload staged review](../work/evidence-ownership/review.md).


# HTML index retention

HTML tokenization interprets base/meta/anchor tokens immediately inside the
private index parser. It borrows tokenizer attributes and retains only locked
URL bindings, one optional explicit base and an absorbing Result. This removes
the historical tag vector and duplicate attribute maps while keeping exact
association/hash checks and all-document version rejection. JSON, HTTP body
bounds and tokenizer allocation remain unchanged. Reconsider if new HTML
semantics require a tree builder or measured inputs show a different tradeoff;
no generic parser event API is warranted by the present consumer. The staged
record links complete synthetic timing samples and preservation limits.
[^html-index-filtering-review]

[^html-index-filtering-review]: [HTML index staged review](../work/html-index-filtering/review.md).


# Borrowed terminal rendering

The CLI's private escaping helper returns Cow<str>: unchanged text borrows its
input, while escaped output owns a growable String. Both command renderers keep
their existing Display-based calls and exact control/bidi policy; the pure core
and serialized reports gain no output-lifetime constraint. A private predicate
owns the escape decision once. This removes per-character Vec construction and
ordinary-text duplication without a bespoke type/public abstraction. Reconsider
if actual workloads favor always-owned output or escape policy changes; escaped
text still scans a prefix twice and can grow its output allocation. Measurements
are helper-only and do not establish whole-command speed or RSS improvements.
[^terminal-text-review]

[^terminal-text-review]: [Terminal text staged review](../work/terminal-text/review.md).
