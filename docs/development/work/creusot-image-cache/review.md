# Exact-key Creusot environment reuse

## Incident, scope and design

The [main formal job](https://github.com/HayaoSuzuki/taloon/actions/runs/41417270475/job/125184075622)
for 5186feae76fc359f4f562ab9983b63183172f9f8 failed on the fixed Ubuntu snapshot's
signed InRelease indexes: HTTP 503 on both existing Docker build attempts.
The [release gate](https://github.com/HayaoSuzuki/taloon/actions/runs/41417270508/job/125184226479)
correctly rejected that failed main CI. Other main jobs, documentation, dependency
audit and all platform wheel smoke jobs succeeded. These are acquisition and
consequent gating failures, not failed production contracts or a publisher bug.

Direct checks of the same HTTPS noble and noble-updates InRelease URLs on
2026-11-02 also returned HTTP 503. The response said no server was available.
Repeated short builds are insufficient during a sustained outage. The existing
bounded retry remains; neither a mutable mirror nor skipped checks is acceptable.

Following hoimin's existing tool-image cache pattern, cache only the serialized
amd64 tool image in GitHub Actions. The exact key contains a format epoch, runner
OS/architecture and the contents of build-image.sh and prepare-image.sh. The
former pins the upstream recipe revision, archive checksum and build arguments;
there are no restore prefixes. A changed preparation/builder/recipe gets a new
key. The source archive, tool versions and Docker recipe are unchanged.

The CI helper loads an exact cache hit or invokes the existing pinned builder.
It requires amd64 and executes the image's installation-check CMD on both paths.
Only a successful cold build/validation is serialized and saved. Load, inspect,
validation and save failures stop the job; an invalid cached image is not silently
replaced or accepted. All project preflights, fresh generation/proofs on three
targets, inventories, goal checks and compiling mutations still run afterward.
No proof outcomes, project targets or source-dependent results are cached.

GitHub's repository/ref cache scoping is a trust boundary, not a cryptographic
attestation of an arbitrary Docker archive. PR-scoped entries are not main cache
entries. No cache from another repository, arbitrary external image, mutable tag
or cross-key fallback is accepted by this workflow. Check-installation and the
existing source/config preflights are validation, not a substitute for that cache
access boundary. Corrupt/invalid caches fail; invalidate their cache entry to
permit a fresh pinned build after acquisition recovers.

This mitigation needs a first successful cold build. It cannot bootstrap during
an upstream outage and does not modify the workflow attached to an already failed
main commit. Recover the original main CI first, then rerun its original release
workflow; its tag/reservation is reused. Keep the release gate unchanged. No
rerun or publication is claimed without a separately observed hosted result.

## Implementation plan and five-pass reviews

1. Cause and scope: traced release failure to exact-source CI, and that failure
   to apt acquisition before proofs. Confirmed sustained 503 directly rather
   than changing the gate or retrying known-broken inputs indefinitely.
2. Interface/identity: one CI-only helper accepts archive path plus exact true/false
   hit state. Both shell scripts participate in the key, binding the pinned recipe
   inputs and validation behavior. No new application/configuration API.
3. Failure/resource boundary: retain the 1,200-second preparation and 30-minute
   formal-job deadlines, existing two-attempt builder and all downstream checks.
   No always-save path, prefix restore or fallback after validation failure.
4. Executable checks: the initial no-cache helper rebuilt even on a reported hit;
   the new warm-path test rejected it before implementation. Ten named synthetic
   cases cover hit/miss, failed build/load, cold/warm failed preflight, wrong CPU,
   invalid mode, failed inspect and failed save. Seven existing bounded-builder
   cases also pass. Helpers use real preparation control flow and local external
   tool doubles, with AAA and independent cases.
5. Sensitivity/review: five syntactically valid helper mutations were rejected:
   skipped preflight, rebuilding a hit, wrong CPU, skipped serialization and
   swallowed external-command failure. A fresh-context reviewer completed five
   passes and found no Critical/Important issue. Its Minor suggestion to cover
   inspect/save failures was implemented as two additional cases. The review
   independently ran the initial eight preparation and seven builder cases;
   the parent ran the final ten preparation cases.

Documentation/verification five-pass review: (1) update only the existing formal
procedure and owning verification concept; (2) distinguish outage recovery from
future cache reuse; (3) state cache trust/scoping and cold-bootstrap limits;
(4) record the actual failed source and local versus hosted evidence; (5) retain
source hashes and validate documentation metadata. No user guide update is needed:
application behavior, Python code, dependency graphs and distribution gates are
unchanged. This PR concerns only formal-environment acquisition reliability.

Local macOS verification: 17 shell cases, Bash syntax, pinned ShellCheck,
actionlint and all workflow run-block ShellCheck, offline zizmor, 137 randomized
Python tooling tests, OKF/link validation and strict user-site build. The first
workflow lint attempt could not use uv's default cache inside the sandbox;
rerunning with authorized cache access succeeded. No Docker daemon was available
locally: real image load/save, hosted cache hit and proofs remain hosted evidence,
not claims established by the synthetic tests. No Rust production code or formal
contract changes require a new proof claim.
