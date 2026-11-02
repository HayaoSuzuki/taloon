# Bounded retry for pinned Creusot image construction

The required formal job failed before proofs while fetching the pinned Ubuntu
snapshot's signed package indexes: [attempt 1](https://github.com/HayaoSuzuki/taloon/actions/runs/41374666152/job/125052207248)
reported HTTP 502; an unchanged-input rerun reported HTTP 503 in prerequisites.
These are acquisition failures, not failed Rust contracts. The pinned upstream
Dockerfile remains byte-identical. Later exact-date InRelease responses returned
HTTP 200 and their signatures verified with the pinned Ubuntu base's archive
keyring. HTTP already redirects to HTTPS; changing scheme, snapshot, mirror or
recipe bytes is not justified by this evidence.

## Change and bounds

Retry only Docker image construction once, after ten seconds. Serialize one
context from the checksum-verified source and reuse its exact bytes and architecture
arguments for both attempts, retaining Docker's successful layers. Source fetch,
archive checksum, extraction and context creation still fail immediately.
A second Docker failure preserves its status. No version, source/GPG/hash,
preflight, solver, proof or named-goal gate is removed. Existing 1,200-second build
and 30-minute job deadlines bound the combined attempts. All Docker failures are
retried once; deterministic failures can consume more of that unchanged budget.
This is simpler and more stable than parsing BuildKit error prose, and does not
promise recovery during a sustained upstream outage.

## Verification and review

Seven named synthetic scenarios exercise first-attempt success, recovered amd64
and arm64 builds, persistent failure, checksum failure, fetch failure and invalid
architecture. They use actual tar/checksum tools and local gh/docker/sleep doubles;
only a temporary wrapper copy's checksum is changed to identify the synthetic
archive. Assertions compare context bytes and full arguments independently, verify
one source fetch, ten-second backoff, preserved final exit status and cleanup.
The retry-success case failed before the wrapper change and passes afterward.

Five executable mutations were rejected: missing retry, checksum bypass, wrong
ARM platform, swallowed final failure and wrong backoff. A mutation exposed a
Bash conditional/errexit assertion weakness; explicit failure exits corrected the
test harness before rerunning the witnesses. This is finite executable sensitivity,
not a formal proof of every shell/interruption behavior.

Five self-reviews covered (1) the observed upstream failure and narrow retry scope,
(2) immutable checksum-verified inputs and downstream gates, (3) bounded attempts,
architecture parameters and resource budget, (4) real orchestration assertions and
negative sensitivity, and (5) source/documentation accuracy and observed limits.
An independent reviewer found no Critical/Important issue and ran all seven cases
with macOS Bash 3.2.57. It did not run Linux, real image construction, network
acquisition or proofs. The helper runs before image construction in the existing
required CI context; installation/reproduction belongs to the
[Creusot procedure](../../../../formal/creusot/README.md).

Bash syntax, fixed ShellCheck, workflow lint and zizmor passed locally. Actual
image/proof outcomes remain separately observable on the PR. No application
behavior, dependency graph or proof obligation is changed by this patch.

Parent integration additionally passed 137 Python tooling tests, the local hook
workflow regression, documentation/OKF validation and strict user-site build.
Only orchestration/docs/workflow files changed; shipping Rust and proof bodies
are unchanged. The retry helper and workflow/static checks passed again in the
parent session. These local checks do not assert an unobserved hosted result.
