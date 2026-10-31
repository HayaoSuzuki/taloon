# Independent whole-change review

On 2026-10-31, a fresh-context GPT-6-astra reviewer inspected the implementation
read-only, without implementation-session history or delegated subreviews.
The candidate was `d5985ba..a4914e4` plus pending stage-8 changes, including
workflow/performance tests and the custom-LicenseRef OR correction. The dispatch
patch SHA-256 was
`d8f644220b7be0726dcef30d1a8d5bf7bf1e2567aa8c18c6fd1456a6a2c7fc47`.
The reviewer inspected current sources and reproduced findings in temporary
fixtures outside the checkout. This is a code review, not a proof of the system.

## Findings and executor disposition

| Finding | Reviewer grade | Executor disposition | Verification |
| --- | --- | --- | --- |
| Same-content distinct sidecars overwrite one weak cache filename | Important | Include exact RequestId in weak cache keys; archive keys remain content-reusable | A failing two-sidecar warm/offline test observed one hit instead of two; after the fix, both associations survive with two hits, zero network, and complete compliance |
| An unreachable unsupported dependency's source is only an opaque fingerprint | Minor | Regraded Important because actionable redacted-source diagnostics are in the contract; add structured source kind/location and render it in text | A failing public report regression expected Git source context; after the fix, context is present and credentials/query values remain absent |

No Critical finding was established. The reviewer assessed the candidate as
"With fixes". The executor applied one consolidated correction pass, checked each
regression red-to-green, and ran the complete green suite. The reviewer was not
asked to rubber-stamp the fixes in a second review.

The review explicitly checked URL matching/redaction, metadata ambiguity,
workspace/path classification, cache interruption/strength upgrades, and failed
or reordered acquisition results. It found the core's coverage ownership,
source/request proof binding, and separate acquisition responsibilities coherent.

## Behaviors set aside and executor rulings

- Publishing/release automation and Poetry/cargo-deny compatibility remain outside
  the approved implementation; adding them requires new scope and verification.
- Inline URL-userinfo authentication remains outside the selected netrc mechanism;
  userinfo is stripped. Supporting it later requires an explicit precedence rule.
- Historical uv.lock revisions beyond version 1 revision 5 remain unsupported;
  supporting more needs versioned fixtures and wire-contract review.
- Additional archive encodings remain unsupported; they need bounded readers and
  adversarial fixtures before acceptance.
- Pending final evidence/checklist updates were work in progress at dispatch;
  the executor completed them with actual results rather than treating them as
  missing product functionality.

## Other final self-review corrections

The executor separately found an overbroad archive-proof requirement for every
LicenseRef occurrence. An ungranted custom OR alternative now remains a denied
unit; the other permitted branch can satisfy the expression using sufficient
index-associated evidence. Archive proof is still required for applicable custom
grants and clarifications. A public failing regression and expanded Lean-generated
correspondence cover the interaction.

An empty editable/virtual location could also disappear as an implicit workspace
root through vacuous path-component matching. SourceId now rejects empty source
locations before membership classification; its failing success-versus-incomplete
regression now yields exit 2.

The combined actionlint/ShellCheck subprocess invocation stalled locally despite
working version commands. Bounded reproductions confirmed actionlint core and
standalone ShellCheck both passed. The shared
[workflow helper](../../../../tools/check_workflows.sh) runs the same pinned tools
separately and checks every Bash run block, including its own shell source.
The final run checked 25 workflow blocks; zizmor reported no findings. CI uses the
same helper. No claim is made that a GitHub-hosted workflow has run before push.


The final reporting trace also found that text failures omitted individual unit
permissions already present in JSON. A failing CLI test now verifies the full
expression and every permitted/denied unit in a compound AND/OR failure.

A maximum-size metadata roundtrip exposed a persistence-bound mismatch: supported
4 MiB headers can expand beyond an 8 MiB JSON read cap. Its regression failed
following a successful write. Acquisition and persistence now share the core's
metadata limit; the encoded-record bound allows six-fold JSON escaping plus
64 KiB of envelope headroom. Cache publication buffers writes before flush,
sync, and atomic rename. The exact-boundary observation now survives restoration.

A read-only live HTTPS smoke used an isolated one-package/two-artifact derivative
of [this repository's public docs lock](../../../../uv.lock), with an empty netrc
and its own temporary cache. The installed release code completed the online
check with four HTTPS requests, 519,622 response-body bytes, and one archive
extraction. The following offline check completed with two cache hits and zero
requests, bytes, or extractions. It installed or executed no Python package.
This complements fixture-based successful/failing acquisition tests; it does
not claim a full live scan of another project.


Applied overrides now also carry an explicit identity: a one-based exception
array position or clarification content hash. The public regression distinguishes
three applied entries sharing the same reason. This closes the final report
traceability gap without changing permission evaluation.

## CI follow-up: prepare the separate fuzz dependency graph

The first hosted dependency-audit job at `9a7169a` passed both vulnerability
audits and the shipping license/source policy, then failed in the offline fuzz
policy helper. Its fresh runner lacked `libfuzzer-sys 0.4.13` and `arbitrary
1.5.0`. The earlier macOS result depended on an already-populated Cargo cache;
it did not establish the fresh-runner prerequisite. See the
[failed hosted job](https://github.com/HayaoSuzuki/taloon/actions/runs/41143446575/job/124362904723).

The dependency workflow now fetches `fuzz/Cargo.toml` with `--locked` immediately
before the offline helper. The developer procedure records the same prerequisite.
No lockfile, permission, license allowance, shipping behavior, or formal-model
contract changes; no OKF decision update is required for this preparation fix.

Five self-review passes checked the following independent concerns:

1. Failure attribution: the hosted log identifies missing crate downloads with
   offline enabled, after successful shipping checks; it is not a license denial.
2. Dependency selection: the fetch selects the separate fuzz manifest and its
   existing lockfile, rather than relying on the shipping graph.
3. Execution order: fetch precedes the helper in the same job and Cargo home;
   an unsuccessful fetch stops the job before inspection.
4. Policy preservation: the helper retains offline/locked operation, exact
   development-only exceptions, and warnings as errors. No cache is required
   for correctness.
5. Verification and documentation: local locked fetching with the existing cache,
   actionlint, ShellCheck for all 26 workflow run blocks, documentation checks,
   and all 28 Python tooling tests passed. Hosted verification of the fix is
   recorded separately when available; local cache success alone is insufficient.
